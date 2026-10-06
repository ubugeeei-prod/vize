# Unknown configured lint rule IDs

- Issue: [#7985](https://github.com/ubugeeei-prod/vize/issues/7985)
- Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7985#issuecomment-6010390316)
- Reported version: 0.432.0
- Original reporter: ubugeeei (public GitHub ID 71201308)

Unknown root or per-entry rule names must fail as configuration errors before
collecting files. Every declared severity counts, including `off`; unmatched
entry globs and later severity overrides cannot hide an invalid declaration.
The CLI returns status 2 and lists sorted unknown IDs separately for
`linter.rules` and `entries[].linter.rules`.

The recognition authority is the actual Patina full/opt-in/Nuxt registries plus
the exported script, CSS and Musea rule metadata. It is independent of preset
selection. Existing config accessors distinguish public IDs from generated
category and type-aware markers. No configuration means no catalog construction;
`--no-config` retains its bypass. No parser or lint execution stage is added.

The corpus retains the complete original Vue/config bytes, plain command and
report body. Its `.vue.txt` is copied unchanged to a real `MyText.vue` in each
isolated test workspace. Required CLI laws compare complete status/stdout/stderr
for the original, no-input and `--fix` calls, every severity in root and unmatched
custom entries, and overwritten/deduplicated declarations. Positive controls
keep registered template/opt-in/Nuxt/script/CSS/Musea IDs valid, preserve generated
category/type-aware flags, retain `--no-config` behavior and compare the whole
corrected-rule warning vector for root and matching custom-basePath entries.

Qualification uses the existing hosted source/Rust workflows and protected full
merge queue. Local source formatting and inventory generation carry no runtime
credit. Source review, exact-head Actions, actual signed merge and installed
release verification are pending when this record is authored.

Follow-ups retained from the original report: reconcile the duplicated public
TypeScript catalog's missing `script/no-next-tick` and all-rule docs' missing
`vue/no-unused-setup-bindings`; optional spelling suggestions are separate.
Recognition never depends on these duplicated presentation catalogs.
