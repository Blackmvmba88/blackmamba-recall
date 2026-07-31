# Privacy

BlackMamba Recall is designed to keep workflow data on the user's machine.

## Collected data

A session may contain timestamps, action types, target identifiers, non-sensitive values, actor identity (`user` or `agent`), confidence, expected and observed state, status and optional evidence references.

## Excluded data

Passwords, tokens, secrets, authorization values and payment fields must not be stored. Matching values are redacted before persistence.

## Storage

The default store is a local SQLite database. No telemetry, cloud synchronization or remote model call is enabled by this repository.

## User control

The user can pause observation, inspect the audit log, correct steps and delete local databases. A future UI must make these controls visible and reachable at all times.

## Screenshots

Screenshot capture is not enabled in the initial milestone. Before it is introduced, the system must support application allowlists, region redaction, retention limits and explicit user-visible capture status.
