# @vizejs/musea-mcp-server

MCP (Model Context Protocol) server for Musea design system integration.

## Features

- **AI Integration** - Connect Musea to AI assistants
- **Component Discovery** - Query available components
- **Design Token Access** - Retrieve design system tokens
- **Documentation** - Access component docs via MCP

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the server to your project:

```bash
vp install -D @vizejs/musea-mcp-server
```

## Usage

### With Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server", "--project", "/path/to/project"]
    }
  }
}
```

### Standalone

```bash
vp dlx @vizejs/musea-mcp-server --project ./my-vue-app
```

## MCP Tools

- `analyze_component` - Extract props and emits from a Vue SFC or linked art component
- `get_palette` - Derive an interactive props palette from an art file
- `list_components` - List registered components with status, variants, and resource URIs
- `get_component` - Get full component details, analysis, palette, and docs
- `get_variant` - Get one specific variant
- `search_components` - Search components by title, tags, category, component name, and variants
- `recommend_components` - Rank components by user intent or UI task
- `generate_variants` - Generate an `.art.vue` draft from a component
- `generate_csf` - Convert an art file to Storybook CSF
- `generate_docs` - Generate Markdown docs for a component
- `generate_catalog` - Generate a catalog for the whole design system
- `get_tokens` - Read design tokens
- `search_tokens` - Search tokens without loading the full token tree

## Reproducible Prompting

Musea helps AI assistants stay grounded in real component data, but reusable prompts still need to
be tuned for reproducibility.

A practical loop is:

1. Fix a few evaluation scenarios first.
2. Run the prompt in fresh assistant sessions.
3. Collect both self-reported ambiguity and tool/checklist traces.
4. Fix one ambiguity per iteration.

For Musea-specific workflows, check whether the assistant:

- used real components from the registry
- matched prop and variant names to MCP output
- read tokens before suggesting visual values
- reported missing metadata instead of guessing

The full MCP guide includes a longer reproducibility workflow and prompt template:
[docs/content/integrations/mcp.md](https://github.com/ubugeeei/vize/blob/main/docs/content/integrations/mcp.md)

## License

MIT
