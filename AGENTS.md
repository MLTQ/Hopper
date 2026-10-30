# Hopper development

Hopper is a standalone, paired media library for macOS, Linux, Windows and iOS.
Read README.md and docs/architecture.md before changing contracts.

- Keep the Rust core independent of Gruve and the desktop UI. iOS embeds the same core.
- Every code file has a companion Markdown file explaining purpose and contracts.
- Preserve original media. Never mutate a user's source file when importing or editing.
- Pairing grants access to a device's Hopper library, never arbitrary filesystem paths.
- Record follow-up work with `bd` (use `--no-daemon` in restricted environments).
- Run targeted tests and builds; report device/platform checks not actually performed.
- This repository starts local-only. Do not create or publish a remote without a request.
- At handoff, update issue status and docs; run `bd sync --flush-only` to persist task records.

The generated Beads workflow below applies when a remote is configured; local-only work does
not require a push or pull. Do not clear unrelated work, stashes, or branches.

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
