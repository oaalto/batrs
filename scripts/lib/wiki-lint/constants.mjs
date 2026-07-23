export const VALID_TYPES = new Set([
  "concept",
  "subsystem",
  "workflow",
  "debugging",
  "trap",
  "source-note",
  "synthesis"
]);
export const VALID_STATUS = new Set(["current", "historical", "draft", "needs-verification"]);
export const LOG_ENTRY_RE = /^## \[\d{4}-\d{2}-\d{2}\] (update|ingest|skip) \| /;
export const EXEMPT_BASENAMES = new Set(["schema.md", "log.md", "index.md", "page-template.md", "path-map.json"]);
export const REQUIRED_SECTIONS_ANY = ["Verified Facts", "Agent Synthesis"];
export const LINK_RE = /\[([^\]]*)\]\(([^)]+)\)/g;
