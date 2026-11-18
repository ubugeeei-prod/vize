## Summary

ESLint drops a leading byte order mark before it reports positions, and editors don't display one, but Patina's reporters counted it as the first column of line 1. Every line-1 finding in a BOM-prefixed file was therefore one column to the right. One example is dbx `AiAssistant.vue`'s `vue/block-lang` on `<script setup lang="ts">` in the Real Project Matrix (#12336), where Patina reported column 16 and eslint-plugin-vue column 15.

`SourceLineIndex` now starts line 1 after the mark. Later lines, byte offsets and fixes are unchanged.

`crates/vize_patina/src/output/shared.rs` is a registered report source, so its new sha256 is appended to `CURRENT_REPORT_SOURCES` in `tests/differential/linter-api.ts`. The three current report packets are byte-identical, since none of them has a BOM.

## Tests

- `output::tests::byte_order_mark::json_columns_skip_a_leading_byte_order_mark`: line 1 `lang` is reported at 15–19, and line 2 is unchanged.
- Differential case `linter/bom-columns-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
