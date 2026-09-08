# Slice 01 — Update Ranger `cs` opener

## Status
accepted

## Parent
- No parent PRD exists yet for this chat-derived plan.
- Feature folder: `docs/features/ranger-cs-kill-opener/`

## Goal
Align the Ranger guild `cs` shortcut with the accepted plan so the opener requires a target and sends `target <target>;use 'bladed fury' <target>;kill <target>` through Command Dispatch.

## Scope
- Update `src/guilds/ranger/commands.rs` so Ranger `cs`:
  - trims the provided target,
  - rejects empty or whitespace-only targets locally,
  - sends `target <target>;use 'bladed fury' <target>;kill <target>` when a target is present.
- Reuse existing Abilities helpers for the targeted `use 'bladed fury'` portion instead of duplicating command formatting.
- Keep Ranger `ubf` and `utc` behavior unchanged.
- Update Ranger user docs in `docs/guilds/ranger.md` so the shortcut description matches runtime behavior.
- Add or update Rust tests covering:
  - `cs` with a target,
  - `cs` with surrounding whitespace in the target,
  - `cs` without a target,
  - `cs` with whitespace-only input.

## Non-goals
- No alias rename; keep `cs` as the Ranger shortcut.
- No changes to other guilds that also use `cs`.
- No Guild Catalog, `CONTEXT.md`, or ADR changes.
- No new abstraction beyond existing Abilities helpers.

## Blocked by
- None.

## Acceptance checks
- `src/guilds/ranger/commands.rs` sends `@target <target>;use 'bladed fury' <target>;kill <target>` for `cs <target>`.
- `src/guilds/ranger/commands.rs` emits a local missing-target message and sends nothing for empty or whitespace-only target input.
- `docs/guilds/ranger.md` documents `kill` instead of `@k` for Ranger `cs`.
- Rust tests cover the updated send line and required-target behavior.
- Validation passes:
  - `cargo fmt --all`
  - `cargo build --all-targets`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --all-features`
