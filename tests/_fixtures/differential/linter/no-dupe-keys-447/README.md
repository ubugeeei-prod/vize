# Script setup duplicate key regression

The v0.447.0 Real Project Matrix (run 38205026873, Vize source `ef34d1a0409fc711a6c5cefcf0708b0cce78f502`) exposed 88 `script/no-dupe-keys` false negatives and zero false positives in 51 SFCs from eight projects. The rule inspected only the Options API descriptor, so a `defineProps` key that collides with a setup or sibling-script value binding was missed.

- `original-sources/` keeps the original file bytes in passive `.vue.txt` carriers so no tool treats them as project sources.
- `cases.json` lists each original `.vue` display filename, its carrier, project and revision, the reference messages from eslint-plugin-vue 10.9.2 (`vue/no-dupe-keys`), and the complete expected native service result. It also holds 39 authored boundary controls.
- `neighbor-reference-replay-whole.json` holds eight neighboring controls for interface inheritance, declaration merging, inherited overrides and closer setup owners.

`crates/vize_patina/tests/no_dupe_keys_447.rs` reads these files with the real linter and compares every diagnostic field and both counters.
