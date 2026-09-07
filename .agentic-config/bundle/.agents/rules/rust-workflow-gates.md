# Rust Workflow Gates

- **Companion skill:** This rule specializes Rust gates; load the bundled `workflow` skill for base cross-language gate policy.
- **Gate 1 - format:** Run `cargo fmt --all --check` before other validation.
- **Gate 2 - lint:** Run `cargo clippy --all-targets --all-features -- -D warnings`.
- **Gate 3 - tests:** Run `cargo test --all-targets --all-features`.
- **Gate 4 - docs/build (as required):** Run `cargo doc --no-deps` and/or `cargo build --release` when the change affects published docs or release behavior.
- **Fail-fast:** Stop at the first failing gate and fix it before proceeding.

## Scope Guidance

- Prefer crate-scoped commands for small local changes when project policy allows it.
- Run workspace-wide checks before completion unless the repository explicitly documents narrower completion criteria.
