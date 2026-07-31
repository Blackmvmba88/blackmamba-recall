# Security Policy

## Current scope

BlackMamba Recall is an experimental local-first automation system. The current milestone records and simulates workflows; it does not perform real SoundCloud publication or uncontrolled browser automation.

## Safety defaults

- Local SQLite storage by default.
- Sensitive field names are detected and their values are replaced with `[REDACTED]` before persistence.
- Automatic execution requires repeated successful sessions and a confidence threshold.
- Unexpected states halt execution and enter exception mode.
- Publish, release, submit, purchase and delete actions require explicit human confirmation.
- Observation can be paused immediately.

## Never store

Do not commit or persist passwords, session cookies, API tokens, private keys, payment details or recovery codes.

## Reporting

Open a private security advisory in GitHub for vulnerabilities. Do not include live credentials or personal data in issues, logs, screenshots or test fixtures.

## Before enabling real automation

A future release must add threat modeling, permission scoping, signed action logs, selector hardening, replay protection, screenshot redaction and an emergency stop tested outside the application process.
