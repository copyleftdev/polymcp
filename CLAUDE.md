# polygon-mcp

MCP server providing Polygon.io financial data API as tools for AI assistants.

## ABSOLUTE REQUIREMENTS

**These are non-negotiable. Violating these is a failure condition.**

### No Tutorial Comments

- NEVER add comments that explain what code does
- Code MUST be self-documenting through clear naming
- Comments are ONLY for: `TODO`, `FIXME`, `SAFETY:`, doc comments (`///`), or non-obvious "why"
- Delete any tutorial comments you encounter

```rust
// BAD - tutorial comment
let client = Client::new(); // Create a new HTTP client

// BAD - explaining what
// Parse the JSON response and extract the results
let results = response.json::<ApiResponse>()?.results;

// GOOD - no comment needed, code is clear
let polygon = PolygonClient::from_env()?;
let aggregates = polygon.get_aggregates("AAPL", params).await?;

// GOOD - explains WHY, not what
// Polygon returns timestamps in milliseconds, MCP expects ISO 8601
let timestamp = Utc.timestamp_millis_opt(ms).single().unwrap();
```

### Embrace the Expert Skills

You have access to skills from world-class developers. USE THEM:

- **turon-api-design**: For ALL public APIs and tool interfaces — user-first design, builders, type-state
- **matsakis-ownership-mastery**: For lifetime management in responses, borrowed data, avoiding clones
- **bos-concurrency-rust**: For async transport, rate limiting, concurrent request handling

Do not write generic Rust. Write Rust as these experts would.

### Branch Management Workflow

**YOU MUST FOLLOW THIS EXACTLY FOR EVERY ISSUE:**

1. **Start clean**: `git checkout main && git pull origin main`
2. **Create branch**: `git checkout -b feat/POLYMCP-XXX-short-description`
3. **Implement**: Work through ALL acceptance criteria
4. **Verify**: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
5. **Commit**: `git add -A && git commit -m "feat(component): POLYMCP-XXX - title"`
6. **Push**: `git push -u origin feat/POLYMCP-XXX-short-description`
7. **Create PR**: `gh pr create --fill`
8. **Self-review**: Read the diff, verify AC compliance
9. **Merge**: `gh pr merge --squash --delete-branch`
10. **Return to main**: `git checkout main && git pull origin main`

**NEVER:**

- Leave PRs open/lingering
- Work on multiple issues simultaneously
- Skip the self-review step
- Merge without all tests passing

## Architecture

**MCP server exposing Polygon.io REST API as tools.**

- **Protocol**: MCP v2025-11-25 over stdio (JSON-RPC 2.0)
- **HTTP Client**: `reqwest` with rate limiting and retries
- **Async Runtime**: `tokio`
- **Target**: 132 Polygon.io endpoints as MCP tools

### Key Components

```
src/
├── lib.rs              # Public API
├── mcp/                # MCP protocol implementation
│   ├── types.rs        # MCP types (Tool, Resource, etc.)
│   ├── jsonrpc.rs      # JSON-RPC 2.0 handling
│   ├── server.rs       # MCP server lifecycle
│   └── transport.rs    # stdio transport
├── polygon/            # Polygon.io client
│   ├── client.rs       # HTTP client with auth/rate limiting
│   ├── error.rs        # Typed errors
│   └── types.rs        # API response types
└── tools/              # MCP tool implementations
    ├── stocks/         # Stocks API tools
    ├── options/        # Options API tools
    ├── forex/          # Forex API tools
    ├── crypto/         # Crypto API tools
    ├── indices/        # Indices API tools
    └── reference/      # Reference data tools
```

## Commands

```bash
cargo build --release          # Build optimized binary
cargo test                     # Run test suite
cargo clippy -- -D warnings    # Lint (treat warnings as errors)
cargo fmt --check              # Format check

# Issue sync
cd scripts && GITHUB_TOKEN=$(gh auth token) .venv/bin/python -m sync_issues --dry-run
```

## Code Style

- Rust 2024 edition
- `thiserror` for library errors — no `.unwrap()` in library code
- `serde` for all JSON serialization
- Group imports: std, external, crate, super/self
- Doc comments on all public items with examples

## Specs

- **MCP Protocol**: `specs/mcp/schema.json` (v2025-11-25)
- **Polygon.io API**: `polygon-stocks-openapi.json` (OpenAPI 3.0.3, 132 endpoints)

## Issue Workflow

Issues are defined as JSON in `.github/issues/`. Each has:

- **Acceptance Criteria**: Given/When/Then format
- **Technical Context**: Crates, files, MCP methods, Polygon endpoints
- **Dependencies**: What must be done first

To implement: `/plan-issue POLYMCP-XXX` then `/implement-issue POLYMCP-XXX`

## DO NOT

- Add `'static` to silence borrow checker
- Use `Rc<RefCell<T>>` as first resort
- Clone to avoid ownership issues without understanding why
- Swallow errors silently
- **Add tutorial comments** (see ABSOLUTE REQUIREMENTS)
- Leave PRs open or work on multiple issues at once
- Skip any step in the branch management workflow
- Use `.unwrap()` in library code
- Ignore rate limits from Polygon.io API
