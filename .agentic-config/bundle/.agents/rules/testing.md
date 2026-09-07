# Testing Guidelines

- **Test preservation:** Do not remove or disable existing tests unless the covered functionality is also removed.
  If a test must be temporarily disabled, add a clear comment with the reason, a tracking issue for re-enabling it, and the expected re-enable condition.
- **New functionality:** Every new feature, component, or library should include corresponding tests.
- **Bug fixes:** Every bug fix should include a regression test that fails before the fix and passes after it.
- **Test execution:** Always run the complete automated test suite before marking work as done.
- **Skipped/disabled reporting:** If a test run reports skipped or disabled tests, always notify the user and include the count plus relevant test identifiers.
- **Skip cause policy:** Skips are only acceptable when tests are explicitly disabled by policy. Skips caused by other reasons (for example environment/config failures or test/runtime errors) must be fixed before marking work as done.
- **Test granularity:**
  - **Unit tests:** Write focused tests for individual functions and data structures, especially for pure logic.
  - **Integration tests:** For cross-component features and end-to-end flows, add integration tests that validate critical behavior.
- **Test automation:** Tests should be part of an automated suite that runs with a single project command.
