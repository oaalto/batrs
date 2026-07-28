# 01 — Move LineEffect to ansi

**Parent:** `prd.md`

**What to build:** Co-locate `LineEffect` and `apply_to` with `StyledLine` in the ansi module while keeping trigger-domain containers (`TriggerEffects`, `OriginalLineEffects`, `apply_line_effects_to`) in triggers. Existing consumers keep importing `LineEffect` through triggers — no call-site import churn. Add LLM-oriented module and item documentation that explains how line edits (`original.edits`, `gag`) differ from emitted lines (`lines`), plus one minimal illustrative example. Add per-variant dispatch smoke tests proving `apply_to` wires each variant to the right `StyledLine` mutator. Land as one green slice with identical runtime behavior.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `LineEffect` and `apply_to` live in ansi; re-exported from ansi module root and from triggers so canonical and consumer import paths both work
- [ ] Triggers retains `OriginalLineEffects`, `TriggerEffects`, and `apply_line_effects_to` unchanged; duplicate `LineEffect` definition removed from triggers
- [ ] No call-site import edits — guild triggers, rule engine, and magic lore analysis keep `crate::triggers::LineEffect`
- [ ] `apply_to` API unchanged — same match arms, same method name; no `StyledLine::apply_effect`
- [ ] Module docs in effect module explain line edits vs emitted lines, document `gag` as a sibling flag (not a `LineEffect` variant), include cross-refs to related trigger types, and one minimal code example
- [ ] Item docs on the enum, each variant, and `apply_to` describe when to use each variant and plain-byte index semantics where relevant
- [ ] Four inline dispatch-smoke tests (one per variant) assert one observable change after `apply_to`; no duplicate mutator parity coverage from styled-line tests
- [ ] All existing tests pass; workflow gates (format, build, clippy, `cargo test --all-targets --all-features`) pass with no new warnings
