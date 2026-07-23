# Headroom Consultation

## Pre-task consultation

Apply before tasks that produce or consume large tool outputs, logs, or conversation context — not only when making file changes.

1. **Verify runtime:** Check `headroom` is on PATH (`command -v headroom`). When missing, use the proxy or wrap workflow documented in `.agentic-config/INSTALL.md`.
2. **Compress before the LLM:** Prefer Headroom compression for bulky command output, stack traces, and repeated context blocks before they reach the model context window.
3. **Proxy path:** When a local Headroom proxy is configured, route eligible tool results through it instead of pasting raw megabyte-scale output into chat.
4. **Pi extension:** On Pi targets, use the installed Headroom extension for tool-result compression with alignment guards — not a separate MCP registration path in v1.

## When to consult

Treat these as Headroom-first when the runtime is installed: large `grep`/`find`/`test` logs, multi-file diffs, CI failure blobs, verbose build output, and repeated tool loops that re-send similar context.

## Source hierarchy

- Treat installed Headroom proxy/MCP/extension behavior as the live compression path.
- Treat bundled `headroom-consultation` as ADC onboarding guidance; upstream RTK blocks in host files may add target-specific detail — dedupe during post-install review when both are present.

## Operations

See `https://github.com/headroomlabs-ai/headroom` for proxy, `headroom wrap`, and MCP usage. ADC installs via PyPI `headroom-ai` and optional Pi extension `npm:@ryan_nookpi/pi-extension-headroom`.
