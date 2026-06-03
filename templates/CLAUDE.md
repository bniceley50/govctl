# CLAUDE.md - {{PROJECT_NAME}}

Operating guidance for AI coding agents working in this repository. Read this first.

## What this project is

> One or two sentences: what {{PROJECT_NAME}} does and who it is for.

## Ground rules

- **Decisions are binding.** Architectural choices live in `DECISIONS.md`. Do not silently
  contradict a `LOCKED` decision - if one needs to change, supersede it explicitly.
- **Match the surrounding code.** Naming, structure, comment density, and idioms should look
  like the code already here.
- **Verify before claiming done.** Run the build and tests; report failures honestly.
- **Small, reviewable changes.** Prefer a focused diff over a sweeping rewrite.

## Workflow

1. Read `DECISIONS.md` and `sprint-status.yaml` to understand current constraints.
2. Make the change.
3. Run `govctl validate .` to confirm the decision log and the code have not drifted apart.
4. Record any new architectural decision in `DECISIONS.md`.

## Project map

> List the key directories and entry points here so an agent can orient quickly.

_Last reviewed: {{DATE}}_
