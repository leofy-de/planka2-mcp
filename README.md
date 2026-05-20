# planka-mcp

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-success.svg)](LICENSE)
[![Docker image](https://img.shields.io/badge/ghcr.io-planka2--mcp-blue?logo=docker)](https://github.com/leofy-de/planka2-mcp/pkgs/container/planka2-mcp)
[![Build](https://github.com/leofy-de/planka2-mcp/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/leofy-de/planka2-mcp/actions/workflows/docker-publish.yml)
[![Tool catalog](https://img.shields.io/badge/tool%20catalog-live-success)](https://leofy-de.github.io/planka2-mcp/)
[![MCP](https://img.shields.io/badge/Model%20Context%20Protocol-compatible-7c3aed)](https://modelcontextprotocol.io/)

A [Model Context Protocol](https://modelcontextprotocol.io/) server for
[Planka v2](https://planka.app/)
([github.com/plankanban/planka](https://github.com/plankanban/planka)) kanban
boards, written in Rust. Designed for LLM use: compact responses,
outcome-focused tools, no context bloat.

<p align="center">
  <a href="https://leofy-de.github.io/planka2-mcp/">
    <img alt="Open the live demo and tool catalog"
         src="https://img.shields.io/badge/%E2%96%B6%20%20Live%20Demo%20%26%20Tool%20Catalog-Open-7c3aed?style=for-the-badge&logoColor=white" />
  </a>
</p>

<p align="center">
  <i>Every tool, parameter, and JSON schema — regenerated from the binary on each release, so it never drifts.<br>
  👉 <a href="https://leofy-de.github.io/planka2-mcp/"><b>leofy-de.github.io/planka2-mcp</b></a></i>
</p>

---

## Table of contents

- [TL;DR](#tldr)
- [Configure your MCP client](#configure-your-mcp-client)
- [Authentication](#authentication)
- [Available tools](#available-tools)
- [Typical workflow](#typical-workflow)
- [Troubleshooting](#troubleshooting)
- [Install from source](#install-from-source)
- [Build the Docker image locally](#build-the-docker-image-locally)
- [Contributing](#contributing)
- [License](#license)

---

## TL;DR

```jsonc
// In your MCP client config (Claude Code, Claude Desktop, Cursor, Cline, …)
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

That's it. Restart your MCP client → the server appears as `planka`. Ten
tools become available — see the [live catalog](https://leofy-de.github.io/planka2-mcp/)
or the [Available tools](#available-tools) section below.

**Prerequisites:** Docker installed and running.

---

## Configure your MCP client

The exact config file differs per client; the inner `mcpServers` block is the
same.

<details>
<summary><b>Claude Code</b> — <code>~/.claude/mcp.json</code> (global) or <code>.mcp.json</code> (per-project)</summary>

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

Verify with `/mcp` inside Claude Code — `planka` should appear with 10 tools.

> Keep credentials out of version control. For a shared per-project
> `.mcp.json`, prefer env-var substitution or a dedicated service account.

</details>

<details>
<summary><b>Claude Desktop</b> — <code>claude_desktop_config.json</code></summary>

Location:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`

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

Quit and reopen Claude Desktop after editing.

</details>

<details>
<summary><b>Cursor</b> — <code>.cursor/mcp.json</code> (per-project) or <code>~/.cursor/mcp.json</code> (global)</summary>

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

</details>

<details>
<summary><b>Cline (VS Code extension)</b></summary>

Open the Cline panel → ⚙️ → **MCP Servers** → **Edit Configuration**, then
paste the same `mcpServers` block as above.

</details>

<details>
<summary><b>Using email/password instead of a token</b></summary>

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

The server caches the access token in-process after the first login.

</details>

---

## Authentication

Set `PLANKA_URL` plus **one** authentication option.

| Variable                              | Required | Description                                                                     |
| ------------------------------------- | -------- | ------------------------------------------------------------------------------- |
| `PLANKA_URL`                          | yes      | Base URL of your Planka instance, e.g. `https://kanban.example.com` (no trailing slash). |
| `PLANKA_TOKEN`                        | one of   | Bearer token (preferred — no repeated logins, no password on disk).             |
| `PLANKA_EMAIL` + `PLANKA_PASSWORD`    | one of   | Email/password credentials. Token is cached in-process.                         |
| `PLANKA_DEFAULT_CARD_TYPE`            | no       | `"project"` (default) or `"story"`. Used by `create_card` when no type is passed. |

How to mint a `PLANKA_TOKEN`: in Planka, go to your user menu → **API access
tokens** → create one. Keep it secret; treat it like a password.

---

## Available tools

The [live catalog](https://leofy-de.github.io/planka2-mcp/) is the canonical
reference, with every parameter and JSON schema. Quick summary:

| Tool                  | Effect                                                                                              |
| --------------------- | --------------------------------------------------------------------------------------------------- |
| `list_projects`       | All projects, with board counts.                                                                    |
| `list_board_summary`  | Board overview — lists with card counts. Use to find list IDs.                                      |
| `find_cards`          | Search cards by name and/or list. Returns compact, image-stripped summaries.                        |
| `get_card`            | Single card: title, sanitised description, list_id, task checklist.                                 |
| `get_card_context`    | **One-shot resolver** for a card URL — card + project + board + sibling lists + labels + members.   |
| `create_card`         | Create a card in a list.                                                                            |
| `update_card`         | Update title or description (just `card_id` plus what you want to change).                          |
| `move_card`           | Move a card. Accepts `list_id` **or** `list_name` (resolved on the card's own board).               |
| `add_comment`         | Post a Markdown comment on a card.                                                                  |
| `delete_card`         | Delete a card permanently. **Destructive** — excluded from programmatic calling.                    |

All tools except `delete_card` support
[programmatic tool calling](https://www.anthropic.com/engineering/advanced-tool-use)
(`allowed_callers: ["code_execution_20250825"]`). `delete_card` is gated
behind a user-visible decision turn on purpose.

Card descriptions returned by `get_card`, `get_card_context`, and `find_cards`
are sanitised: inline `data:image/...;base64,...` blobs become
`[image omitted]`, long base64 runs become `[binary omitted]`, and the result
is capped at 1500 characters. `update_card` writes are not modified.

### Typical workflow

**From a card URL** (the fast path):

```
get_card_context(card_id from URL)
  → add_comment(card_id, text)              # or
  → move_card(card_id, list_name="Done")    # or
  → update_card(card_id, …)
```

**Open-ended discovery:**

```
list_projects
  → list_board_summary(board_id)
    → find_cards(board_id, query="…")
      → get_card(card_id)
      → move_card(card_id, list_id)
```

---

## Troubleshooting

<details>
<summary>The <code>planka</code> server doesn't appear in <code>/mcp</code></summary>

- Quit and fully relaunch your MCP client after editing the config.
- Run `docker run --rm -i ghcr.io/leofy-de/planka2-mcp:latest --version`
  manually — it should print `planka-mcp 0.x.y`. If Docker can't pull the
  image, the client won't either.
- Check the client's log for the MCP server stderr — every Rust error this
  server emits goes to stderr.

</details>

<details>
<summary>401 / 403 from Planka</summary>

- If using `PLANKA_TOKEN`, make sure the token belongs to a user that can see
  the boards you're trying to read.
- Tokens expire when the user changes their password. Mint a fresh one.
- For email/password auth, check that **2FA is disabled** for that account —
  the login endpoint does not currently negotiate a second factor.

</details>

<details>
<summary><code>PLANKA_URL</code> issues</summary>

- No trailing slash: `https://kanban.example.com`, **not**
  `https://kanban.example.com/`.
- The URL must be reachable from inside the Docker container. If Planka runs
  on the same host, use the host's LAN IP (or `host.docker.internal` on
  macOS/Windows) rather than `localhost`.
- Self-signed certificates: not currently supported. Use a real cert or run
  the binary natively (see [Install from source](#install-from-source)).

</details>

<details>
<summary>"Project Manager role required" when creating boards</summary>

The Planka API only lets Project Managers create boards. Either elevate the
account behind your `PLANKA_TOKEN`, or create the board manually in the
Planka UI.

</details>

<details>
<summary>I want to see the raw MCP traffic</summary>

Use the official [MCP Inspector](https://github.com/modelcontextprotocol/inspector):

```bash
npx @modelcontextprotocol/inspector \
  docker run --rm -i \
    -e PLANKA_URL=https://kanban.example.com \
    -e PLANKA_TOKEN=your-bearer-token \
    ghcr.io/leofy-de/planka2-mcp:latest
```

It speaks JSON-RPC over stdio and shows every request, response, and notification.

</details>

<details>
<summary>What's <code>--dump-tools</code>?</summary>

A built-in CLI flag that prints the JSON tool catalog to stdout and exits.
Useful for inspecting the schema without booting an MCP client:

```bash
docker run --rm ghcr.io/leofy-de/planka2-mcp:latest --dump-tools | jq '.[].name'
```

No Planka credentials needed for this path.

</details>

---

## Install from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
# Via cargo install (builds from GitHub)
cargo install --git https://github.com/leofy-de/planka2-mcp

# Or clone and build
git clone https://github.com/leofy-de/planka2-mcp
cd planka2-mcp
cargo build --release
./target/release/planka-mcp --version
```

Then point your MCP client at the binary:

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

## Build the Docker image locally

```bash
docker build -t planka-mcp:local .

# Then in mcp.json:
# "args": ["run", "--rm", "-i", "-e", "PLANKA_URL", "-e", "PLANKA_TOKEN", "planka-mcp:local"]
```

---

## Contributing

Contributions welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) for the build
loop and Conventional Commits rules, and [AGENTS.md](AGENTS.md) for the
tool-design philosophy and links to the MCP best-practice sources we follow.

Release history: [CHANGELOG.md](CHANGELOG.md) (auto-generated from
Conventional Commits via
[release-please](https://github.com/googleapis/release-please)).

---

## License

Released into the **public domain** under [The Unlicense](LICENSE) — no
rights reserved. Use it however you want, no attribution required.
