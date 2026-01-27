Plan implementation for polygon-mcp issue POLYMCP-$ARGUMENTS.

## Steps

### 1. Load Issue Spec
```bash
cat .github/issues/stories/*/POLYMCP-$ARGUMENTS*.json 2>/dev/null || \
cat .github/issues/epics/*.json | jq 'select(.id == "POLYMCP-$ARGUMENTS")'
```

### 2. Check Dependencies
```bash
cat .github/issues/_index.json | jq '.dependency_graph["POLYMCP-$ARGUMENTS"]'
```

If any dependencies are not `done`, list them and STOP.

### 3. Analyze Technical Context
- List files to create/modify
- List crates needed
- Review MCP methods involved
- Review Polygon endpoints involved
- Note any interfaces to implement

### 4. Map Acceptance Criteria to Tests
For each AC, describe:
- Test function name
- Arrange (Given)
- Act (When)
- Assert (Then)

### 5. Identify Risks & Apply Skills
- Complex lifetime scenarios → apply **matsakis** skill
- Async/concurrency requirements → apply **bos** skill
- API design decisions → apply **turon** skill

### 6. Estimate
- Story points (1-13)
- Confidence (high/medium/low)
- Blockers or open questions

### 7. Output Implementation Plan
Structured checklist ready for `/implement-issue`
