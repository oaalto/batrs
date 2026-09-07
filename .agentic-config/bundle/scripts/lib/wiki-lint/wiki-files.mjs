import fs from "node:fs";
import path from "node:path";

import { EXEMPT_BASENAMES } from "./constants.mjs";

/**
 * @param {import("node:fs").Dirent} entry
 */
function isWikiPageFile(entry) {
  return entry.isFile() && entry.name.endsWith(".md") && !EXEMPT_BASENAMES.has(entry.name);
}

/**
 * @param {string} dir
 * @param {string[]} files
 */
function collectMarkdownFiles(dir, files) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      collectMarkdownFiles(full, files);
      continue;
    }
    if (isWikiPageFile(entry)) {
      files.push(full);
    }
  }
}

/**
 * @param {string} rootDir
 */
export function listWikiPageFiles(rootDir) {
  const wikiRoot = path.join(rootDir, "docs/wiki");
  if (!fs.existsSync(wikiRoot)) {
    return [];
  }
  /** @type {string[]} */
  const files = [];
  collectMarkdownFiles(wikiRoot, files);
  return files;
}

/**
 * @param {string} rootDir
 * @param {string[]} errors
 */
export function readPathMap(rootDir, errors) {
  const mapPath = path.join(rootDir, "docs/wiki/path-map.json");
  if (!fs.existsSync(mapPath)) {
    return { mappings: [] };
  }
  try {
    return JSON.parse(fs.readFileSync(mapPath, "utf8"));
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    errors.push(`docs/wiki/path-map.json: invalid JSON (${message})`);
    return { mappings: [] };
  }
}
