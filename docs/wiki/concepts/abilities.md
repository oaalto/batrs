---
title: Abilities
type: concept
status: current
updated: 2026-07-27
sources:
  - CONTEXT.md
  - src/abilities/mod.rs
  - docs/features/move-guild-wrappers-to-abilities/prd.md
---

# Abilities

## Summary

Abilities is the bounded context for canonical BatMUD `use` and `cast` command-line formatting. Guild command modules consume it for `use_skill`, `cast_spell`, and related helpers.

## Verified Facts

- Implementation: `src/abilities/mod.rs` (plus `src/abilities/floating_disc.rs` for floating-disc helpers).
- Primary exports:
  - `use_skill` / `cast_spell` — build a client-send line from command `Data` (target args from `data.args`).
  - `targeted_use` / `targeted_cast` — logical lines without the `@` prefix.
  - `client_send_line` — adds at most one `@` prefix to a logical line.
- Targeted-command formatting rules:
  - Empty target args → `use '<skill>'` / `cast '<spell>'` (logical line; `client_send_line` adds `@`).
  - Non-empty target args → `target <t>;use '<skill>' <t>` / `target <t>;cast '<spell>' <t>`.
- Guild command modules import `use_skill` and `cast_spell` from Abilities, not from the Guild root module.

## Other exports

- `cast_quoted_tail` / `use_quoted_tail` — quoted name with optional tail fragment.
- `cast_quoted_with_suffix` / `use_quoted_with_suffix` — quoted name plus tail, wrapped for client send.
- `compound_send` — join logical fragments with `;`, then one client prefix.
- `repeat_inf_cast_heal_self` — `repeat inf cast heal self` line for Civmage / Mage / Psionicist `chf`.
- `floating_disc` submodule — floating-disc command helpers.

## Related

- [Guild Catalog](guild-catalog.md)
- [Command Dispatch](command-dispatch.md) — does not own guild `use`/`cast` formatting
- `CONTEXT.md` — Abilities section
