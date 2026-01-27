# pdfvec Scripts

Development scripts for the pdfvec project.

## sync_issues

Syncs JSON issue definitions from `.github/issues/` to GitHub Issues.

### Setup

```bash
cd scripts
python -m venv .venv
.venv/bin/pip install -e .
```

### Usage

```bash
# Preview changes
GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues --dry-run

# Sync all issues
GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues

# Force re-sync even if unchanged
GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues --force

# Check sync status
GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues --status
```

### How It Works

1. Reads issue definitions from `.github/issues/epics/` and `.github/issues/stories/`
2. Computes content hash for each issue
3. Creates or updates GitHub issues that have changed
4. Tracks sync state in `.github/.sync-state.json`
