/**
 * @param {string} line
 * @param {Record<string, string | string[]>} fields
 * @param {string | null} currentList
 */
function appendListItem(line, fields, currentList) {
  const listMatch = line.match(/^  - (.+)$/);
  if (!listMatch || !currentList || !Array.isArray(fields[currentList])) {
    return currentList;
  }
  fields[currentList].push(listMatch[1].trim());
  return currentList;
}

/**
 * @param {string} line
 * @param {Record<string, string | string[]>} fields
 */
function appendScalarField(line, fields) {
  const fieldMatch = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
  if (!fieldMatch) {
    return null;
  }
  const [, key, value] = fieldMatch;
  if (value === "") {
    fields[key] = [];
    return key;
  }
  fields[key] = value.trim();
  return null;
}

/**
 * @param {string} line
 * @param {Record<string, string | string[]>} fields
 * @param {string | null} currentList
 * @returns {string | null}
 */
function applyFrontmatterLine(line, fields, currentList) {
  const afterList = appendListItem(line, fields, currentList);
  if (afterList !== currentList) {
    return afterList;
  }
  return appendScalarField(line, fields);
}

/**
 * @param {string} content
 */
export function parseFrontmatter(content) {
  if (!content.startsWith("---\n")) {
    return null;
  }
  const end = content.indexOf("\n---\n", 4);
  if (end === -1) {
    return null;
  }

  /** @type {Record<string, string | string[]>} */
  const fields = {};
  let currentList = null;
  for (const line of content.slice(4, end).split("\n")) {
    currentList = applyFrontmatterLine(line, fields, currentList);
  }
  return fields;
}
