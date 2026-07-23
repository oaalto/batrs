#!/usr/bin/env node
/**
 * Mechanical wiki lint — reference implementation.
 *
 * Usage:
 *   node scripts/wiki-lint.mjs [--staged]
 *
 * Exits 1 on errors; prints warnings to stderr without failing (v1 obligation backstop).
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runWikiLint } from "./lib/wiki-lint/run.mjs";

/**
 * @param {{ warnings: string[], errors: string[], ok: boolean }} result
 */
function printLintResult(result) {
  for (const warning of result.warnings) {
    console.error(`warning: ${warning}`);
  }
  for (const error of result.errors) {
    console.error(`error: ${error}`);
  }
}

function main() {
  const staged = process.argv.includes("--staged");
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = fs.existsSync(path.join(scriptDir, "docs/wiki")) ? scriptDir : path.resolve(scriptDir, "..");
  const result = runWikiLint({ rootDir, staged });
  printLintResult(result);
  if (!result.ok) {
    process.exit(1);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main();
}
