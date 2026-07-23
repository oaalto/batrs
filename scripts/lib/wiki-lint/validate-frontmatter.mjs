import { VALID_STATUS, VALID_TYPES } from "./constants.mjs";

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFrontmatterType(fm, rel, errors) {
  if (typeof fm.type === "string" && !VALID_TYPES.has(fm.type)) {
    errors.push(`${rel}: invalid frontmatter type "${fm.type}"`);
  }
}

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFrontmatterStatus(fm, rel, errors) {
  if (typeof fm.status === "string" && !VALID_STATUS.has(fm.status)) {
    errors.push(`${rel}: invalid frontmatter status "${fm.status}"`);
  }
}

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFrontmatterUpdated(fm, rel, errors) {
  if (typeof fm.updated === "string" && !/^\d{4}-\d{2}-\d{2}$/.test(fm.updated)) {
    errors.push(`${rel}: frontmatter updated must be YYYY-MM-DD`);
  }
}

/**
 * @param {Record<string, string | string[]>} fm
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFrontmatterSources(fm, rel, errors) {
  if (!Array.isArray(fm.sources) || fm.sources.length === 0) {
    errors.push(`${rel}: frontmatter sources must list at least one entry`);
  }
}

export function validateFrontmatterEnums(fm, rel, errors) {
  validateFrontmatterType(fm, rel, errors);
  validateFrontmatterStatus(fm, rel, errors);
  validateFrontmatterUpdated(fm, rel, errors);
  validateFrontmatterSources(fm, rel, errors);
}
