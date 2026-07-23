import fs from "node:fs";
import path from "node:path";

import { extractMarkdownLinks, resolveWikiHref } from "./links.mjs";

/**
 * @param {string} rootDir
 * @param {string} indexPath
 */
function linkedWikiPaths(rootDir, indexPath) {
  const indexContent = fs.readFileSync(indexPath, "utf8");
  const linkedPaths = new Set();
  for (const link of extractMarkdownLinks(indexContent)) {
    const resolved = resolveWikiHref(rootDir, link.href, indexPath);
    if (resolved) {
      linkedPaths.add(resolved);
    }
  }
  return linkedPaths;
}

/**
 * @param {string} rootDir
 * @param {string[]} pageFiles
 * @param {Set<string>} linkedPaths
 * @param {string[]} errors
 */
function reportOrphanPages(rootDir, pageFiles, linkedPaths, errors) {
  for (const pageFile of pageFiles) {
    if (linkedPaths.has(pageFile)) {
      continue;
    }
    const rel = path.relative(rootDir, pageFile).replace(/\\/g, "/");
    errors.push(`${rel}: orphan page — not linked from docs/wiki/index.md`);
  }
}

/**
 * @param {string} rootDir
 * @param {string[]} pageFiles
 * @param {string} indexPath
 * @param {string[]} errors
 */
export function validateIndexCoverage(rootDir, pageFiles, indexPath, errors) {
  if (!fs.existsSync(indexPath)) {
    errors.push("docs/wiki/index.md: missing index file");
    return;
  }
  const linkedPaths = linkedWikiPaths(rootDir, indexPath);
  reportOrphanPages(rootDir, pageFiles, linkedPaths, errors);
}
