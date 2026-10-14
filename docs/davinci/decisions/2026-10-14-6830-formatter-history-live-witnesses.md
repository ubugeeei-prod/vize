# Formatter history reads archived inputs, not live witnesses

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The formatter history loaders pinned the whole bytes of live files that other
work edits: the shipped NAPI formatter source, the Vite+ test files, a glyph
fixture and the import-sorting CLI test. Appending a newline to any of them
failed six tooling tests even though no formatter behaviour changed.

- The historical Vite+ inputs (`import-sorting.test.ts`, `config.test.ts` and
  `UserCard.vue`) are read from archived copies under
  `tests/_fixtures/differential/config/vue-namespace/source-custody/`. The
  archives keep their size and SHA-256 checks; the live files are not read.
- The NAPI witness reads `crates/vize_vitrine/src/napi/format.rs` only to find
  the witnessed functions. The authored test law
  `native_sfc_sorting_uses_the_authored_full_reference` keeps its function
  hash; shipped functions are checked for presence only.
- The sorting-config witness keeps its retained-law function hash and drops the
  whole-file hash of `crates/vize/tests/fmt_import_sorting_cli.rs`.
- The archived helper and caller copies that only served the removed
  self-hashing test are deleted.

Every fixture input, reference output, API manifest and report validation is
unchanged. `formatter-instruction-counts` still fixes `benches/formatter.rs`,
because the instruction budget compares runs of that exact benchmark.
