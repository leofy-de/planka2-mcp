# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust MCP (Model Context Protocol) server that integrates with self-hosted Planka kanban instances. The server exposes Planka features as MCP tools via JSON-RPC 2.0 over stdin/stdout.

## Build and Run Commands

```bash
# Build
cargo build --release

# Run (requires environment variables)
./target/release/planka-mcp

# Development build and run
cargo run

# Run tests
cargo test

# Run a single test
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Environment Variables

Required configuration (set before running):
- `PLANKA_URL` - Planka instance URL (e.g., `https://kanban.local`)
- Authentication (one of):
  - `PLANKA_TOKEN` - Bearer token (preferred)
  - `PLANKA_EMAIL` + `PLANKA_PASSWORD` - Login credentials

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

| Method | Description |
|--------|-------------|
| `list_projects` | List all projects (no params) |
| `list_boards` | List boards in a project (`project_id` param) |
| `list_lists` | List lists/columns on a board (`board_id` param) |
| `list_cards` | List cards on a board (`board_id` param) |
| `create_board` | Create board (`project_id`, `name`) - requires Project Manager role |
| `create_list` | Create list (`board_id`, `name`) |
| `create_card` | Create card (`list_id`, `name`, optional `description`) |
| `update_card` | Update card (`card_id`, optional `name`, optional `description`) |
| `move_card` | Move card (`card_id`, `list_id`, optional `position`) |
| `delete_card` | Delete card (`card_id`) |
| `delete_list` | Delete list and cards (`list_id`) |

## Adding New Tools

1. Add HTTP method to `PlankaClient` in `src/planka/client.rs`
2. Add any new types to `src/planka/types.rs`
3. In `src/tools/mod.rs`:
   - Add `Tool` entry in `list_tools()`
   - Add match arm in `call_tool()`
   - Add handler function and args struct

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
