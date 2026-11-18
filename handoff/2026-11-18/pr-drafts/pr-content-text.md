## Summary

`vue/no-content-outside-block` reported everything between the SFC blocks, including top-level elements compiler-sfc drops. vue-multiselect `documentation/src/getting-started-code/HTMLInstallCode.vue` (Real Project Matrix, #12336) shows the problem:

```html
<script src="https://unpkg.com/vue-multiselect"></script>
<link rel="stylesheet" href="https://unpkg.com/vue-multiselect/dist/vue-multiselect.min.css">
```

eslint-plugin-vue reports only the `VText` children of the document fragment, and the rule's docs cover text content. So the `<link>`, and an empty custom block such as `<docs></docs>`, are not content.

Comments, start tags (with their raw content up to the matching end tag), void elements, self-closing tags and stray end tags now end a text node, as they do in vue-eslint-parser. An element without its end tag runs to the end of the file. vue-eslint-parser then has an invalid EOF, and eslint-plugin-vue reports nothing.

A top-level element that compiler-sfc drops may still deserve a warning of its own, but that would be a separately named rule (for example `vue/no-dropped-top-level-element`), not this one.

## Tests

- `compatibility_sfc::tests::content_outside_blocks_reports_text_nodes_only` covers three cases:
  - the vue-multiselect file reports nothing;
  - `lead <br/> tail`, `<docs></docs>`, a quoted `>` in an attribute and `</bar>end` report exactly `lead`, `tail` and `end`;
  - an unclosed `<docs>` reports nothing.
- The existing text and comment cases are unchanged.
- Differential case `linter/content-outside-block-elements-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
