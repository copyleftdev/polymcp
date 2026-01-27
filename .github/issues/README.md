# polygon-mcp Issues

Machine-readable issue definitions for the polygon-mcp project.

## Structure

```
.github/issues/
├── _schema.json      # JSON Schema for issue validation
├── _index.json       # Project overview and dependency graph
├── _labels.json      # GitHub label definitions
├── _milestones.json  # Milestone definitions
├── epics/            # Epic issue definitions
└── stories/          # Story/task definitions
```

## Issue Format

Issues use Given/When/Then acceptance criteria:

```json
{
  "id": "POLYMCP-XXX",
  "title": "Short descriptive title",
  "type": "epic|story|task|bug|spike",
  "status": "draft|ready|in_progress|review|done|blocked",
  "priority": "critical|high|medium|low",
  "acceptance_criteria": [
    {
      "id": "AC-1",
      "given": "Precondition",
      "when": "Action",
      "then": "Expected outcome"
    }
  ],
  "technical_context": {
    "crates": ["tokio", "serde"],
    "files": ["src/lib.rs"],
    "mcp_methods": ["tools/list", "tools/call"],
    "polygon_endpoints": ["/v2/aggs/ticker/{ticker}/range/..."]
  }
}
```

## Syncing to GitHub

```bash
cd scripts
python -m venv .venv
source .venv/bin/activate
pip install -e .

# Preview changes
GITHUB_TOKEN=$(gh auth token) python -m sync_issues --dry-run

# Sync
GITHUB_TOKEN=$(gh auth token) python -m sync_issues
```

## Specs

- **MCP Protocol**: `specs/mcp/schema.json` (v2025-11-25)
- **Polygon.io API**: `polygon-stocks-openapi.json` (OpenAPI 3.0.3, 132 endpoints)
