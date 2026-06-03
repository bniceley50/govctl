# RED_TEAM.md - govctl

Adversarial review notes for govctl.

## Standing questions

- What is the most dangerous hidden assumption in this diff?
- What does the scanner miss, and what does it over-report (false orphans)?
- Which LOCKED decision (D001-D004) does this quietly violate?

## Known risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| False "orphaned reference" from doc/test fixtures | high | medium | `.govctlignore` + inline suppression (D003) |
| Template comment parsed as a real decision | medium | medium | comment-aware parser (D002) |
| Non-ASCII source mis-decoded by the toolchain | medium | medium | enforce ASCII-only source |
| Git history scan misses refs in commit bodies | low | low | scan full `%B`, not just subject |

## Premortem

Assume govctl shipped and nobody adopted it. Most likely cause: drift detection produced false
positives that trained users to ignore it. Mitigation: suppression must be precise and
documented (D003), and `validate` must pass on govctl's own repo at all times.

_Last reviewed: 2026-06-03_
