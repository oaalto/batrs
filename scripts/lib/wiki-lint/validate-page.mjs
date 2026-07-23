import { VALID_STATUS, VALID_TYPES } from "./constants.mjs";
import { parseFrontmatter } from "./frontmatter.mjs";
import { validateFrontmatterEnums } from "./validate-frontmatter.mjs";
import { validateRequiredSections } from "./validate-sections.mjs";

const REQUIRED_FRONTMATTER_KEYS = ["title", "type", "status", "updated"];

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} key
 */
function hasRequiredStringField(fm, key) {
  return typeof fm[key] === "string" && fm[key].length > 0;
}

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFrontmatterRequiredFields(fm, rel, errors) {
  for (const key of REQUIRED_FRONTMATTER_KEYS) {
    if (hasRequiredStringField(fm, key)) {
      continue;
    }
    errors.push(`${rel}: frontmatter missing required field "${key}"`);
  }
}

/**
 * @param {string} rel
 * @param {string} content
 * @param {string[]} errors
 */
export function validateWikiPage(rel, content, errors) {
  const fm = parseFrontmatter(content);
  if (!fm) {
    errors.push(`${rel}: missing or invalid YAML frontmatter`);
    return;
  }
  validateFrontmatterRequiredFields(fm, rel, errors);
  validateFrontmatterEnums(fm, rel, errors);
  validateRequiredSections(content, rel, errors);
}
