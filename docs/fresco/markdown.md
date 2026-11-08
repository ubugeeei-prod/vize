# Markdown

Fresco's `Markdown` component renders a deliberately small CommonMark core without adding a
parser dependency to the terminal bundle. It supports ATX and Setext headings, paragraphs, block
quotes, ordered and unordered lists, task items, fenced and indented code, thematic breaks,
emphasis, strong emphasis, code spans, links, and backslash escapes.

```vue
<script setup lang="ts">
import { Markdown } from "@vizejs/fresco";

const source = `# Build status

- [x] Typecheck
- [ ] Publish

Read the [release policy](https://example.test/releases).`;
</script>

<template>
  <Markdown :source="source" heading-fg="cyan" link-fg="blue" />
</template>
```

## Extend one block family

A block provider returns terminal rows for blocks it owns and returns `undefined` for every other
block. Providers run in array order before the built-in renderer.

```ts
import type { MarkdownBlockProvider } from "@vizejs/fresco";

export const noteProvider: MarkdownBlockProvider = {
  render(block) {
    if (block.type !== "quote") return undefined;
    return [
      {
        key: `note-${block.line}`,
        spans: [{ text: `NOTE: ${block.content}`, fg: "yellow", bold: true }],
      },
    ];
  },
};
```

Pass the provider with `<Markdown :source :providers="[noteProvider]" />`. Use the `renderer` prop
when an application needs to replace the complete source-to-row pipeline instead.

Providers receive immutable parsed blocks and render options. They should return stable row keys
and avoid terminal-width assumptions; wrapping and clipping remain the owning layout's job.
