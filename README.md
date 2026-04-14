# planka v2 mcp server

A Model Context Protocol (MCP) server for [Planka v2](https://planka.app/) kanban boards, written in Rust.

Optimised for LLM use: compact responses, outcome-focused tools, no context bloat.

## Quick Start (Docker — no local build required)

The recommended way to use planka-mcp is via the pre-built Docker image published to GitHub Container Registry.

### Prerequisites

- Docker installed and running

### 1. Add to your MCP client config

**`~/.claude/mcp.json`** (global, for all projects):

```json
{
  "mcpServers": {
    "planka": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-e", "PLANKA_URL",
        "-e", "PLANKA_TOKEN",
        "ghcr.io/leofy-de/planka2-mcp:latest"
      ],
      "env": {
        "PLANKA_URL": "https://kanban.example.com",
        "PLANKA_TOKEN": "your-bearer-token"
      }
    }
  }
}
```

Or with email/password authentication:

```json
{
  "mcpServers": {
    "planka": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-e", "PLANKA_URL",
        "-e", "PLANKA_EMAIL",
        "-e", "PLANKA_PASSWORD",
        "ghcr.io/leofy-de/planka2-mcp:latest"
      ],
      "env": {
        "PLANKA_URL": "https://kanban.example.com",
        "PLANKA_EMAIL": "user@example.com",
        "PLANKA_PASSWORD": "your-password"
      }
    }
  }
}
```

**`.mcp.json`** (per-project, checked into the repository):

```json
{
  "mcpServers": {
    "planka": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "-e", "PLANKA_URL",
        "-e", "PLANKA_EMAIL",
        "-e", "PLANKA_PASSWORD",
        "ghcr.io/leofy-de/planka2-mcp:latest"
      ],
      "env": {
        "PLANKA_URL": "https://kanban.example.com",
        "PLANKA_EMAIL": "mcp@example.com",
        "PLANKA_PASSWORD": "your-password"
      }
    }
  }
}
```

> Keep credentials out of version control — use environment variable substitution or a secrets manager for shared `.mcp.json` files.

### 2. Restart your MCP client

In Claude Code: run `/mcp` to verify the `planka` server appears.

---

## Authentication

Set **one** of the following:

| Variable | Description |
|---|---|
| `PLANKA_URL` | Base URL of your Planka instance (required) |
| `PLANKA_TOKEN` | Bearer token (preferred, avoids repeated logins) |
| `PLANKA_EMAIL` + `PLANKA_PASSWORD` | Email/password credentials (token is cached in-process) |

---

## Available Tools

| Tool | Description |
|---|---|
| `list_projects` | List all projects. Returns `[{id, name}]`. |
| `list_board_summary` | Board overview with lists and card counts. Use to find list IDs. |
| `find_cards` | Search cards by name or list. Returns compact summaries. |
| `get_card` | Full card detail: description + task checklist. |
| `create_card` | Create a card in a list. |
| `update_card` | Update a card's title or description. |
| `move_card` | Move a card to a different list. |
| `add_comment` | Post a comment on a card (summaries, status updates, notes). |
| `delete_card` | Delete a card permanently. |

All tools except `delete_card` support [programmatic tool calling](https://www.anthropic.com/engineering/advanced-tool-use) (`allowed_callers: ["code_execution_20250825"]`).

### Typical workflow

```
list_projects
  → list_board_summary(board_id)
    → find_cards(board_id, query="...")
      → get_card(card_id)
      → move_card(card_id, list_id)
```

---

## Alternative: Install from source

Requires [Rust](https://rustup.rs/) installed.

```bash
# Via cargo install (builds from GitHub)
cargo install --git https://github.com/leofy-de/planka2-mcp

# Or clone and build
git clone https://github.com/leofy-de/planka2-mcp
cd planka2-mcp
cargo build --release
./target/release/planka-mcp
```

Source-based `mcp.json`:

```json
{
  "mcpServers": {
    "planka": {
      "command": "/path/to/planka-mcp",
      "env": {
        "PLANKA_URL": "https://kanban.example.com",
        "PLANKA_TOKEN": "your-token"
      }
    }
  }
}
```

---

## Building the Docker image locally

```bash
docker build -t planka-mcp:local .

# Then use in mcp.json:
# "args": ["run", "--rm", "-i", "-e", "PLANKA_URL", "-e", "PLANKA_TOKEN", "planka-mcp:local"]
```

---

## Extending

To add new tools:

1. Add HTTP method to `src/planka/client.rs`
2. Add any new types to `src/planka/types.rs`
3. Add tool definition and handler to `src/tools/mod.rs`

---

## License

MIT
