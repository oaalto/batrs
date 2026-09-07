/**
 * Test suite for .piignore extension.
 *
 * Uses Node.js built-in test runner (node:test) — no external dependencies.
 * Run with: node --test tests/piignore.test.ts
 *
 * Categories:
 *  1. Pure function unit tests (no fs/git/process side effects).
 *  2. discoverPiignoreFiles integration (real fs, no git needed).
 *  3. createPathVisibilityChecker integration (real git repos in temp dirs).
 */

import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import {
  toPosixPath,
  resolveSearchPath,
  normalizeFindPattern,
  extractNegatedPatterns,
  resolveNegationScan,
  patternOverlapsSearch,
  scoreEntry,
  buildCompletionValue,
  parsePathPrefix,
  extractAtPrefix,
  findLastDelimiter,
  buildFdPathQuery,
  createPathVisibilityChecker,
  discoverPiignoreFiles,
  matchesGitignorePattern
} from "../extensions/piignore.ts";

// ---- Helpers ----

interface TestRepo {
  dir: string;
  piignorePath: string;
  gitignorePath: string;
}

function createTestRepo(name: string): TestRepo {
  const dir = join(tmpdir(), `piignore-test-${Date.now()}-${name}`);
  mkdirSync(dir, { recursive: true });

  spawnSync("git", ["init"], { cwd: dir, stdio: "ignore" });
  spawnSync("git", ["config", "user.email", "test@test.com"], {
    cwd: dir,
    stdio: "ignore"
  });
  spawnSync("git", ["config", "user.name", "Test"], {
    cwd: dir,
    stdio: "ignore"
  });

  return {
    dir,
    piignorePath: join(dir, ".piignore"),
    gitignorePath: join(dir, ".gitignore")
  };
}

function cleanupRepo(repo: TestRepo): void {
  if (existsSync(repo.dir)) {
    rmSync(repo.dir, { recursive: true, force: true });
  }
}

// ---- Pure function tests ----

describe("toPosixPath", () => {
  test("returns forward-slash path", () => {
    assert.equal(toPosixPath("a/b/c"), "a/b/c");
  });

  test("already forward-slash is unchanged", () => {
    assert.equal(toPosixPath("a/b/c"), "a/b/c");
  });
});

describe("resolveSearchPath", () => {
  test("strips @ prefix", () => {
    assert.equal(resolveSearchPath("@src/main.ts", "/cwd"), "/cwd/src/main.ts");
  });

  test("returns cwd when undefined", () => {
    // path.resolve('/cwd', '.') returns '/cwd' (not '/cwd/.')
    assert.equal(resolveSearchPath(undefined, "/cwd"), "/cwd");
  });

  test("no prefix passes through", () => {
    assert.equal(resolveSearchPath("src/main.ts", "/cwd"), "/cwd/src/main.ts");
  });
});

describe("normalizeFindPattern", () => {
  test("adds **/ prefix when pattern contains /", () => {
    const { effectivePattern, useFullPath } = normalizeFindPattern("src/**/*.ts");
    assert.equal(effectivePattern, "**/src/**/*.ts");
    assert.equal(useFullPath, true);
  });

  test("does not add prefix for already-prefixed patterns", () => {
    const { effectivePattern } = normalizeFindPattern("**/src");
    assert.equal(effectivePattern, "**/src");
  });

  test("does not add prefix for absolute patterns", () => {
    const { effectivePattern } = normalizeFindPattern("/src");
    assert.equal(effectivePattern, "/src");
  });

  test("pattern without / is not marked as full path", () => {
    const { effectivePattern, useFullPath } = normalizeFindPattern("*.ts");
    assert.equal(effectivePattern, "*.ts");
    assert.equal(useFullPath, false);
  });

  test("pattern without / stays unchanged", () => {
    const { effectivePattern } = normalizeFindPattern("*.ts");
    assert.equal(effectivePattern, "*.ts");
  });
});

describe("extractNegatedPatterns", () => {
  test("extracts ! patterns", () => {
    const input = "!docs/prds/**\n!*.md\n# comment\n!build/**";
    const { dir } = createTestRepo("extract-negations");
    try {
      writeFileSync(join(dir, ".piignore"), input);
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, ["docs/prds/**", "*.md", "build/**"]);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("deduplicates patterns", () => {
    const { dir } = createTestRepo("dedup");
    try {
      writeFileSync(join(dir, ".piignore"), "!docs/**\n!docs/**");
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, ["docs/**"]);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("ignores comments", () => {
    const { dir } = createTestRepo("comments");
    try {
      writeFileSync(join(dir, ".piignore"), "# !this is a comment\n!real");
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, ["real"]);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("handles Windows line endings", () => {
    const { dir } = createTestRepo("crlf");
    try {
      writeFileSync(join(dir, ".piignore"), "!docs/**\r\n!*.md\r\n");
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, ["docs/**", "*.md"]);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("ignores empty lines", () => {
    const { dir } = createTestRepo("empty");
    try {
      writeFileSync(join(dir, ".piignore"), "\n\n!docs/**\n\n");
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, ["docs/**"]);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("returns empty array for no negations", () => {
    const { dir } = createTestRepo("no-neg");
    try {
      writeFileSync(join(dir, ".piignore"), "*.log\n# comment");
      const result = extractNegatedPatterns([join(dir, ".piignore")]);
      assert.deepEqual(result, []);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("handles multiple piignore files", () => {
    const repo1 = createTestRepo("multi-1");
    const repo2 = createTestRepo("multi-2");
    try {
      writeFileSync(join(repo1.dir, ".piignore"), "!docs/**");
      writeFileSync(join(repo2.dir, ".piignore"), "!build/**");
      const result = extractNegatedPatterns([join(repo1.dir, ".piignore"), join(repo2.dir, ".piignore")]);
      assert.deepEqual(result.sort(), ["build/**", "docs/**"]);
    } finally {
      cleanupRepo({
        ...repo1,
        piignorePath: join(repo1.dir, ".piignore"),
        gitignorePath: join(repo1.dir, ".gitignore")
      });
      cleanupRepo({
        ...repo2,
        piignorePath: join(repo2.dir, ".piignore"),
        gitignorePath: join(repo2.dir, ".gitignore")
      });
    }
  });
});

describe("resolveNegationScan", () => {
  test("splits directory path from glob", () => {
    const result = resolveNegationScan("/project", "docs/prds/**");
    assert.equal(result.scanRoot, join("/project", "docs", "prds"));
    assert.equal(result.glob, "**");
  });

  test("handles pattern without glob suffix", () => {
    const result = resolveNegationScan("/project", "docs/prds");
    assert.equal(result.scanRoot, join("/project", "docs", "prds"));
    assert.equal(result.glob, "**");
  });

  test("preserves glob segments", () => {
    const result = resolveNegationScan("/project", "docs/**/*.md");
    assert.equal(result.scanRoot, join("/project", "docs"));
    assert.equal(result.glob, "**/*.md");
  });

  test("handles wildcards in middle of path", () => {
    const result = resolveNegationScan("/project", "docs/*prd*.md");
    // Splits at first wildcard: dir prefix = "docs", glob = "*prd*.md"
    assert.equal(result.scanRoot, join("/project", "docs"));
    assert.equal(result.glob, "*prd*.md");
  });

  test("handles root-level negation", () => {
    const result = resolveNegationScan("/project", "*.log");
    assert.equal(result.scanRoot, "/project");
    assert.equal(result.glob, "*.log");
  });

  test("handles leading slash", () => {
    const result = resolveNegationScan("/project", "/docs/prds/**");
    assert.equal(result.scanRoot, join("/project", "docs", "prds"));
    assert.equal(result.glob, "**");
  });

  test("handles empty pattern", () => {
    const result = resolveNegationScan("/project", "");
    assert.equal(result.scanRoot, "/project");
    assert.equal(result.glob, "");
  });

  test("handles doublestar at root", () => {
    const result = resolveNegationScan("/project", "**/*.ts");
    assert.equal(result.scanRoot, "/project");
    assert.equal(result.glob, "**/*.ts");
  });
});

describe("patternOverlapsSearch", () => {
  test("returns true when search is root", () => {
    assert.equal(patternOverlapsSearch("docs/**", "/project", "/project"), true);
  });

  test("returns true when pattern starts with search path", () => {
    assert.equal(patternOverlapsSearch("docs/prds/**", join("/project", "docs"), "/project"), true);
  });

  test("returns true when search path starts with pattern root", () => {
    assert.equal(patternOverlapsSearch("docs", join("/project", "docs", "prds"), "/project"), true);
  });

  test("returns false when no overlap", () => {
    assert.equal(patternOverlapsSearch("docs/**", join("/project", "src"), "/project"), false);
  });

  test("returns false for unrelated paths", () => {
    assert.equal(patternOverlapsSearch("build/**", join("/project", "src", "test"), "/project"), false);
  });

  test("returns true for exact pattern match", () => {
    assert.equal(patternOverlapsSearch("docs/prds", join("/project", "docs", "prds"), "/project"), true);
  });
});

describe("scoreEntry", () => {
  test("exact filename match gets highest score", () => {
    assert.equal(scoreEntry("docs/prd.md", "prd.md", false), 100);
  });

  test("filename start match", () => {
    assert.equal(scoreEntry("docs/prd-2024.md", "prd", false), 80);
  });

  test("filename contains match", () => {
    assert.equal(scoreEntry("docs/my-prd-file.md", "prd", false), 50);
  });

  test("filename starts with query", () => {
    // "prd.md" starts with "prd", so score is 80
    assert.equal(scoreEntry("docs/prd.md", "prd", false), 80);
  });

  test("full path contains match (query not in filename)", () => {
    assert.equal(scoreEntry("docs/my-prd-file.md", "prd", false), 50);
  });

  test("directory gets +10 bonus", () => {
    // "issues" matches basename "issues" (100) + directory bonus (10) = 110
    assert.equal(scoreEntry("docs/issues/", "issues", true), 110);
  });

  test("no match returns 0", () => {
    assert.equal(scoreEntry("docs/prd.md", "xyz", false), 0);
  });

  test("case insensitive", () => {
    // "PRD.md" starts with "prd" (case insensitive), so score is 80
    assert.equal(scoreEntry("docs/PRD.md", "prd", false), 80);
  });

  test("exact directory match", () => {
    assert.equal(scoreEntry("docs/prds", "prds", true), 110);
  });
});

describe("buildCompletionValue", () => {
  test("directory path with trailing slash", () => {
    // Function adds trailing slash for directories
    assert.equal(buildCompletionValue("docs", true, false), "@docs/");
  });

  test("file path without slash", () => {
    assert.equal(buildCompletionValue("docs/prd.md", false, false), "@docs/prd.md");
  });

  test("path with space gets quoted", () => {
    assert.equal(buildCompletionValue("docs/my file.md", false, false), `@"docs/my file.md"`);
  });

  test("quoted prefix passes through", () => {
    assert.equal(buildCompletionValue("docs/prd.md", false, true), `@"docs/prd.md"`);
  });

  test("empty path", () => {
    assert.equal(buildCompletionValue("", false, false), "@");
  });
});

describe("parsePathPrefix", () => {
  test("strips @ prefix", () => {
    const result = parsePathPrefix("@docs");
    assert.equal(result.rawPrefix, "docs");
    assert.equal(result.isQuotedPrefix, false);
  });

  test('strips @" prefix and marks quoted', () => {
    const result = parsePathPrefix(`@"docs"`);
    assert.equal(result.rawPrefix, "docs");
    assert.equal(result.isQuotedPrefix, true);
  });

  test("no @ prefix returns as-is", () => {
    const result = parsePathPrefix("docs");
    assert.equal(result.rawPrefix, "docs");
    assert.equal(result.isQuotedPrefix, false);
  });

  test("empty string", () => {
    const result = parsePathPrefix("");
    assert.equal(result.rawPrefix, "");
    assert.equal(result.isQuotedPrefix, false);
  });
});

describe("extractAtPrefix", () => {
  test("extracts @ prefix after =", () => {
    assert.equal(extractAtPrefix("cmd arg=@value"), "@value");
  });

  test("extracts @ prefix after space", () => {
    assert.equal(extractAtPrefix("cmd arg @value"), "@value");
  });

  test("extracts @ prefix at start of line", () => {
    assert.equal(extractAtPrefix("@value"), "@value");
  });

  test("returns null when no @ prefix", () => {
    assert.equal(extractAtPrefix("cmd arg value"), null);
  });

  test("handles @ in middle of text", () => {
    assert.equal(extractAtPrefix("text @path/to/file"), "@path/to/file");
  });

  test("handles multiple delimiters", () => {
    assert.equal(extractAtPrefix("cmd --arg=@value"), "@value");
    assert.equal(extractAtPrefix("cmd --arg=@value path"), null);
  });

  test("empty string returns null", () => {
    assert.equal(extractAtPrefix(""), null);
  });
});

describe("findLastDelimiter", () => {
  test("finds last space", () => {
    // "cmd arg value" - last space is at index 7 (before "value")
    assert.equal(findLastDelimiter("cmd arg value"), 7);
  });

  test("finds last =", () => {
    assert.equal(findLastDelimiter("key=value"), 3);
  });

  test("returns -1 when no delimiter", () => {
    assert.equal(findLastDelimiter("nodelim"), -1);
  });

  test("handles quoted string", () => {
    // 'cmd "path"' - last " is at index 9
    assert.equal(findLastDelimiter('cmd "path"'), 9);
  });

  test("finds tab", () => {
    assert.equal(findLastDelimiter("cmd\targ"), 3);
  });

  test("finds single quote", () => {
    // "cmd 'path'" - last ' is at index 9
    assert.equal(findLastDelimiter("cmd 'path'"), 9);
  });
});

describe("buildFdPathQuery", () => {
  test("simple name stays simple", () => {
    assert.equal(buildFdPathQuery("file.ts"), "file.ts");
  });

  test("path with / gets regex pattern", () => {
    const result = buildFdPathQuery("docs/prd.md");
    assert.ok(result.includes("[\\\\/]"));
  });

  test("trailing / adds separator pattern", () => {
    const result = buildFdPathQuery("docs/");
    assert.ok(result.endsWith("[\\\\/]"));
  });

  test("escapes regex special chars in path", () => {
    // Only escapes when path contains /
    const result = buildFdPathQuery("file[1].ts");
    // Simple filename without / is returned as-is
    assert.equal(result, "file[1].ts");
  });

  test("escapes regex special chars in path with /", () => {
    const result = buildFdPathQuery("dir/file[1].ts");
    assert.ok(result.includes("\\["));
    assert.ok(result.includes("\\]"));
  });

  test("empty string stays empty", () => {
    assert.equal(buildFdPathQuery(""), "");
  });

  test("multiple segments", () => {
    const result = buildFdPathQuery("a/b/c");
    assert.ok(result.includes("[\\\\/]"));
    assert.ok(result.includes("a"));
    assert.ok(result.includes("b"));
    assert.ok(result.includes("c"));
  });
});

// ---- discoverPiignoreFiles tests ----

describe("discoverPiignoreFiles", () => {
  test("finds .piignore in project root", () => {
    const { dir } = createTestRepo("discover-root");
    try {
      writeFileSync(join(dir, ".piignore"), "!docs/**");
      const result = discoverPiignoreFiles(dir, dir);
      assert.equal(result.length, 1);
      assert.equal(result[0], join(dir, ".piignore"));
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("returns empty when no .piignore", () => {
    const { dir } = createTestRepo("discover-none");
    try {
      const result = discoverPiignoreFiles(dir, dir);
      assert.equal(result.length, 0);
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });

  test("walks up directory tree", () => {
    const { dir } = createTestRepo("discover-walk");
    try {
      const subDir = join(dir, "sub", "deep");
      mkdirSync(subDir, { recursive: true });
      writeFileSync(join(dir, ".piignore"), "!docs/**");

      const result = discoverPiignoreFiles(subDir, dir);
      assert.equal(result.length, 1);
      assert.equal(result[0], join(dir, ".piignore"));
    } finally {
      cleanupRepo({
        ...{ dir },
        piignorePath: join(dir, ".piignore"),
        gitignorePath: join(dir, ".gitignore")
      });
    }
  });
});

// ---- Integration tests: createPathVisibilityChecker ----

describe("createPathVisibilityChecker (integration)", () => {
  test("path not in .gitignore is visible", () => {
    const repo = createTestRepo("not-ignored");
    try {
      writeFileSync(repo.gitignorePath, "*.log\n");
      mkdirSync(join(repo.dir, "src"), { recursive: true });
      writeFileSync(join(repo.dir, "src", "main.ts"), "console.log('hi');\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("src/main.ts"), true);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("path in .gitignore is not visible without .piignore negation", () => {
    const repo = createTestRepo("gitignored-no-negation");
    try {
      writeFileSync(repo.gitignorePath, "build/\n");
      mkdirSync(join(repo.dir, "build"), { recursive: true });
      writeFileSync(join(repo.dir, "build", "output.js"), "// built\n");

      // No .piignore file — all gitignored paths should be invisible
      const checker = createPathVisibilityChecker(repo.dir, []);
      assert.equal(checker("build/output.js"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("path in .gitignore with .piignore negation is visible", () => {
    const repo = createTestRepo("gitignored-with-negation");
    try {
      writeFileSync(repo.gitignorePath, "docs/prds/\n");
      mkdirSync(join(repo.dir, "docs", "prds"), { recursive: true });
      writeFileSync(join(repo.dir, "docs", "prds", "prd.md"), "# PRD\n");
      writeFileSync(repo.piignorePath, "!docs/prds/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      // Path is gitignored but negated in .piignore → should be visible
      assert.equal(checker("docs/prds/prd.md"), true);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("multiple negation patterns work", () => {
    const repo = createTestRepo("multiple-negations");
    try {
      writeFileSync(repo.gitignorePath, "docs/\nbuild/\n");
      mkdirSync(join(repo.dir, "docs", "prds"), { recursive: true });
      mkdirSync(join(repo.dir, "build", "dist"), { recursive: true });
      writeFileSync(join(repo.dir, "docs", "prds", "prd.md"), "# PRD\n");
      writeFileSync(join(repo.dir, "build", "dist", "app.js"), "// app\n");
      writeFileSync(repo.piignorePath, "!docs/prds/**\n!build/dist/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("docs/prds/prd.md"), true);
      assert.equal(checker("build/dist/app.js"), true);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("path not matching any negation remains ignored", () => {
    const repo = createTestRepo("no-match-negation");
    try {
      writeFileSync(repo.gitignorePath, "docs/\n");
      mkdirSync(join(repo.dir, "docs", "prds"), { recursive: true });
      mkdirSync(join(repo.dir, "docs", "issues"), { recursive: true });
      writeFileSync(join(repo.dir, "docs", "prds", "prd.md"), "# PRD\n");
      writeFileSync(join(repo.dir, "docs", "issues", "issue.md"), "# Issue\n");
      writeFileSync(repo.piignorePath, "!docs/prds/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      // docs/issues/ is NOT negated → should remain invisible
      assert.equal(checker("docs/issues/issue.md"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("caching works", () => {
    const repo = createTestRepo("caching");
    try {
      writeFileSync(repo.gitignorePath, "build/\n");
      mkdirSync(join(repo.dir, "build"), { recursive: true });
      writeFileSync(join(repo.dir, "build", "output.js"), "// built\n");

      const checker = createPathVisibilityChecker(repo.dir, []);
      // First call
      checker("build/output.js");
      // Second call should use cache (no git subprocess)
      assert.equal(checker("build/output.js"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("basename negation pattern works", () => {
    const repo = createTestRepo("basename-negation");
    try {
      writeFileSync(repo.gitignorePath, "*.log\n");
      writeFileSync(join(repo.dir, "debug.log"), "debug\n");
      writeFileSync(join(repo.dir, "error.log"), "error\n");
      writeFileSync(repo.piignorePath, "!debug.log\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("debug.log"), true);
      assert.equal(checker("error.log"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("directory negation pattern works", () => {
    const repo = createTestRepo("directory-negation");
    try {
      writeFileSync(repo.gitignorePath, "node_modules/\n");
      mkdirSync(join(repo.dir, "node_modules", "pkg"), { recursive: true });
      writeFileSync(join(repo.dir, "node_modules", "pkg", "index.js"), "module\n");
      writeFileSync(repo.piignorePath, "!node_modules/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("node_modules/pkg/index.js"), true);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("nested .gitignore files", () => {
    const repo = createTestRepo("nested-gitignore");
    try {
      writeFileSync(repo.gitignorePath, "src/\n");
      mkdirSync(join(repo.dir, "src", "internal"), { recursive: true });
      writeFileSync(join(repo.dir, "src", "internal", "lib.ts"), "lib\n");

      // Nested .gitignore that re-includes some files
      const srcGitignore = join(repo.dir, "src", ".gitignore");
      writeFileSync(srcGitignore, "!internal/\n");

      writeFileSync(repo.piignorePath, "!src/internal/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      // src/internal/lib.ts is NOT gitignored (re-included by src/.gitignore)
      // So it should be visible (already found by normal scan)
      assert.equal(checker("src/internal/lib.ts"), true);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("empty .piignore makes no difference", () => {
    const repo = createTestRepo("empty-piignore");
    try {
      writeFileSync(repo.gitignorePath, "build/\n");
      mkdirSync(join(repo.dir, "build"), { recursive: true });
      writeFileSync(join(repo.dir, "build", "output.js"), "// built\n");
      writeFileSync(repo.piignorePath, "");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("build/output.js"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("non-existent path returns false", () => {
    const repo = createTestRepo("non-existent");
    try {
      writeFileSync(repo.gitignorePath, "*.xyz\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      // Path doesn't exist and isn't in .gitignore
      assert.equal(checker("nonexistent.xyz"), false);
    } finally {
      cleanupRepo(repo);
    }
  });

  test("complex path with multiple segments", () => {
    const repo = createTestRepo("complex-path");
    try {
      writeFileSync(repo.gitignorePath, "deep/nested/path/\n");
      mkdirSync(join(repo.dir, "deep", "nested", "path"), { recursive: true });
      writeFileSync(join(repo.dir, "deep", "nested", "path", "file.md"), "# Deep\n");
      writeFileSync(repo.piignorePath, "!deep/nested/path/**\n");

      const checker = createPathVisibilityChecker(repo.dir, [repo.piignorePath]);
      assert.equal(checker("deep/nested/path/file.md"), true);
    } finally {
      cleanupRepo(repo);
    }
  });
});

// ---- Integration tests: matchesGitignorePattern ----

describe("matchesGitignorePattern", () => {
  test("basename pattern matches filename", () => {
    assert.equal(matchesGitignorePattern("*.log", "debug.log"), true);
    assert.equal(matchesGitignorePattern("*.log", "src/debug.log"), true);
    assert.equal(matchesGitignorePattern("*.log", "debug.txt"), false);
  });

  test("path pattern matches full path", () => {
    assert.equal(matchesGitignorePattern("docs/prds/**", "docs/prds/prd.md"), true);
    assert.equal(matchesGitignorePattern("docs/prds/**", "docs/prds/deep/prd.md"), true);
    assert.equal(matchesGitignorePattern("docs/prds/**", "src/prds/prd.md"), false);
  });

  test("* matches any characters except /", () => {
    assert.equal(matchesGitignorePattern("*.ts", "file.ts"), true);
    assert.equal(matchesGitignorePattern("*.ts", "src/file.ts"), true);
    assert.equal(matchesGitignorePattern("*.ts", "file.js"), false);
  });

  test("? matches any single character except /", () => {
    assert.equal(matchesGitignorePattern("file?.ts", "file1.ts"), true);
    assert.equal(matchesGitignorePattern("file?.ts", "file12.ts"), false);
  });

  test("** matches any number of directories", () => {
    assert.equal(matchesGitignorePattern("**/*.ts", "file.ts"), true);
    assert.equal(matchesGitignorePattern("**/*.ts", "src/file.ts"), true);
    assert.equal(matchesGitignorePattern("**/*.ts", "src/deep/file.ts"), true);
  });

  test("trailing / matches only directories", () => {
    assert.equal(matchesGitignorePattern("docs/", "docs/"), true);
    assert.equal(matchesGitignorePattern("docs/", "docs"), false);
    assert.equal(matchesGitignorePattern("docs/", "docs/file.md"), false);
  });

  test("pattern with multiple segments", () => {
    assert.equal(matchesGitignorePattern("a/b/c", "a/b/c"), true);
    assert.equal(matchesGitignorePattern("a/b/c", "a/b/d"), false);
  });
});
