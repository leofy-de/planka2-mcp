<!--
PR title must follow Conventional Commits — it becomes the squash-merge commit
and feeds release-please. See CONTRIBUTING.md for allowed types.

Examples:
  feat: add list_attachments tool
  fix: handle 404 from /api/cards when card was deleted
  docs: clarify Cursor mcp.json example
-->

## Summary

<!-- 1–3 bullets describing what changed and why. -->

## Test plan

<!-- Bulleted checklist of how you verified this. Local Docker build is required for any binary change. -->

- [ ] `docker build -t planka-mcp:test .` succeeds
- [ ] `./planka-mcp --dump-tools` lists the expected tools
- [ ] Manually exercised the change through an MCP client (Claude Code / Inspector / …)

## Tool catalog

<!-- If this PR adds, removes, or modifies a tool, confirm: -->

- [ ] N/A — no tool surface change
- [ ] Tool schema additions/changes are documented in tool description (LLM-readable)
- [ ] Destructive operations are NOT marked with `programmatic_annotations()`
