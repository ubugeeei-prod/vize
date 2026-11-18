## Summary

vue-eslint-parser lists the JavaScript comments of directive values and interpolations in `templateBody.comments`, together with the template's HTML comments. eslint-plugin-vue's `comment-directive` reads `eslint-disable` pragmas from all of them. In airi `packages/ui/src/components/misc/callout.vue` (Real Project Matrix, #12336), this comment kept eslint-plugin-vue quiet, while Patina still reported the next line:

```vue
:class="[
  ...themeClasses[props.theme || 'violet'].container,
  // eslint-disable-next-line vue/prefer-separate-static-class
  'before-position-absolute before:left-2 …',
]"
```

The template prescan now registers line and block comments found outside strings and template-literal text in directive values and interpolations, the same way it registers HTML comments. A value without `lint-` in it is skipped without being scanned.

Stacked on #13287, which maps `vue/prefer-separate-static-class` to `vapor/prefer-static-class`.

## Tests

- `expression_comments::tests::finds_line_and_block_comments_outside_strings`: finds a `//` pragma, a `/* */` inside `${}` and a block pragma; a `//` inside a string or template text is not a comment.
- `eslint_vue_aliases::tests::expression_comment_pragmas_suppress_like_template_comments`: `eslint-disable-next-line` in a `:class` array and `eslint-disable-line` in a block comment suppress. A pragma spelled inside a string literal does not.
- Differential case `linter/template-expression-disable-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
