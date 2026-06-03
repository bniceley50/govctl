# AGENTS.md - govctl

Roles for agents (human or AI) working on govctl.

## Roles

| Role       | Responsibility                                                          |
|------------|-------------------------------------------------------------------------|
| Builder    | Implements features against locked decisions (D001-D004).               |
| Reviewer   | Adversarially checks diffs for correctness and drift (see RED_TEAM.md). |
| Maintainer | Owns DECISIONS.md; approves supersessions.                              |

## Conventions

- Reference the decision a change honors in the commit message (e.g. `honors D003`).
  `govctl validate` cross-checks those references against DECISIONS.md.
- New architectural choices get a decision entry before merge, not after.
- Keep source ASCII-only.

_Last reviewed: 2026-06-03_
