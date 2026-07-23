import { REQUIRED_SECTIONS_ANY } from "./constants.mjs";

/**
 * @param {string} content
 * @param {string} rel
 * @param {string[]} errors
 */
function validateSummarySection(content, rel, errors) {
  if (!/^## Summary\b/m.test(content)) {
    errors.push(`${rel}: missing required section "## Summary"`);
  }
}

/**
 * @param {string} content
 * @param {string} rel
 * @param {string[]} errors
 */
function validateFactsOrSynthesisSection(content, rel, errors) {
  const hasSection = REQUIRED_SECTIONS_ANY.some((heading) => new RegExp(`^## ${heading}\\b`, "m").test(content));
  if (!hasSection) {
    errors.push(`${rel}: missing "## Verified Facts" or "## Agent Synthesis"`);
  }
}

/**
 * @param {string} content
 * @param {string} rel
 * @param {string[]} errors
 */
function validateRelatedSection(content, rel, errors) {
  if (!/^## Related\b/m.test(content)) {
    errors.push(`${rel}: missing required section "## Related"`);
  }
}

export function validateRequiredSections(content, rel, errors) {
  validateSummarySection(content, rel, errors);
  validateFactsOrSynthesisSection(content, rel, errors);
  validateRelatedSection(content, rel, errors);
}
