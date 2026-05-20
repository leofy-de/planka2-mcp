# Contributing to planka-mcp

Thanks for your interest! This guide covers the few things you need to know to
get your change merged smoothly.

- For tool-design philosophy (when to add a tool, what shape its output should
  take, how it interacts with LLMs), read [AGENTS.md](AGENTS.md) first.
- For the live, generated tool catalog, see
  <https://leofy-de.github.io/planka2-mcp/>.

---

## 1. Conventional Commits (required)

The PR title is squash-merged onto `main` and feeds
[release-please](https://github.com/googleapis/release-please), which decides
the next version and writes the CHANGELOG. **The title must follow
[Conventional Commits](https://www.conventionalcommits.org/).**

| Type       | When to use                                         | Bumps   |
| ---------- | --------------------------------------------------- | ------- |
| `feat`     | A new tool, new capability, new MCP surface         | minor   |
| `fix`      | A bug fix                                           | patch   |
| `perf`     | Performance-only change, observable to users        | patch   |
| `refactor` | Code reshuffle with no behaviour change             | none    |
| `docs`     | README, AGENTS, CONTRIBUTING, code comments         | none    |
| `test`     | Adding or fixing tests                              | none    |
| `ci`       | Workflows, action versions, release plumbing        | none    |
| `build`    | Dockerfile, Cargo.toml deps                         | none    |
| `chore`    | Anything else that doesn't fit                      | none    |

Append `!` after the type, or include `BREAKING CHANGE:` in the body, for a
breaking change → triggers a **major** bump.

Subjects start lowercase. The CI job
[`commitlint.yml`](.github/workflows/commitlint.yml) enforces this on every
PR.

Examples:

```
feat: add list_attachments tool
fix: handle 404 from /api/cards when card was deleted
docs: clarify Cursor mcp.json example
feat!: switch get_card to return ISO timestamps
```

---

## 2. Build & test loop

`cargo` is **not** assumed to be installed locally — the canonical build is via
Docker, which matches CI exactly.

```bash
# Build the image
docker build -t planka-mcp-build .

# Extract the binary for local smoke tests
docker create --name planka-mcp-extract planka-mcp-build
docker cp planka-mcp-extract:/usr/local/bin/planka-mcp ./planka-mcp
docker rm planka-mcp-extract

# Verify it lists the expected tools (no credentials needed)
./planka-mcp --dump-tools | jq '.[].name'
```

If you have a local Rust toolchain, `cargo build --release` and `cargo test`
work too — the Dockerfile uses the same flags.

---

## 3. Running the MCP Inspector against your build

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is the
fastest way to manually exercise a change — it speaks the protocol and shows
every request/response.

```bash
# Build a local image
docker build -t planka-mcp:dev .

# Launch the inspector pointed at your local image
npx @modelcontextprotocol/inspector \
  docker run --rm -i \
    -e PLANKA_URL=https://kanban.example.com \
    -e PLANKA_TOKEN=your-bearer-token \
    planka-mcp:dev
```

---

## 4. Adding a new MCP tool

The pattern is consistent across all existing tools — `add_comment` is a good,
small reference.

1. **Add the HTTP method** in [`src/planka/client.rs`](src/planka/client.rs)
   that hits the Planka API endpoint you need.
2. **Add request/response types** in
   [`src/planka/types.rs`](src/planka/types.rs) (use `#[derive(Deserialize)]`
   for responses, `Serialize` for requests).
3. **Register and implement the tool** in
   [`src/tools/mod.rs`](src/tools/mod.rs):
   - Add a `Tool { … }` entry inside `list_tools()`.
   - Use `programmatic_annotations()` unless the tool is destructive (see
     `delete_card` for the opt-out pattern).
   - Add a `match` arm inside `call_tool()` that dispatches to your handler.
   - Add the handler function and a `#[derive(Deserialize)]` args struct.
4. **Update the test** at the bottom of `src/tools/mod.rs` so the tool count
   and name list match.
5. **Confirm the catalog reflects it** by running `./planka-mcp --dump-tools`.

### Tool description checklist

Tool descriptions are read by LLMs every turn — keep them outcome-focused:

- Start with the user-visible result, not the API call.
- Mention required vs optional params and any "magic" behaviour (e.g.
  `move_card` accepting `list_name` as a convenience).
- Flag destructive or irreversible effects.
- Skip implementation detail unless it changes when the LLM should pick the
  tool.

Refer to [AGENTS.md](AGENTS.md) for the linked MCP best-practice sources.

---

## 5. PR process

1. Branch from `main`.
2. Push your changes; open a PR with a Conventional Commits title.
3. CI runs:
   - PR-title lint
   - Docker build & smoke test
4. A maintainer reviews, then **squash-merges** — the squash message becomes
   the commit on `main`, which release-please consumes.
5. release-please keeps an open "release PR" on `main`. Merging that PR cuts a
   release; the existing `docker-publish.yml` workflow ships a versioned image
   to GHCR automatically.

That's it — thanks for contributing!
