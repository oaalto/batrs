# Documentation

- **Document functions:** Provide at least function-level documentation for public or non-trivial functions, explaining purpose, parameters, return values, and important failure modes.
- **Use local doc style:** Match the project's established documentation format, such as Doxygen, rustdoc, JSDoc, docstrings, or equivalent language conventions.
- **Document complex logic:** Add concise block or line-level documentation for non-obvious algorithms, trade-offs, constraints, and edge-case handling.
- **Keep docs in sync with behavior:** When functionality, configuration, setup, CLI/API usage, scripts, or operational workflows change, update the related documentation in the same change.
- **Update user-facing surfaces:** Keep affected README files, usage guides, API docs, CLI docs, runbooks, build/setup steps, troubleshooting notes, and repository script documentation aligned with current behavior.
- **Preserve or improve existing docs:** Do not remove or alter documentation unless replacing it with clearer or more accurate content.
- **Reflect interface changes:** If function/module contracts change, update parameter, return-value, and error-handling documentation.

## Context And Decisions

- **Use project language when present:** When documentation explains domain terms, behavior, module relationships, architecture, workflows, or design rationale, consult `CONTEXT.md` or `CONTEXT-MAP.md` if present and use the same vocabulary.
- **Align with ADRs when present:** When documentation explains architecture, dependencies, constraints, or trade-offs, consult relevant `docs/adr/` records if present and keep the explanation consistent with accepted decisions.
- **Document dependencies with context:** For complex internal or external dependencies, explain ownership, direction, purpose, and constraints using the project's glossary and ADR-backed decisions where available.
- **Reference ADRs selectively:** Link or name an ADR when documenting a non-obvious architectural constraint, dependency direction, or trade-off that future maintainers might otherwise re-litigate.

## Review Expectations

- Document updates should be included in the same PR/commit as the behavior/config change.
- Missing docs updates for user-visible changes should be treated as incomplete work.
