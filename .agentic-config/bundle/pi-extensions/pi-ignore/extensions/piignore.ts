/**
 * Pi extension: `.piignore` overrides for gitignored files.
 *
 * Pi's find/grep tools and `@` autocomplete use fd/ripgrep and respect .gitignore.
 * A project `.piignore` file uses gitignore negation syntax (`!pattern`) to
 * re-include gitignored paths for agent discovery only.
 *
 * fd does not honor negation in --ignore-file, so this extension merges normal
 * results with a filtered `--no-ignore-vcs` scan of negated paths.
 *
 * Path visibility is determined via `git check-ignore -v`, which gives the
 * authoritative answer for whether a path is excluded by `.gitignore` (without
 * depending on ripgrep's `--ignore-file` behavior, which does not handle
 * path-based negation patterns correctly).
 *
 * Example `.piignore`:
 *   !docs/issues/
 *   !docs/issues/**
 *   !docs/*prd*.md
 */

import { existsSync, readFileSync, statSync } from "node:fs";
import { access, readFile as fsReadFile, stat as fsStat } from "node:fs/promises";
import { constants } from "node:fs";
import { spawnSync } from "node:child_process";
import { createInterface } from "node:readline";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { spawn } from "node:child_process";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import {
  createFindTool,
  createFindToolDefinition,
  createGrepTool,
  createGrepToolDefinition,
  DEFAULT_MAX_BYTES,
  type FindOperations,
  type GrepToolInput,
  formatSize,
  getAgentDir,
  truncateHead,
  truncateLine
} from "@earendil-works/pi-coding-agent";
import type { AutocompleteItem, AutocompleteProvider, AutocompleteSuggestions } from "@earendil-works/pi-tui";

const PIIGNORE = ".piignore";
const FIND_DEFAULT_LIMIT = 1000;
const GREP_DEFAULT_LIMIT = 100;

type PathVisibilityChecker = (relPath: string) => boolean;

export function toPosixPath(value: string): string {
  return value.split(sep).join("/");
}

/** Strip leading `@` and resolve against cwd, matching built-in tool path handling. */
export function resolveSearchPath(filePath: string | undefined, cwd: string): string {
  const raw = filePath ?? ".";
  const stripped = raw.startsWith("@") ? raw.slice(1) : raw;
  return resolve(cwd, stripped);
}

export function normalizeFindPattern(pattern: string): {
  effectivePattern: string;
  useFullPath: boolean;
} {
  let effectivePattern = pattern;
  let useFullPath = false;
  if (pattern.includes("/")) {
    useFullPath = true;
    if (!pattern.startsWith("/") && !pattern.startsWith("**/") && pattern !== "**") {
      effectivePattern = `**/${pattern}`;
    }
  }
  return { effectivePattern, useFullPath };
}

function resolveToolBinary(name: "fd" | "rg" | "git"): string | null {
  const local = join(getAgentDir(), "bin", name);
  if (existsSync(local)) {
    return local;
  }

  if (name === "git") {
    const probe = spawnSync("git", ["--version"], { stdio: "ignore" });
    if (probe.error === undefined || probe.error === null) {
      return "git";
    }
    return null;
  }

  const candidates = name === "fd" ? ["fd", "fdfind"] : ["rg"];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { stdio: "ignore" });
    if (probe.error === undefined || probe.error === null) {
      return candidate;
    }
  }

  return null;
}

/**
 * Match a gitignore-style pattern against a path.
 *
 * Handles:
 *  - Basename patterns (no /): match against filename only
 *  - Path patterns (with /): match against full path
 *  - ** (match any number of directories)
 *  - * (match any characters except /)
 *  - ? (match any single character except /)
 *  - Trailing / (match only directories)
 */
export function matchesGitignorePattern(pattern: string, path: string): boolean {
  const hasSlash = pattern.includes("/");
  const hasTrailingSlash = pattern.endsWith("/");

  // Convert pattern to regex
  let regex = "";
  let i = 0;
  while (i < pattern.length) {
    const ch = pattern[i];
    if (ch === "*" && pattern[i + 1] === "*") {
      // ** matches any number of directories (including zero)
      // This is equivalent to .* which matches any characters
      regex += ".*";
      i += 2;
      // Skip trailing /
      if (i < pattern.length && pattern[i] === "/") {
        i++;
      }
    } else if (ch === "*") {
      regex += "[^/]*";
      i++;
    } else if (ch === "?") {
      regex += "[^/]";
      i++;
    } else if (/[.+^${}()|[\]\\]/.test(ch)) {
      regex += `\\${ch}`;
      i++;
    } else {
      regex += ch;
      i++;
    }
  }

  // If pattern doesn't contain /, match against basename
  if (!hasSlash) {
    const basename = path.split("/").pop() ?? path;
    if (hasTrailingSlash) {
      // Only match directories (paths ending with /)
      return new RegExp(`^${regex}/$`).test(path);
    }
    return new RegExp(`^${regex}$`).test(basename);
  }

  // Path pattern: match against full path (trailing / is already in regex)
  return new RegExp(`^${regex}$`).test(path);
}

export function createPathVisibilityChecker(projectRoot: string, piignoreFiles: string[]): PathVisibilityChecker {
  const gitPath = resolveToolBinary("git");
  const rgPath = resolveToolBinary("rg");
  const cache = new Map<string, boolean>();

  // Parse .piignore negation patterns once
  const negatedPatterns = extractNegatedPatterns(piignoreFiles);

  return (relPath: string): boolean => {
    const normalized = toPosixPath(relPath);
    const cached = cache.get(normalized);
    if (cached !== undefined) {
      return cached;
    }

    let visible: boolean;

    if (gitPath) {
      // Use git's own .gitignore resolution — authoritative and handles
      // path-based patterns correctly, unlike ripgrep's --ignore-file.
      // Exit 0: path is gitignored
      // Exit 1: path is not gitignored (already found by normal scan)
      // Exit 128: git error (not a git repo or other issue)
      const result = spawnSync(gitPath, ["check-ignore", "-v", normalized], {
        cwd: projectRoot,
        encoding: "utf-8"
      });
      const isGitIgnored = result.status === 0;

      if (!isGitIgnored) {
        // Path is not gitignored → visible (already found by normal scan)
        visible = true;
      } else {
        // Path is gitignored → visible only if negated in .piignore
        visible = negatedPatterns.some((pattern) => matchesGitignorePattern(pattern, normalized));
      }
    } else if (rgPath) {
      // Fallback: ripgrep with --ignore-file (does not handle path-based
      // negation patterns correctly, but better than nothing).
      const args = ["--files", "--hidden"];
      for (const ignoreFile of piignoreFiles) {
        args.push("--ignore-file", ignoreFile);
      }
      args.push("--glob", normalized, ".");
      const result = spawnSync(rgPath, args, {
        cwd: projectRoot,
        encoding: "utf-8"
      });
      visible = result.stdout.trim().length > 0;
    } else {
      cache.set(normalized, false);
      return false;
    }

    cache.set(normalized, visible);
    return visible;
  };
}

export function discoverPiignoreFiles(searchDir: string, projectRoot: string): string[] {
  const files: string[] = [];
  let dir = resolve(searchDir);
  const stop = resolve(projectRoot);

  while (true) {
    const candidate = join(dir, PIIGNORE);
    if (existsSync(candidate)) {
      files.push(candidate);
    }
    if (dir === stop) {
      break;
    }
    const parent = dirname(dir);
    if (parent === dir) {
      break;
    }
    dir = parent;
  }

  return files;
}

export function extractNegatedPatterns(piignoreFiles: string[]): string[] {
  const patterns: string[] = [];
  for (const file of piignoreFiles) {
    try {
      for (const line of readFileSync(file, "utf-8").split(/\r?\n/)) {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith("#") && trimmed.startsWith("!")) {
          patterns.push(trimmed.slice(1));
        }
      }
    } catch {
      // ignore unreadable piignore
    }
  }
  return [...new Set(patterns)];
}

export function resolveNegationScan(projectRoot: string, negatedPattern: string): { scanRoot: string; glob: string } {
  const normalized = toPosixPath(negatedPattern.replace(/^\//, ""));
  const segments = normalized.split("/").filter(Boolean);
  if (segments.length === 0) {
    return { scanRoot: projectRoot, glob: normalized };
  }

  let prefixLength = 0;
  for (const segment of segments) {
    if (segment.includes("*") || segment.includes("?") || segment.includes("[")) {
      break;
    }
    prefixLength += 1;
  }

  if (prefixLength === 0) {
    return { scanRoot: projectRoot, glob: normalized };
  }

  const dirSegments = segments.slice(0, prefixLength);
  const globSegments = segments.slice(prefixLength);
  const scanRoot = join(projectRoot, ...dirSegments);
  return {
    scanRoot,
    glob: globSegments.length > 0 ? globSegments.join("/") : "**"
  };
}

export function patternOverlapsSearch(negatedPattern: string, searchPath: string, projectRoot: string): boolean {
  const normalizedPattern = toPosixPath(negatedPattern.replace(/^\//, ""));
  const relSearch = toPosixPath(relative(projectRoot, searchPath) || ".");
  if (relSearch === ".") {
    return true;
  }
  return (
    normalizedPattern.startsWith(`${relSearch}/`) ||
    relSearch.startsWith(`${normalizedPattern.split("/")[0]}/`) ||
    normalizedPattern.startsWith(relSearch)
  );
}

async function runFdPaths(fdPath: string, baseDir: string, args: string[], signal?: AbortSignal): Promise<string[]> {
  return new Promise((resolvePromise, reject) => {
    if (signal?.aborted) {
      reject(new Error("Operation aborted"));
      return;
    }

    const child = spawn(fdPath, args, { stdio: ["ignore", "pipe", "pipe"] });
    const rl = createInterface({ input: child.stdout });
    let stderr = "";
    const lines: string[] = [];
    let settled = false;

    const settle = (fn: () => void): void => {
      if (settled) {
        return;
      }
      settled = true;
      signal?.removeEventListener("abort", onAbort);
      fn();
    };

    const onAbort = (): void => {
      if (!child.killed) {
        child.kill();
      }
      settle(() => reject(new Error("Operation aborted")));
    };

    signal?.addEventListener("abort", onAbort, { once: true });

    child.stderr?.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    rl.on("line", (line) => {
      lines.push(line);
    });

    child.on("error", (error) => {
      rl.close();
      settle(() => reject(new Error(`Failed to run fd: ${error.message}`)));
    });

    child.on("close", (code) => {
      rl.close();
      if (signal?.aborted) {
        settle(() => reject(new Error("Operation aborted")));
        return;
      }
      if (code !== 0 && lines.length === 0) {
        settle(() => reject(new Error(stderr.trim() || `fd exited with code ${code}`)));
        return;
      }
      settle(() => resolvePromise(lines));
    });
  });
}

async function runBuiltinFdSearch(
  fdPath: string,
  searchPath: string,
  pattern: string,
  limit: number,
  signal?: AbortSignal
): Promise<string[]> {
  const args = ["--glob", "--color=never", "--hidden", "--no-require-git", "--max-results", String(limit)];
  const { effectivePattern, useFullPath } = normalizeFindPattern(pattern);
  if (useFullPath) {
    args.push("--full-path");
  }
  args.push("--", effectivePattern, searchPath);

  const rawLines = await runFdPaths(fdPath, searchPath, args, signal);
  const relativized: string[] = [];
  for (const rawLine of rawLines) {
    const line = rawLine.replace(/\r$/, "").trim();
    if (!line) {
      continue;
    }
    const hadTrailingSlash = line.endsWith("/") || line.endsWith("\\");
    let relativePath = line.startsWith(searchPath) ? line.slice(searchPath.length + 1) : relative(searchPath, line);
    if (hadTrailingSlash && !relativePath.endsWith("/")) {
      relativePath += "/";
    }
    relativized.push(toPosixPath(relativePath));
  }
  return relativized;
}

async function collectPiignoredPaths(
  projectRoot: string,
  searchPath: string,
  piignoreFiles: string[],
  isVisible: PathVisibilityChecker,
  signal?: AbortSignal,
  limit = FIND_DEFAULT_LIMIT,
  userPattern?: string
): Promise<string[]> {
  const fdPath = resolveToolBinary("fd");
  if (!fdPath || piignoreFiles.length === 0) {
    return [];
  }

  const negatedPatterns = extractNegatedPatterns(piignoreFiles);
  if (negatedPatterns.length === 0) {
    return [];
  }

  const found = new Set<string>();
  for (const pattern of negatedPatterns) {
    if (!patternOverlapsSearch(pattern, searchPath, projectRoot)) {
      continue;
    }

    const { scanRoot, glob } = resolveNegationScan(projectRoot, pattern);
    if (!existsSync(scanRoot)) {
      continue;
    }

    const args = [
      "--base-directory",
      scanRoot,
      "--glob",
      glob,
      "--color=never",
      "--hidden",
      "--no-ignore-vcs",
      "--no-require-git",
      "--max-results",
      String(limit),
      "."
    ];

    if (userPattern) {
      const { effectivePattern, useFullPath } = normalizeFindPattern(userPattern);
      if (useFullPath) {
        args.push("--full-path");
      }
      args.push("--glob", effectivePattern);
    }

    let lines: string[];
    try {
      lines = await runFdPaths(fdPath, scanRoot, args, signal);
    } catch {
      continue;
    }

    for (const rawLine of lines) {
      const line = toPosixPath(rawLine.replace(/\r$/, "").trim().replace(/^\.\//, ""));
      if (!line || line.endsWith("/")) {
        continue;
      }
      const absolutePath = resolve(scanRoot, line);
      const relToProject = toPosixPath(relative(projectRoot, absolutePath));
      if (!relToProject || relToProject.startsWith("..")) {
        continue;
      }
      if (!isVisible(relToProject)) {
        continue;
      }
      if (!absolutePath.startsWith(searchPath)) {
        continue;
      }
      found.add(toPosixPath(relative(searchPath, absolutePath)));
    }
  }

  return [...found];
}

function createPiignoreFindOperations(projectRoot: string, piignoreFiles: string[]): FindOperations {
  return {
    exists: async (absolutePath: string) => {
      try {
        await access(absolutePath, constants.F_OK);
        return true;
      } catch {
        return false;
      }
    },
    glob: async (pattern, searchPath, { limit }) => {
      const fdPath = resolveToolBinary("fd");
      if (!fdPath) {
        throw new Error("fd is not available and could not be located");
      }

      const isVisible = createPathVisibilityChecker(projectRoot, piignoreFiles);
      const [builtinPaths, extras] = await Promise.all([
        runBuiltinFdSearch(fdPath, searchPath, pattern, limit),
        collectPiignoredPaths(projectRoot, searchPath, piignoreFiles, isVisible, undefined, limit, pattern)
      ]);

      const results = new Set<string>();
      for (const relPath of builtinPaths) {
        results.add(resolve(searchPath, relPath));
      }
      for (const extra of extras) {
        results.add(resolve(searchPath, extra));
      }
      return [...results];
    }
  };
}

const findToolCache = new Map<string, ReturnType<typeof createFindTool>>();
const grepToolCache = new Map<string, ReturnType<typeof createGrepTool>>();

function getBuiltinFind(cwd: string): ReturnType<typeof createFindTool> {
  let tool = findToolCache.get(cwd);
  if (!tool) {
    tool = createFindTool(cwd);
    findToolCache.set(cwd, tool);
  }
  return tool;
}

function getBuiltinGrep(cwd: string): ReturnType<typeof createGrepTool> {
  let tool = grepToolCache.get(cwd);
  if (!tool) {
    tool = createGrepTool(cwd);
    grepToolCache.set(cwd, tool);
  }
  return tool;
}

async function executePiignoreGrep(
  cwd: string,
  params: GrepToolInput,
  piignoreFiles: string[],
  signal?: AbortSignal
): Promise<Awaited<ReturnType<ReturnType<typeof createGrepTool>["execute"]>>> {
  const rgPath = resolveToolBinary("rg");
  if (!rgPath) {
    throw new Error("ripgrep (rg) is not available and could not be located");
  }

  const searchPath = resolveSearchPath(params.path, cwd);
  let isDirectory: boolean;
  try {
    isDirectory = (await fsStat(searchPath)).isDirectory();
  } catch {
    throw new Error(`Path not found: ${searchPath}`);
  }

  const contextValue = params.context && params.context > 0 ? params.context : 0;
  const effectiveLimit = Math.max(1, params.limit ?? GREP_DEFAULT_LIMIT);

  const formatPath = (filePath: string): string => {
    if (isDirectory) {
      const rel = relative(searchPath, filePath);
      if (rel && !rel.startsWith("..")) {
        return toPosixPath(rel);
      }
    }
    return basename(filePath);
  };

  const fileCache = new Map<string, string[]>();
  const getFileLines = async (filePath: string): Promise<string[]> => {
    let lines = fileCache.get(filePath);
    if (!lines) {
      try {
        const content = await fsReadFile(filePath, "utf-8");
        lines = content.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
      } catch {
        lines = [];
      }
      fileCache.set(filePath, lines);
    }
    return lines;
  };

  return new Promise((resolvePromise, reject) => {
    if (signal?.aborted) {
      reject(new Error("Operation aborted"));
      return;
    }

    const args = ["--json", "--line-number", "--color=never", "--hidden"];
    for (const ignoreFile of piignoreFiles) {
      args.push("--ignore-file", ignoreFile);
    }
    if (params.ignoreCase) {
      args.push("--ignore-case");
    }
    if (params.literal) {
      args.push("--fixed-strings");
    }
    if (params.glob) {
      args.push("--glob", params.glob);
    }
    args.push("--", params.pattern, searchPath);

    const child = spawn(rgPath, args, { stdio: ["ignore", "pipe", "pipe"] });
    const rl = createInterface({ input: child.stdout });
    let stderr = "";
    let matchCount = 0;
    let matchLimitReached = false;
    let linesTruncated = false;
    let aborted = false;
    let killedDueToLimit = false;
    const outputLines: string[] = [];

    const cleanup = (): void => {
      rl.close();
      signal?.removeEventListener("abort", onAbort);
    };

    const stopChild = (dueToLimit = false): void => {
      if (!child.killed) {
        killedDueToLimit = dueToLimit;
        child.kill();
      }
    };

    const onAbort = (): void => {
      aborted = true;
      stopChild();
    };
    signal?.addEventListener("abort", onAbort, { once: true });

    child.stderr?.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    const formatBlock = async (filePath: string, lineNumber: number): Promise<string[]> => {
      const relativePath = formatPath(filePath);
      const lines = await getFileLines(filePath);
      if (!lines.length) {
        return [`${relativePath}:${lineNumber}: (unable to read file)`];
      }
      const block: string[] = [];
      const start = contextValue > 0 ? Math.max(1, lineNumber - contextValue) : lineNumber;
      const end = contextValue > 0 ? Math.min(lines.length, lineNumber + contextValue) : lineNumber;
      for (let current = start; current <= end; current++) {
        const lineText = lines[current - 1] ?? "";
        const sanitized = lineText.replace(/\r/g, "");
        const isMatchLine = current === lineNumber;
        const { text: truncatedText, wasTruncated } = truncateLine(sanitized);
        if (wasTruncated) {
          linesTruncated = true;
        }
        if (isMatchLine) {
          block.push(`${relativePath}:${current}: ${truncatedText}`);
        } else {
          block.push(`${relativePath}-${current}- ${truncatedText}`);
        }
      }
      return block;
    };

    const matches: {
      filePath: string;
      lineNumber: number;
      lineText?: string;
    }[] = [];
    rl.on("line", (line) => {
      if (!line.trim() || matchCount >= effectiveLimit) {
        return;
      }
      let event: {
        type?: string;
        data?: {
          path?: { text?: string };
          line_number?: number;
          lines?: { text?: string };
        };
      };
      try {
        event = JSON.parse(line);
      } catch {
        return;
      }
      if (event.type === "match") {
        matchCount++;
        const filePath = event.data?.path?.text;
        const lineNumber = event.data?.line_number;
        const lineText = event.data?.lines?.text;
        if (filePath && typeof lineNumber === "number") {
          matches.push({ filePath, lineNumber, lineText });
        }
        if (matchCount >= effectiveLimit) {
          matchLimitReached = true;
          stopChild(true);
        }
      }
    });

    child.on("error", (error) => {
      cleanup();
      reject(new Error(`Failed to run ripgrep: ${error.message}`));
    });

    child.on("close", (code) => {
      void (async (): Promise<void> => {
        cleanup();
        if (aborted) {
          reject(new Error("Operation aborted"));
          return;
        }
        if (!killedDueToLimit && code !== 0 && code !== 1) {
          reject(new Error(stderr.trim() || `ripgrep exited with code ${code}`));
          return;
        }
        if (matchCount === 0) {
          resolvePromise({
            content: [{ type: "text", text: "No matches found" }],
            details: undefined
          });
          return;
        }

        for (const match of matches) {
          if (contextValue === 0 && match.lineText !== undefined) {
            const relativePath = formatPath(match.filePath);
            const sanitized = match.lineText.replace(/\r\n/g, "\n").replace(/\r/g, "").replace(/\n$/, "");
            const { text: truncatedText, wasTruncated } = truncateLine(sanitized);
            if (wasTruncated) {
              linesTruncated = true;
            }
            outputLines.push(`${relativePath}:${match.lineNumber}: ${truncatedText}`);
          } else {
            const block = await formatBlock(match.filePath, match.lineNumber);
            outputLines.push(...block);
          }
        }

        const rawOutput = outputLines.join("\n");
        const truncation = truncateHead(rawOutput, {
          maxLines: Number.MAX_SAFE_INTEGER
        });
        let output = truncation.content;
        const details: {
          matchLimitReached?: number;
          truncation?: typeof truncation;
          linesTruncated?: boolean;
        } = {};
        const notices: string[] = [];
        if (matchLimitReached) {
          notices.push(
            `${effectiveLimit} matches limit reached. Use limit=${effectiveLimit * 2} for more, or refine pattern`
          );
          details.matchLimitReached = effectiveLimit;
        }
        if (truncation.truncated) {
          notices.push(`${formatSize(DEFAULT_MAX_BYTES)} limit reached`);
          details.truncation = truncation;
        }
        if (linesTruncated) {
          notices.push("Some lines truncated to 500 chars. Use read tool to see full lines");
          details.linesTruncated = true;
        }
        if (notices.length > 0) {
          output += `\n\n[${notices.join(". ")}]`;
        }
        resolvePromise({
          content: [{ type: "text", text: output }],
          details: Object.keys(details).length > 0 ? details : undefined
        });
      })();
    });
  });
}

const PATH_DELIMITERS = new Set([" ", "\t", '"', "'", "="]);

export function findLastDelimiter(text: string): number {
  for (let i = text.length - 1; i >= 0; i -= 1) {
    if (PATH_DELIMITERS.has(text[i] ?? "")) {
      return i;
    }
  }
  return -1;
}

export function extractAtPrefix(text: string): string | null {
  const lastDelimiterIndex = findLastDelimiter(text);
  const tokenStart = lastDelimiterIndex === -1 ? 0 : lastDelimiterIndex + 1;
  if (text[tokenStart] === "@") {
    return text.slice(tokenStart);
  }
  return null;
}

export function parsePathPrefix(prefix: string): {
  rawPrefix: string;
  isQuotedPrefix: boolean;
} {
  if (prefix.startsWith('@"')) {
    // Strip leading @" and trailing "
    const inner = prefix.slice(2);
    const rawPrefix = inner.endsWith('"') ? inner.slice(0, -1) : inner;
    return { rawPrefix, isQuotedPrefix: true };
  }
  if (prefix.startsWith("@")) {
    return { rawPrefix: prefix.slice(1), isQuotedPrefix: false };
  }
  return { rawPrefix: prefix, isQuotedPrefix: false };
}

export function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function buildFdPathQuery(query: string): string {
  const normalized = toPosixPath(query);
  if (!normalized.includes("/")) {
    return normalized;
  }
  const hasTrailingSeparator = normalized.endsWith("/");
  const trimmed = normalized.replace(/^\/+|\/+$/g, "");
  if (!trimmed) {
    return normalized;
  }
  const separatorPattern = "[\\\\/]";
  const segments = trimmed.split("/").filter(Boolean).map(escapeRegex);
  if (segments.length === 0) {
    return normalized;
  }
  let pattern = segments.join(separatorPattern);
  if (hasTrailingSeparator) {
    pattern += separatorPattern;
  }
  return pattern;
}

export function buildCompletionValue(path: string, isDirectory: boolean, isQuotedPrefix: boolean): string {
  const displayPath = isDirectory ? `${path}/` : path;
  const needsQuotes = isQuotedPrefix || displayPath.includes(" ");
  if (!needsQuotes) {
    return `@${displayPath}`;
  }
  return `@"${displayPath}"`;
}

export function scoreEntry(path: string, query: string, isDirectory: boolean): number {
  const lowerQuery = query.toLowerCase();
  const lowerFileName = basename(path.replace(/\/$/, "")).toLowerCase();
  const filePath = path.toLowerCase();

  let score = 0;
  if (lowerFileName === lowerQuery) {
    score = 100;
  } else if (lowerFileName.startsWith(lowerQuery)) {
    score = 80;
  } else if (lowerFileName.includes(lowerQuery)) {
    score = 50;
  } else if (filePath.includes(lowerQuery)) {
    score = 30;
  }
  if (isDirectory && score > 0) {
    score += 10;
  }
  return score;
}

function resolveScopedQuery(
  projectRoot: string,
  rawQuery: string
): { baseDir: string; query: string; displayBase: string } | null {
  const normalizedQuery = toPosixPath(rawQuery);
  const slashIndex = normalizedQuery.lastIndexOf("/");
  if (slashIndex === -1) {
    return null;
  }

  const displayBase = normalizedQuery.slice(0, slashIndex + 1);
  const query = normalizedQuery.slice(slashIndex + 1);
  const baseDir = join(projectRoot, displayBase);

  try {
    if (!statSync(baseDir).isDirectory()) {
      return null;
    }
  } catch {
    return null;
  }

  return { baseDir, query, displayBase };
}

async function walkForAutocomplete(
  baseDir: string,
  fdPath: string,
  query: string,
  isVisible: PathVisibilityChecker,
  projectRoot: string,
  signal: AbortSignal,
  maxResults = 100
): Promise<{ path: string; isDirectory: boolean }[]> {
  const args = [
    "--base-directory",
    baseDir,
    "--max-results",
    String(maxResults),
    "--type",
    "f",
    "--type",
    "d",
    "--follow",
    "--hidden",
    "--no-ignore-vcs",
    "--no-require-git",
    "--exclude",
    ".git",
    "--exclude",
    ".git/*",
    "--exclude",
    ".git/**"
  ];
  if (toPosixPath(query).includes("/")) {
    args.push("--full-path");
  }
  if (query) {
    args.push(buildFdPathQuery(query));
  }
  args.push(".");

  let lines: string[];
  try {
    lines = await runFdPaths(fdPath, baseDir, args, signal);
  } catch {
    return [];
  }

  const results: { path: string; isDirectory: boolean }[] = [];
  for (const rawLine of lines) {
    const displayLine = toPosixPath(rawLine.replace(/\r$/, "").trim().replace(/^\.\//, ""));
    if (!displayLine || displayLine === ".git" || displayLine.startsWith(".git/")) {
      continue;
    }
    const isDirectory = displayLine.endsWith("/");
    const normalizedPath = isDirectory ? displayLine.slice(0, -1) : displayLine;
    const absolutePath = resolve(baseDir, normalizedPath);
    const relToProject = toPosixPath(relative(projectRoot, absolutePath));
    if (!relToProject || relToProject.startsWith("..") || !isVisible(relToProject)) {
      continue;
    }
    results.push({ path: displayLine, isDirectory });
  }
  return results;
}

async function getAtSuggestions(
  projectRoot: string,
  atPrefix: string,
  piignoreFiles: string[],
  isVisible: PathVisibilityChecker,
  signal: AbortSignal
): Promise<AutocompleteItem[]> {
  const fdPath = resolveToolBinary("fd");
  if (!fdPath) {
    return [];
  }

  const { rawPrefix, isQuotedPrefix } = parsePathPrefix(atPrefix);
  const scoped = resolveScopedQuery(projectRoot, rawPrefix);
  const fdBaseDir = scoped?.baseDir ?? projectRoot;
  const fdQuery = scoped?.query ?? rawPrefix;

  const entries = await walkForAutocomplete(fdBaseDir, fdPath, fdQuery, isVisible, projectRoot, signal);

  const extraPaths = await collectPiignoredPaths(
    projectRoot,
    scoped ? fdBaseDir : projectRoot,
    piignoreFiles,
    isVisible,
    signal,
    100
  );

  for (const extra of extraPaths) {
    const displayPath = scoped ? `${scoped.displayBase}${extra}` : extra;
    if (!entries.some((entry) => toPosixPath(entry.path.replace(/\/$/, "")) === toPosixPath(displayPath))) {
      entries.push({ path: displayPath, isDirectory: false });
    }
  }

  const scoredEntries = entries
    .map((entry) => ({
      ...entry,
      score: fdQuery ? scoreEntry(entry.path, fdQuery, entry.isDirectory) : 1
    }))
    .filter((entry) => entry.score > 0);

  scoredEntries.sort((a, b) => b.score - a.score);

  const suggestions: AutocompleteItem[] = [];
  for (const { path: entryPath, isDirectory } of scoredEntries.slice(0, 20)) {
    const pathWithoutSlash = isDirectory ? entryPath.slice(0, -1) : entryPath;
    const displayPath = scoped
      ? `${toPosixPath(scoped.displayBase)}${toPosixPath(pathWithoutSlash)}`
      : toPosixPath(pathWithoutSlash);
    const entryName = basename(pathWithoutSlash);
    const completionPath = isDirectory ? `${displayPath}/` : displayPath;
    suggestions.push({
      value: buildCompletionValue(completionPath, isDirectory, isQuotedPrefix),
      label: entryName + (isDirectory ? "/" : ""),
      description: displayPath
    });
  }

  return suggestions;
}

function createPiignoreAutocompleteProvider(
  current: AutocompleteProvider,
  projectRoot: string,
  piignoreFiles: string[],
  isVisible: PathVisibilityChecker
): AutocompleteProvider {
  return {
    async getSuggestions(
      lines: string[],
      cursorLine: number,
      cursorCol: number,
      options
    ): Promise<AutocompleteSuggestions | null> {
      const currentLine = lines[cursorLine] ?? "";
      const textBeforeCursor = currentLine.slice(0, cursorCol);
      const atPrefix = extractAtPrefix(textBeforeCursor);
      if (!atPrefix) {
        return current.getSuggestions(lines, cursorLine, cursorCol, options);
      }

      const suggestions = await getAtSuggestions(projectRoot, atPrefix, piignoreFiles, isVisible, options.signal);
      if (options.signal.aborted) {
        return null;
      }
      if (suggestions.length === 0) {
        return current.getSuggestions(lines, cursorLine, cursorCol, options);
      }

      return {
        items: suggestions,
        prefix: atPrefix
      };
    },

    applyCompletion(lines, cursorLine, cursorCol, item, prefix) {
      return current.applyCompletion(lines, cursorLine, cursorCol, item, prefix);
    },

    shouldTriggerFileCompletion(lines, cursorLine, cursorCol) {
      return current.shouldTriggerFileCompletion?.(lines, cursorLine, cursorCol) ?? true;
    }
  };
}

export default function (pi: ExtensionAPI): void {
  pi.on("session_start", async (_event, ctx) => {
    const piignorePath = join(ctx.cwd, PIIGNORE);
    if (!existsSync(piignorePath)) {
      return;
    }

    const piignoreFiles = discoverPiignoreFiles(ctx.cwd, ctx.cwd);
    const isVisible = createPathVisibilityChecker(ctx.cwd, piignoreFiles);

    ctx.ui.notify("`.piignore` active: find/grep/@ autocomplete include negated gitignored paths", "info");
    ctx.ui.addAutocompleteProvider((current) =>
      createPiignoreAutocompleteProvider(current, ctx.cwd, piignoreFiles, isVisible)
    );
  });

  const findDef = createFindToolDefinition(process.cwd());
  pi.registerTool({
    ...findDef,
    description:
      "Search for files by glob pattern. Returns matching file paths relative to the search directory. Respects .gitignore unless overridden by .piignore. Output is truncated to 1000 results or 50KB (whichever is hit first).",
    promptSnippet: "Find files by glob pattern (respects .gitignore; .piignore can un-ignore paths)",

    async execute(toolCallId, params, signal, onUpdate, ctx) {
      const searchPath = resolveSearchPath(params.path, ctx.cwd);
      const piignoreFiles = discoverPiignoreFiles(searchPath, ctx.cwd);
      if (piignoreFiles.length === 0) {
        return getBuiltinFind(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
      }

      const tool = createFindTool(ctx.cwd, {
        operations: createPiignoreFindOperations(ctx.cwd, piignoreFiles)
      });
      return tool.execute(toolCallId, params, signal, onUpdate);
    }
  });

  const grepDef = createGrepToolDefinition(process.cwd());
  pi.registerTool({
    ...grepDef,
    description:
      "Search file contents for a pattern. Returns matching lines with file paths and line numbers. Respects .gitignore unless overridden by .piignore. Output is truncated to 100 matches or 50KB (whichever is hit first).",
    promptSnippet: "Search file contents for patterns (.piignore can un-ignore gitignored paths)",

    async execute(toolCallId, params, signal, onUpdate, ctx) {
      const searchPath = resolveSearchPath(params.path, ctx.cwd);
      const piignoreFiles = discoverPiignoreFiles(searchPath, ctx.cwd);
      if (piignoreFiles.length === 0) {
        return getBuiltinGrep(ctx.cwd).execute(toolCallId, params, signal, onUpdate);
      }
      return executePiignoreGrep(ctx.cwd, params, piignoreFiles, signal);
    }
  });
}
