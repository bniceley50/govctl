# Decisions - govctl

Architectural decisions for govctl itself. govctl validates this file with its own `validate`
command (it dogfoods its own rules). Append-only; each entry has a stable id and a status.

<!--
HOW TO ADD A DECISION
Copy this block. Status is one of: PROPOSED | LOCKED | SUPERSEDED (by D00Y).
A SUPERSEDED entry MUST name a successor that exists in this file.

### D00X - <short title>
- **Status:** PROPOSED
- **Date:** YYYY-MM-DD
- **Context:** ...
- **Decision:** ...
- **Consequences:** ...
-->

### D001 - Adopt the govctl governance stack
- **Status:** LOCKED
- **Date:** 2026-06-03
- **Context:** govctl's own development needs a durable, enforceable decision record.
- **Decision:** Track decisions here and enforce drift with `govctl validate . --strict` in CI.
- **Consequences:** Any `D###` referenced in source or commits must exist here, or CI fails.

### D002 - Comment-aware DECISIONS parsing
- **Status:** LOCKED
- **Date:** 2026-06-03
- **Context:** The DECISIONS.md template carries an instructional `<!-- ... ### D00X ... -->`
  block. A naive parser read that example heading as a real decision (a phantom `D00`).
- **Decision:** Strip `<!-- ... -->` HTML comment spans before parsing the decision log, with a
  line-state parser that preserves line numbers.
- **Consequences:** Template instruction blocks never pollute the parsed decision set.

### D003 - Reference suppression via .govctlignore and inline markers
- **Status:** LOCKED
- **Date:** 2026-06-03
- **Context:** Test fixtures and documentation examples legitimately contain fake D-numbers.
  Scanned blindly, they produce false "orphaned reference" errors.
- **Decision:** Honor a `.govctlignore` (glob-lite) for whole paths, plus inline `govctl:ignore`
  and `govctl:ignore-start`/`-end` range markers applied to raw lines before comment stripping.
- **Consequences:** Drift detection stays precise; doc examples and fixtures don't trigger noise.

### D004 - CI via a reusable composite action with source-build fallback
- **Status:** LOCKED
- **Date:** 2026-06-03
- **Context:** Every repo that adopts govctl needs `validate --strict` on each PR, ideally in two
  lines, without each repo hand-rolling install logic.
- **Decision:** Ship a composite GitHub Action that installs govctl (prebuilt-binary fast path,
  build-from-source fallback) and runs validate. Other repos reference it with one `uses:` line.
- **Consequences:** Adding govctl to a repo is a copy-paste of one workflow; install logic is
  centralized and fixed once.

### D005 - Parser accepts any heading level and is UTF-8 safe
- **Status:** LOCKED
- **Date:** 2026-06-03
- **Context:** Tested against real decision logs (GravenSpire, Clinic Notes AI), govctl parsed
  zero decisions: those logs use `## D001` (level-2) headings and contain em-dash separators.
  The parser only matched `###`, and `strip_html_comments` corrupted multibyte UTF-8.
- **Decision:** Recognize a decision heading at any markdown level (`#`..`######`) whose text is
  `D<digits>`, and parse strictly on `&str`/`chars` (never cast bytes to `char`).
- **Consequences:** govctl is compatible with real-world `##`-style logs and preserves non-ASCII
  titles. Regression tests cover both the heading level and the multibyte path.
