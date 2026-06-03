# AGENTS.md - {{PROJECT_NAME}}

Roles and responsibilities for the agents (human or AI) who work on {{PROJECT_NAME}}.

## Roles

| Role        | Responsibility                                                        |
|-------------|-----------------------------------------------------------------------|
| Builder     | Implements features against locked decisions.                         |
| Reviewer    | Adversarially checks diffs for correctness and drift (see RED_TEAM.md).|
| Maintainer  | Owns `DECISIONS.md`; approves supersessions.                          |

## Conventions

- Every nontrivial change references the decision(s) it honors in its commit message
  (e.g. `honors D001`). `govctl validate` cross-checks those references.
- A change that introduces a new architectural choice must add a decision entry **before**
  merge, not after.
- When in doubt, leave a `PROPOSED` decision and ask the Maintainer to lock it.

## Handoff

State what is in flight, what is blocked, and what the next agent should pick up.

_Last reviewed: {{DATE}}_
