# CLI template whitespace (#7880)

`App.vue.txt` is the literal result of the reporter’s `printf` command, including its final LF. The two preserve configs retain the reported JSON and TS declarations; hashes and byte counts are frozen in `manifest.json`. Git checkout filters are disabled for this directory.

`crates/vize/tests/build_whitespace_cli.rs` compares complete JS/JSON modules, statuses, stdout/stderr and unchanged inputs for auto/explicit JSON/TS configuration across DOM, SSR and Vapor. Only the well-formed four-decimal elapsed duration is replaced in stderr. The stats path additionally shares one actual cache across all three modes in `runner/settings/tests.rs`; it verifies compile output length, entry count, repeated hit behavior and restoration of the existing parser scope.

`tests/tooling/cli-build-whitespace-7880.test.ts` consumes the existing exact-source CLI build receipt and renders twelve complete original DOM/SSR outputs against independent official Vue 3.5.35, already pinned in the UI workspace. Complete process streams, modules, official modules and rendered results are retained before assertions. The reporter compared 3.6.0-rc.10; this suite does not claim that version or browser/Vapor hydration parity.

These are legacy CLI transport and semantic tests. Davinci replacement, native performance, public installation and actual merge/release require their separate evidence.
