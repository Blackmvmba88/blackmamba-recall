# Architecture

BlackMamba Recall is local-first and event-driven. The timeline of recorded user actions is authoritative; predictions never rewrite history.

## Data flow

```text
Keyboard / Accessibility event
          |
          v
      observer
          |
          v
  sanitized Step event
          |
          +------> SQLite audit log
          |
          v
  workflow-engine
          |
          +--> Observe
          +--> Suggest
          +--> Execute known safe step
          +--> Require human confirmation
          +--> Halt / exception
```

## Crates

- `recall-core`: canonical `Workflow`, `Session`, `Step`, status and actor models.
- `recall-observer`: observation events and immediate pause control.
- `recall-workflow-engine`: supervised promotion and fallback state machine.
- `recall-storage`: local SQLite persistence.
- `recall-macos-accessibility`: boundary for macOS Accessibility APIs.
- `recall-cli`: inspection, database initialization and offline simulations.
- `recall-desktop`: visible-state desktop shell.

## Invariants

1. A recorded user step is never silently changed into an agent step.
2. Passwords, tokens, secrets and payment fields are redacted before persistence.
3. Low confidence in automatic mode halts execution and enters exception mode.
4. Publish, release, submit, purchase and delete actions require a human.
5. Observation can be paused immediately.
6. Browser automation is disabled in the initial milestone.

## SoundCloud learning sequence

1. First successful run: observation only.
2. Second and third runs: suggestions are shown and the user confirms or corrects them.
3. After three validated runs: known non-final steps may execute automatically above the configured confidence threshold.
4. Any unexpected page state: halt, mark the failed step and return control to the user.
5. The final Publish action remains manual.
