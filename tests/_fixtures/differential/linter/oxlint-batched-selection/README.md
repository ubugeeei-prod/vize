# Oxlint selected-rule batching

`n8n-rules.json` preserves the 51 Vize rules and options from the read-only
n8n PR #40393 at `aa173be0c65c0646a7fcec32d2c18e1eaacbc8ff`.
`App.vue.txt` is an independently authored witness for whole native
diagnostic equality before and after batching, including custom options.

`npm/oxlint/src/file-state.test.ts` compares the complete native diagnostic
vectors. `npm/oxlint/src/batched-rules.test.ts` drives all 51 rules over
1,127 physical files and verifies one native call per file, together with
option/config/source invalidation and override/type-aware controls.
