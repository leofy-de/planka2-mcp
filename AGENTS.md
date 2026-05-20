# AGENTS.md — Tool design guide for planka-mcp

This file is the source of truth for **how tools should be designed and
written** in this repository. It is written for AI coding agents (Claude Code,
Cursor, Codex, …) and human contributors who want to add a tool that "feels
right" to LLM callers.

If you are looking for build instructions, see
[CONTRIBUTING.md](CONTRIBUTING.md). If you want a live, browsable catalog of
the tools this server exposes today, see
<https://leofy-de.github.io/planka2-mcp/>.

---

## What this server is

A Rust MCP server that exposes a small, opinionated set of tools for
[Planka v2](https://planka.app/). The tools are tuned for **LLM use**, not
for general API completeness. We deliberately do not 1:1 mirror the Planka
REST API — instead we pick a few high-leverage tools and shape their
inputs/outputs so an agent can solve realistic kanban tasks in as few turns as
possible.

---

## MCP best practices we follow

These external sources shaped the tool design in this repo. If you are adding
a tool, skim them first.

- **Official MCP docs** — <https://modelcontextprotocol.io/>
  Concepts, lifecycle, transport, and capabilities.
- **MCP specification** — <https://spec.modelcontextprotocol.io/>
  The wire format — JSON-RPC 2.0 over stdio in our case.
- **Phil Schmid, "MCP Best Practices"** —
  <https://www.philschmid.de/mcp-best-practices>
  The single best overview on tool-shape decisions.
- **Anthropic, "Writing tools for AI agents"** —
  <https://www.anthropic.com/engineering/writing-tools-for-agents>
  Why outcome-focused descriptions and compact outputs matter.
- **Anthropic, "Advanced tool use"** —
  <https://www.anthropic.com/engineering/advanced-tool-use>
  Programmatic tool calling, `allowed_callers` annotations, batching.
- **Anthropic, MCP introduction** —
  <https://docs.claude.com/en/docs/mcp>
  The Claude-side view of MCP.

---

## Concrete patterns this codebase applies

Each pattern below is backed by a code reference so you can see it in action.

### 1. Compact, outcome-focused responses

LLMs pay context for every byte we return. We aggressively trim:

- Card descriptions are sanitized in
  [`src/planka/sanitize.rs`](src/planka/sanitize.rs): inline
  `data:image/...;base64,...` blobs are replaced with `[image omitted]`, long
  base64 runs become `[binary omitted]`, and the result is capped at **1500
  characters**.
- `find_cards` returns only `{id, name, list}` — never the full description.
- Listing tools include counts and IDs but skip nested timestamps unless they
  matter.

When you add a tool, pick the smallest response shape that still lets the LLM
make a useful next decision.

### 2. One-shot context tools

Discovery chains burn turns. `get_card_context` in
[`src/tools/mod.rs`](src/tools/mod.rs) is the canonical example: given a card
ID from a Planka URL, it returns the card **plus** its project, board, sibling
lists, board labels, board members, and the card's own tasks — everything you
need to comment/move/update without further calls.

When a workflow has a natural "starting handle" (a URL, an ID), prefer a
single tool that resolves the whole context over forcing the LLM to chain
list/get calls.

### 3. Tolerant inputs

`move_card` accepts either `list_id` **or** `list_name`. If the LLM only
knows the column is called "Done", it shouldn't have to call
`list_board_summary` first. The handler resolves names against the card's own
board and returns a helpful error if the name is ambiguous.

Avoid forcing the LLM to chase IDs through extra calls when a name lookup is
unambiguous.

### 4. Destructive ops are gated

`delete_card` is the only tool whose `annotations` is `None`. Every other
tool sets

```rust
annotations: Some(ToolAnnotations {
    allowed_callers: Some(vec!["code_execution_20250825".to_string()]),
})
```

…enabling [programmatic tool
calling](https://www.anthropic.com/engineering/advanced-tool-use). We
explicitly opt destructive operations **out** so they always require a
user-facing decision turn.

If you add a tool that permanently destroys data (delete, force-overwrite,
mass-move), follow the `delete_card` pattern and set `annotations: None`.

### 5. Stdout is sacred

The MCP transport is JSON-RPC over stdio. Anything we accidentally write to
stdout breaks the client. See [`src/main.rs`](src/main.rs) — `tracing` is
configured with `.with_writer(std::io::stderr)`. Never `println!` inside a
tool handler. The only exception is the explicit `--dump-tools` mode, which
is run as a CLI, not as a server.

### 6. No secrets in logs

Even on `RUST_LOG=debug`, do not log:

- The Planka token
- Email or password
- The full `Authorization` header

Log the request method and path; redact everything else.

### 7. Errors return structured JSON-RPC

Tool handlers return `ToolCallResult::error(msg)` rather than panicking. The
`msg` is shown to the LLM, so phrase it as a corrective hint: tell the model
what to do differently ("Pass a `card_id`, not a card URL.") rather than just
restating the failure.

---

## Workflow when adding a tool

1. Confirm the Planka API supports it (Swagger:
   <https://plankanban.github.io/planka/swagger-ui/>).
2. Sketch the tool shape on paper. Apply patterns 1–4 above:
   - What's the smallest useful response?
   - Can a single tool short-circuit a multi-call chain?
   - Are any inputs more humanely expressed as names than IDs?
   - Is it destructive?
3. Implement it following the four-step recipe in
   [CONTRIBUTING.md §4](CONTRIBUTING.md).
4. Verify with `./planka-mcp --dump-tools` and the MCP Inspector recipe in
   CONTRIBUTING.md.
5. Open a PR with a `feat:` title.
