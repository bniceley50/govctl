# Decisions - {{PROJECT_NAME}}

Architectural decisions, append-only. Each entry has a stable id (`D001`, `D002`, ?) and a
lifecycle status. This file is the source of truth that `govctl validate` enforces.

<!--
HOW TO ADD A DECISION
Copy the block below. Status is one of: PROPOSED | LOCKED | SUPERSEDED (by D00Y).
A SUPERSEDED entry MUST name its successor so the chain stays intact, and that successor
must exist in this file.

### D00X - <short title>
- **Status:** PROPOSED
- **Date:** YYYY-MM-DD
- **Context:** Why this decision came up.
- **Decision:** What was decided.
- **Consequences:** What this commits the project to.
-->

### D001 - Adopt the govctl governance stack
- **Status:** PROPOSED
- **Date:** {{DATE}}
- **Context:** {{PROJECT_NAME}} needs a durable, enforceable record of architectural decisions.
- **Decision:** Track decisions in this file and enforce drift with `govctl validate`.
- **Consequences:** Any `D###` referenced in code or commits must exist here, or CI fails.
