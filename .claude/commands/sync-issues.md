# Sync Issues to GitHub

Sync polygon-mcp issues to GitHub.

```bash
cd /home/ops/Project/polymcp/scripts
GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues $ARGUMENTS
```

## Common Flags

- `--dry-run` — Preview without changes
- `--force` — Re-sync all issues
- `--status` — Show sync state
- `--reset` — Clear state file

## Setup (first time)

```bash
cd /home/ops/Project/polymcp/scripts
python -m venv .venv
.venv/bin/pip install -e .
```
