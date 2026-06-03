# RED_TEAM.md - {{PROJECT_NAME}}

Adversarial review: the place to write down how this project might fail, before it does.

## Standing questions for every change

- What is the most dangerous hidden assumption in this diff?
- What breaks if input is empty, huge, malformed, or hostile?
- Which `LOCKED` decision does this quietly violate?
- What did the tests *not* cover?

## Known risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| _example: scope creep delays the core slice_ | medium | high | timebox; cut to vertical slice |

## Premortem

> Assume {{PROJECT_NAME}} failed six months from now. Write the story of how. Update this
> section whenever a new failure mode becomes plausible.

_Last reviewed: {{DATE}}_
