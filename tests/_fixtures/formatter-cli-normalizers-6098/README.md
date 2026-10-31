# Remaining direct formatter normalizers (#6098)

These authored full-output fixtures isolate the existing `normalizeInlineStyles`,
`normalizeVForParens`, and `reindentPreprocessorStyles` policies. Controls retain
dynamic style expressions, quoted semicolons, comments, destructuring and single
aliases, parenthesized aliases, unsupported Sass, and unbalanced SCSS.

The CLI matrix covers implicit true and explicit booleans after config/presets,
unchanged config-only behavior, complete source bytes on three write passes,
unchanged configuration bytes, and final `--check` acceptance. The underlying
formatting algorithms, schema, named presets and defaults are unchanged.

`combinations.json` adds all eight combinations on one SFC, with inline styles
and `v-for` aliases sharing one element and a nested SCSS block. Every CLI value
overrides the opposite project setting. Three write passes and a final check
compare complete output bytes and retain the configuration. Separate controls
cover implicit defaults and discovery through the real `fmt --help` command.
`fmt-help.json` records its complete real-process stdout, including padding on
blank lines; the ordinary unit help snapshot intentionally normalizes that
padding. Both complete output contracts remain checked independently.

The original nine fixtures remain unchanged. Together the tests plan 251 real
CLI invocations; execution is qualified by the current Actions run.
