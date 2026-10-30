# Remaining direct formatter normalizers (#6098)

These authored full-output fixtures isolate the existing `normalizeInlineStyles`,
`normalizeVForParens`, and `reindentPreprocessorStyles` policies. Controls retain
dynamic style expressions, quoted semicolons, comments, destructuring and single
aliases, parenthesized aliases, unsupported Sass, and unbalanced SCSS.

The CLI matrix covers implicit true and explicit booleans after config/presets,
unchanged config-only behavior, complete source bytes on three write passes,
unchanged configuration bytes, and final `--check` acceptance. The underlying
formatting algorithms, schema, named presets and defaults are unchanged.
