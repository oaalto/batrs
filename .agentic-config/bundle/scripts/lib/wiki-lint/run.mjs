import fs from "node:fs";
import path from "node:path";

import { checkStagedObligation } from "./staged-obligation.mjs";
import { validateIndexCoverage } from "./validate-index.mjs";
import { validateWikiLinks } from "./validate-links.mjs";
import { validateLogEntries } from "./validate-log.mjs";
import { validateWikiPage } from "./validate-page.mjs";
import { listWikiPageFiles, readPathMap } from "./wiki-files.mjs";

/**
 * @param {string} rootDir
 * @param {string[]} pageFiles
 * @param {string[]} errors
 */
function validateAllPages(rootDir, pageFiles, errors) {
  for (const pageFile of pageFiles) {
    const rel = path.relative(rootDir, pageFile).replace(/\\/g, "/");
    const content = fs.readFileSync(pageFile, "utf8");
    validateWikiPage(rel, content, errors);
  }
}

/**
 * @param {object} options
 * @param {string} [options.rootDir]
 * @param {boolean} [options.staged]
 * @param {string[]} [options.stagedFiles]
 * @returns {{ errors: string[], warnings: string[], ok: boolean }}
 */
export function runWikiLint({ rootDir = process.cwd(), staged = false, stagedFiles = null } = {}) {
  /** @type {string[]} */
  const errors = [];
  /** @type {string[]} */
  const warnings = [];

  const wikiRoot = path.join(rootDir, "docs/wiki");
  if (!fs.existsSync(wikiRoot)) {
    return { errors: [], warnings: [], ok: true };
  }

  const pageFiles = listWikiPageFiles(rootDir);
  const indexPath = path.join(wikiRoot, "index.md");
  const logPath = path.join(wikiRoot, "log.md");

  validateAllPages(rootDir, pageFiles, errors);
  validateIndexCoverage(rootDir, pageFiles, indexPath, errors);

  const allWikiMarkdown = [indexPath, logPath, ...pageFiles].filter((file) => fs.existsSync(file));
  validateWikiLinks(rootDir, allWikiMarkdown, errors);
  validateLogEntries(logPath, errors);
  readPathMap(rootDir, errors);
  checkStagedObligation(rootDir, staged, stagedFiles, warnings);

  return { errors, warnings, ok: errors.length === 0 };
}
