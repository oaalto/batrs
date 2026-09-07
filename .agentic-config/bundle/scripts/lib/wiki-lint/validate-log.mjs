import fs from "node:fs";

import { LOG_ENTRY_RE } from "./constants.mjs";

/**
 * @param {string} line
 */
function isInvalidLogHeader(line) {
  return line.startsWith("## [") && !LOG_ENTRY_RE.test(line);
}

/**
 * @param {string} logPath
 * @param {string[]} errors
 */
export function validateLogEntries(logPath, errors) {
  if (!fs.existsSync(logPath)) {
    return;
  }
  for (const line of fs.readFileSync(logPath, "utf8").split("\n")) {
    if (isInvalidLogHeader(line)) {
      errors.push(`docs/wiki/log.md: invalid log entry header: ${line}`);
    }
  }
}
