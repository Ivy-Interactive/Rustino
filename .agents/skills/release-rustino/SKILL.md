---
name: release-rustino
description: Switches to latest development branch, bumps the patch version in src/Directory.Build.props, creates a PR from development into main with @claude review, merges if no conflicts, syncs main back to development, and triggers the NuGet release.
---

# Rustino Release Automator

This skill automates the release preparation, branch merging, branch synchronization, and NuGet deployment process for the Rustino repository.

## Invocation

```bash
/release-rustino
```

## What This Skill Does

1. **Ensures the workspace is clean** and checks out the `development` branch.
2. **Pulls the latest updates** from `origin/development`.
3. **Bumps the patch version** in `src/Directory.Build.props` (e.g. `0.3.4` -> `0.3.5`).
4. **Commits and pushes** the version bump to `development`.
5. **Creates a Pull Request** from `development` into `main` using the GitHub CLI (`gh`).
6. **Requests review from @claude** bot on the PR by posting a comment with `@claude review this`.
7. **Waits for mergeability checks** to ensure there are no conflicts.
8. **Merges the PR** into `main`.
9. **Synchronizes the branches** by switching to `main`, pulling, switching to `development`, merging `main`, and pushing.
10. **Triggers the GitHub release workflow** (`release.yml`) on `main`.

## Prerequisites

- **GitHub CLI** (`gh`) must be installed and authenticated.
- **PowerShell 7** (`pwsh`) or Windows PowerShell must be available.

## Step-by-Step Execution

You can execute the entire release flow automatically by running:

```powershell
powershell -File .agents/skills/release-rustino/release.ps1
```

If you prefer to run it manually or need to troubleshoot a step, follow the steps below:

### Phase 1 — Switch and Bump Version
```powershell
git checkout development
git pull origin development

# Edit src/Directory.Build.props to increment the patch version (e.g., <Version>0.3.5</Version>)
# Commit and push:
git add src/Directory.Build.props
git commit -m "chore: bump patch version to 0.3.5"
git push origin development
```

### Phase 2 — Create PR and Request Review
Create the Pull Request and comment for Claude review:
```powershell
gh pr create --base main --head development --title "Release: Merge development into main (v0.3.5)" --body "Automated release PR."
# Note down the PR URL/number returned, then comment:
gh pr comment <PR_URL> --body "@claude review this"
```

### Phase 3 — Verify and Merge PR
Ensure no conflicts exist, then merge:
```powershell
gh pr merge <PR_URL> --merge --delete-branch=false
```

### Phase 4 — Sync branches
```powershell
git checkout main
git pull origin main
git checkout development
git merge main --no-ff -m "Merge branch 'main' into development to sync [skip ci]"
git push origin development
```

### Phase 5 — Trigger Release
```powershell
gh workflow run release.yml --ref main
# Check status:
gh run list --workflow=release.yml --limit 1
```
