# Logging Practices

- **Use clear levels and categories:** Log with consistent severity and component/category tags so signals are easy to filter.
- **Log actionable context:** Include identifiers, relevant state transitions, and error causes that help reproduction and triage.
- **Prefer structured facts over prose-heavy messages:** Keep logs concise and machine-searchable where possible.
- **Avoid noise:** Do not log every operation; prioritize warnings, errors, significant state changes, and decision points.
- **No sensitive data:** Never log secrets, credentials, tokens, or personal data unless explicitly redacted by policy.
- **Control verbosity externally:** Make detailed debug logging configurable through environment/config flags. batrs uses `env_logger` / `RUST_LOG` (see `src/main.rs`).

## Review Expectations

- New logging should improve diagnosability without materially increasing noise.
- Changes that alter behavior should update logging when it improves observability of new flows/failures.
