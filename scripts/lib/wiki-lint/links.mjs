import path from "node:path";

import { LINK_RE } from "./constants.mjs";

/**
 * @param {string} content
 */
export function extractMarkdownLinks(content) {
  /** @type {Array<{ text: string, href: string }>} */
  const links = [];
  for (const match of content.matchAll(LINK_RE)) {
    links.push({ text: match[1], href: match[2] });
  }
  return links;
}

/**
 * @param {string} href
 */
function isExternalWikiHref(href) {
  return href.startsWith("http://") || href.startsWith("https://");
}

/**
 * @param {string} rootDir
 * @param {string} href
 * @param {string} fromFile
 */
export function resolveWikiHref(rootDir, href, fromFile) {
  if (!href.endsWith(".md") || isExternalWikiHref(href)) {
    return null;
  }
  const resolved = path.normalize(path.join(path.dirname(fromFile), href));
  const wikiRoot = path.join(rootDir, "docs/wiki");
  if (!resolved.startsWith(wikiRoot)) {
    return null;
  }
  return resolved;
}
