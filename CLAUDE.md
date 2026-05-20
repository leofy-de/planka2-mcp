# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## See also

- [AGENTS.md](AGENTS.md) — tool-design philosophy + links to MCP best-practice sources (Phil Schmid, Anthropic, the MCP spec). Read this before adding or modifying a tool.
- [CONTRIBUTING.md](CONTRIBUTING.md) — Conventional Commits rules, PR process, MCP Inspector recipe.
- [Live tool catalog](https://leofy-de.github.io/planka2-mcp/) — regenerated from the binary on every release.

## Project Overview

This is a Rust MCP (Model Context Protocol) server that integrates with self-hosted Planka kanban instances. The server exposes Planka features as MCP tools via JSON-RPC 2.0 over stdin/stdout.

## Build and Run Commands

**IMPORTANT: `cargo` is not installed in this environment. Always use Docker to build.**

```bash
# Build using Docker (produces ./planka-mcp binary)
docker build -t planka-mcp-build . && \
  docker create --name planka-mcp-extract planka-mcp-build && \
  docker cp planka-mcp-extract:/usr/local/bin/planka-mcp ./planka-mcp && \
  docker rm planka-mcp-extract

# Run (requires environment variables)
./planka-mcp
```

After any code change: rebuild with Docker and extract the binary before testing or committing.

To inspect the live tool catalog without booting an MCP client:

```bash
./planka-mcp --dump-tools | jq '.[].name'
```

## Environment Variables

Required configuration (set before running):
- `PLANKA_URL` - Planka instance URL (e.g., `https://kanban.local`)
- Authentication (one of):
  - `PLANKA_TOKEN` - Bearer token (preferred)
  - `PLANKA_EMAIL` + `PLANKA_PASSWORD` - Login credentials
- `PLANKA_DEFAULT_CARD_TYPE` - Default card type for `create_card` (optional, defaults to `"project"`). Valid values per the Planka API are `"project"` and `"story"`. There is no `"task"` type.

## Architecture

```
src/
  main.rs              # Entry point, tokio runtime, JSON-RPC loop
  mcp/
    mod.rs
    server.rs          # JSON-RPC server (stdin/stdout)
    types.rs           # MCP protocol types (JsonRpcRequest, JsonRpcResponse)
  planka/
    mod.rs
    client.rs          # HTTP client for Planka REST API
    types.rs           # Data models (Project, Board, Card, List)
  tools/
    mod.rs             # Tool definitions and handlers (all tools in one file)
```

**Key patterns:**
- Async throughout using Tokio runtime
- HTTP via reqwest (async)
- JSON-RPC 2.0 protocol over stdin/stdout
- `PlankaClient` struct abstracts all Planka API calls
- `PlankaError` enum handles HTTP, config, and serialization errors
- Each MCP tool is a separate module in `tools/`

## MCP Tools

10 tools are registered in `list_tools()` (see `src/tools/mod.rs`):

| Method | Description |
|--------|-------------|
| `list_projects` | All projects with board counts. |
| `list_board_summary` | Board overview: lists with card counts. |
| `find_cards` | Search cards on a board by name and/or list. Compact, image-stripped. |
| `get_card` | Single card (title, sanitized description, list_id, tasks). |
| `get_card_context` | One-shot resolver: card + project + board + sibling lists + labels + members. Use first when given a card URL. |
| `create_card` | Create a card (`list_id`, `name`, optional `description`, optional `card_type`). |
| `update_card` | Update a card (`card_id`, optional `name`, optional `description`). |
| `move_card` | Move a card. Accepts `list_id` OR `list_name`. |
| `add_comment` | Post a Markdown comment on a card. |
| `delete_card` | Delete a card. **Destructive** — excluded from programmatic calling (`annotations: None`). |

The test at the bottom of `src/tools/mod.rs` asserts both the count (10) and the name list; keep it in sync when adding tools.

## Adding New Tools

See [CONTRIBUTING.md §4](CONTRIBUTING.md) for the full recipe. Short version:

1. Add HTTP method to `PlankaClient` in `src/planka/client.rs`
2. Add any new types to `src/planka/types.rs`
3. In `src/tools/mod.rs`:
   - Add `Tool` entry in `list_tools()` (use `programmatic_annotations()` unless destructive)
   - Add match arm in `call_tool()`
   - Add handler function and args struct
4. Update the test count + name list at the bottom of `src/tools/mod.rs`.

Read [AGENTS.md](AGENTS.md) for the design patterns (compact responses, one-shot context tools, tolerant inputs, destructive-ops gating).

## Planka API Endpoints

Find all Endpoints in the swagger docs: https://plankanban.github.io/planka/swagger-ui/

- `POST /api/access-tokens` - Login (email/password auth)
- `GET /api/projects` - List projects
- `GET /api/projects/{projectId}` - Get project with boards
- `POST /api/projects/{projectId}/boards` - Create board
- `GET /api/boards/{boardId}` - Get board with lists and cards
- `POST /api/boards/{boardId}/lists` - Create list
- `POST /api/lists/{listId}/cards` - Create card
- `PATCH /api/cards/{cardId}` - Update/move card
- `DELETE /api/cards/{cardId}` - Delete card
- `DELETE /api/lists/{listId}` - Delete list

## Constraints

- Local-only: stdin/stdout transport (or TCP bound to 127.0.0.1 only)
- Never log secrets (tokens, passwords, credential URLs)
- All inputs treated as untrusted; validate and return proper JSON-RPC errors
- JSON-RPC error codes: -32602 (invalid params), -32603 (internal error)
