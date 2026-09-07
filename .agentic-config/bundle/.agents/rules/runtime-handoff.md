# Runtime Handoff

- **Delegate runtime execution when needed:** If validation depends on local environment constraints (hardware, credentials, privileged access, deployment context), ask the user to run commands.
- **Do not guess runtime outcomes:** Use actual user-provided runtime output instead of assumptions.
- **Request reproducible commands:** Provide copy-pastable commands and expected arguments so the user can run the exact scenario.
- **Ask for targeted evidence:** Request key logs, exit codes, and observable signals needed to confirm or reject hypotheses.

## Handoff Format

- **Command to run:** exact command(s) user should execute.
- **Expected success signals:** logs/output patterns indicating correct behavior.
- **Expected failure signals:** logs/output patterns indicating likely root causes.
- **What to return:** the specific log segments, errors, and command output needed for analysis.

## Review Expectations

- Runtime-dependent debugging should include clear handoff instructions.
- Follow-up analysis should reference returned logs/signals directly.
