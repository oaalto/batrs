import fs from "node:fs";
import path from "node:path";

import { extractMarkdownLinks, resolveWikiHref } from "./links.mjs";

/**
 * @param {string} rootDir
 * @param {string} file
 * @param {string[]} errors
 */
function validateFileWikiLinks(rootDir, file, errors) {
  const content = fs.readFileSync(file, "utf8");
  const relFrom = path.relative(rootDir, file).replace(/\\/g, "/");
  for (const link of extractMarkdownLinks(content)) {
    const resolved = resolveWikiHref(rootDir, link.href, file);
    if (!resolved || fs.existsSync(resolved)) {
      continue;
    }
    errors.push(`${relFrom}: broken wiki link "${link.href}"`);
  }
}

/**
 * @param {string} rootDir
 * @param {string[]} files
 * @param {string[]} errors
 */
export function validateWikiLinks(rootDir, files, errors) {
  for (const file of files) {
    validateFileWikiLinks(rootDir, file, errors);
  }
}
