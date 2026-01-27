Implement polygon-mcp issue POLYMCP-$ARGUMENTS with full branch workflow.

**ABSOLUTE REQUIREMENTS:**
- NO tutorial comments. Code must be self-documenting.
- Follow branch workflow EXACTLY. No lingering PRs.
- Apply relevant skills: matsakis (ownership), bos (concurrency), turon (API design)

## Workflow

### 1. Setup
```bash
git checkout main && git pull origin main
git checkout -b feat/POLYMCP-$ARGUMENTS-short-description
```

### 2. Understand
- Read spec: `cat .github/issues/stories/*/POLYMCP-$ARGUMENTS*.json 2>/dev/null || cat .github/issues/epics/*.json | jq 'select(.id == "POLYMCP-$ARGUMENTS")'`
- Check deps: `cat .github/issues/_index.json | jq '.dependency_graph["POLYMCP-$ARGUMENTS"]'`
- If deps incomplete, STOP and report which issues must be done first

### 3. Implement
- Create files from `technical_context.files`
- Use crates from `technical_context.crates`
- Implement EVERY acceptance criterion
- Write tests for each AC (Given/When/Then → Arrange/Act/Assert)
- Respect `performance_constraints` if present
- NO TUTORIAL COMMENTS

### 4. Verify
```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```
All must pass before proceeding.

### 5. Commit & PR
```bash
git add -A
git commit -m "feat(component): POLYMCP-$ARGUMENTS - [title from spec]"
git push -u origin feat/POLYMCP-$ARGUMENTS-short-description
gh pr create --fill
```

### 6. Self-Review
- Read the PR diff
- Verify each AC is satisfied
- Check for tutorial comments (delete any found)
- Confirm tests cover edge cases

### 7. Merge & Cleanup
```bash
gh pr merge --squash --delete-branch
git checkout main
git pull origin main
```

### 8. Report completion and await next issue

**FAILURE CONDITIONS:**
- Leaving PR unmerged
- Skipping any verification step
- Adding tutorial comments
- Proceeding with failing tests
- Ignoring performance constraints
