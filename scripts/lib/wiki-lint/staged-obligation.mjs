import { getStagedFiles, stagedWikiTouchPresent } from "./git-staged.mjs";
import { matchesSourcePattern } from "./patterns.mjs";
import { readPathMap } from "./wiki-files.mjs";

const OBLIGATION_MESSAGE =
  "Staged changes touch path-map sources without a matching docs/wiki/ update or log entry (update|ingest|skip). Run /wiki-update or append docs/wiki/log.md before commit.";

/**
 * @param {unknown} mapping
 * @param {string} file
 */
function mappingMatchesFile(mapping, file) {
  const sources = Array.isArray(mapping.sources) ? mapping.sources : [];
  return sources.some((pattern) => matchesSourcePattern(file, pattern));
}

/**
 * @param {unknown[]} mappings
 * @param {string} file
 */
function triggeredIndexesForFile(mappings, file) {
  /** @type {string[]} */
  const indexes = [];
  for (const [index, mapping] of mappings.entries()) {
    if (mappingMatchesFile(mapping, file)) {
      indexes.push(String(index));
    }
  }
  return indexes;
}

/**
 * @param {unknown[]} mappings
 * @param {string[]} stagedFiles
 */
function collectTriggeredMappingIndexes(mappings, stagedFiles) {
  /** @type {Set<string>} */
  const triggered = new Set();
  for (const file of stagedFiles) {
    for (const index of triggeredIndexesForFile(mappings, file)) {
      triggered.add(index);
    }
  }
  return triggered;
}

/**
 * @param {unknown[]} mappings
 * @param {Set<string>} triggeredIndexes
 */
function obligationIsBlocked(mappings, triggeredIndexes) {
  for (const index of triggeredIndexes) {
    if (mappings[Number(index)]?.allowSkip === false) {
      return true;
    }
  }
  return false;
}

/**
 * @param {boolean} blocked
 */
function obligationWarning(blocked) {
  const suffix = blocked ? " (includes allowSkip: false mappings)" : "";
  return `${OBLIGATION_MESSAGE}${suffix}`;
}

/**
 * @param {string} rootDir
 * @param {string[] | null} stagedFiles
 */
function resolveStagedMappings(rootDir, stagedFiles) {
  const pathMap = readPathMap(rootDir, []);
  const mappings = Array.isArray(pathMap.mappings) ? pathMap.mappings : [];
  const effectiveStagedFiles = stagedFiles ?? getStagedFiles(rootDir);
  return {
    mappings,
    triggeredMappings: collectTriggeredMappingIndexes(mappings, effectiveStagedFiles),
    effectiveStagedFiles
  };
}

/**
 * @param {string} rootDir
 * @param {string[]} effectiveStagedFiles
 * @param {Set<string>} triggeredMappings
 */
function shouldWarnStagedObligation(rootDir, effectiveStagedFiles, triggeredMappings) {
  if (triggeredMappings.size === 0) {
    return false;
  }
  return !stagedWikiTouchPresent(rootDir, effectiveStagedFiles);
}

/**
 * @param {string} rootDir
 * @param {boolean} staged
 * @param {string[] | null} stagedFiles
 * @param {string[]} warnings
 */
export function checkStagedObligation(rootDir, staged, stagedFiles, warnings) {
  if (!staged) {
    return;
  }
  const { mappings, triggeredMappings, effectiveStagedFiles } = resolveStagedMappings(rootDir, stagedFiles);
  if (!shouldWarnStagedObligation(rootDir, effectiveStagedFiles, triggeredMappings)) {
    return;
  }
  warnings.push(obligationWarning(obligationIsBlocked(mappings, triggeredMappings)));
}
