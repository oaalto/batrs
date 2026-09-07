import { execSync } from "node:child_process";

import { LOG_ENTRY_RE } from "./constants.mjs";

/**
 * @param {string} rootDir
 */
export function getStagedFiles(rootDir) {
  try {
    const out = execSync("git diff --cached --name-only --diff-filter=ACMR", {
      cwd: rootDir,
      encoding: "utf8"
    });
    return out
      .trim()
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

/**
 * @param {string} rootDir
 * @param {string[]} stagedFiles
 */
export function stagedWikiTouchPresent(rootDir, stagedFiles) {
  if (stagedFiles.some((file) => file.startsWith("docs/wiki/"))) {
    return true;
  }
  if (!stagedFiles.includes("docs/wiki/log.md")) {
    return false;
  }
  try {
    const diff = execSync("git diff --cached -- docs/wiki/log.md", {
      cwd: rootDir,
      encoding: "utf8"
    });
    return LOG_ENTRY_RE.test(diff);
  } catch {
    return false;
  }
}
