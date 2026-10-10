---
title: MCP Server
---

# MCP Server

The Musea MCP server lets an MCP-compatible assistant read your project's component metadata:
props, emitted events, variants, and design tokens. Use it to discover components or draft
examples from their actual APIs. It can also analyze props and emits directly from a Vue SFC.

[Install the package](#installation), [connect your MCP client](#setup), then try a concrete
request such as “List the variants for our Button component.” For a component registry, start
with the [Musea guide](../guide/musea.md) and add art files. The server is experimental; check the
[package support tiers](../stability.md#package-support-tiers) before relying on its API.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then run
this command in the project that contains your components:

```bash
vp install -D @vizejs/musea-mcp-server
```

The installed binary is `musea-mcp`. It accepts the project root as a positional argument:

```bash
vp exec musea-mcp /absolute/path/to/project
```

The process waits for an MCP client over stdio; startup messages go to stderr. Stop this manual
run before connecting your client. If you omit the root, the server uses `MUSEA_PROJECT_ROOT`,
then its working directory. The client examples below set the root explicitly.

## Setup

### With Claude Code

From your project directory, register the installed server:

```bash
claude mcp add --transport stdio --scope project vize-musea -- \
  vp -C "$PWD" exec musea-mcp "$PWD"
claude mcp get vize-musea
```

Project scope writes `.mcp.json` in your project root. Open Claude Code and use `/mcp` to
check the connection and approve the project server when prompted. See the
[official Claude Code MCP instructions](https://code.claude.com/docs/en/mcp#project-scope)
for scopes and connection status.

You can also add this entry to `.mcp.json` manually. Replace both project paths with your
absolute project directory; preserve any existing entries in `mcpServers`:

```json
{
  "mcpServers": {
    "vize-musea": {
      "type": "stdio",
      "command": "vp",
      "args": ["-C", "/absolute/path/to/project", "exec", "musea-mcp", "/absolute/path/to/project"]
    }
  }
}
```

The `-C` argument lets `vp exec` find your installed dependency, while the final argument
selects the files Musea reads. If team members use different directories, adapt the paths
using the [documented environment-variable expansion](https://code.claude.com/docs/en/mcp#environment-variable-expansion-in-mcp-json).

### With Claude Desktop

Open Developer settings, choose **Edit Config**, and add an entry to
`claude_desktop_config.json`. Replace `command` with the absolute path to your `vp` executable
and both project paths with your project directory. Preserve other server entries:

```json
{
  "mcpServers": {
    "vize-musea": {
      "command": "/absolute/path/to/vp",
      "args": ["-C", "/absolute/path/to/project", "exec", "musea-mcp", "/absolute/path/to/project"]
    }
  }
}
```

Fully quit and restart Claude Desktop, then inspect the server connection and tools in Developer
settings. The [official local-server guide](https://modelcontextprotocol.io/docs/develop/connect-local-servers)
covers config locations and logs. An absolute executable path also avoids depending on the
desktop application's shell `PATH`.

### With Other AI Assistants

Configure a local stdio server using your client's configuration format. Use the same executable
and arguments as the Desktop example, including both absolute project paths.

## Use Cases

Replace the component names in these requests with names from your own project.

### Component Discovery

> “What button components do we have? Show me the variants for VFButton.”

Use `search_components` to find candidates, then `get_component` to inspect their metadata,
props, and variants before choosing one.

### Code Generation

> “Create a form with our VFInput and VFTextarea components, including validation error states.”

Ask the assistant to read the component details first and use the returned prop names and
variant templates. Review the draft and run your project's checks before using it.

### API Reference

> “What props does VFNameBadgePreview accept? What are the valid values for user-role?”

`analyze_component` returns statically extracted props and emits from the resolved Vue source.
When metadata is missing, inspect that source rather than inventing an API.

### Documentation Assistance

> “Write documentation for our SponsorGrid component based on its props and variants.”

`generate_docs` produces a Markdown draft for a component; `generate_catalog` covers the
registry. Review the draft against the component source and intended usage.

## Capabilities

### Component Discovery

- `list_components` lists registered art files with category, tags, status, and variant names.
- `search_components` finds matches by title, description, category, tags, or component name.
- `get_component` returns metadata, variants, source analysis, palette data, and resource links.
- `recommend_components` ranks candidates for a described UI task.

### Component API

- `analyze_component` reports props, defaults, required status, and emitted events.
- `get_palette` infers prop controls, options, and available defaults.
- Component source resources let the assistant inspect APIs beyond the analysis response.

### Story Information

- `get_variant` returns a variant's template, metadata, and default status.
- `generate_variants` returns an art-file draft from a Vue component.
- `generate_csf` returns a Storybook CSF draft from an art file.

### Design Tokens

- `get_tokens` reads configured design tokens as JSON or Markdown.
- `search_tokens` searches names, category paths, values, and descriptions.
- Set `--tokens-path tokens.json` or `MUSEA_TOKENS_PATH` to select a file or directory inside
  your project. Otherwise the server searches `tokens/`, `design-tokens/`, and `style-dictionary/`.

## What is MCP?

The Model Context Protocol connects AI assistants to tools and data. Musea exposes component
metadata and source resources through that protocol, so an assistant can inspect your project's
actual components before answering.

## How It Works

```text
AI Assistant
  ↕ MCP (JSON-RPC over stdio)
@vizejs/musea-mcp-server
  ↕ Reads art files and component sources
Your Project (*.art.vue files + components)
```

The server scans the selected root for `*.art.vue` files, excluding `node_modules/` and `dist/`,
and parses them through the native binding. Tools and resources expose their metadata and linked
component sources on request. Nonempty scan results are cached for five seconds, so a newly
added art file may appear on the next scan.
