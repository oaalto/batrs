/**
 * @param {string} filePath
 * @param {string} pattern
 */
export function matchesSourcePattern(filePath, pattern) {
  const normalized = filePath.replace(/\\/g, "/");
  if (pattern.endsWith("/**")) {
    const prefix = pattern.slice(0, -3);
    return normalized === prefix || normalized.startsWith(`${prefix}/`);
  }
  if (!pattern.includes("*")) {
    return normalized === pattern;
  }
  const regex = new RegExp(
    `^${pattern
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*\*/g, ".*")
      .replace(/\*/g, "[^/]*")}$`
  );
  return regex.test(normalized);
}
