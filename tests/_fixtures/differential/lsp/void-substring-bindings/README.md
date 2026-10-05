# Bare binding source-map regression

`App.vue.txt` retains the complete original SFC from issue #7835. The strict
Bundler config is preserved alongside it. `lsp_void_substrings_cli.rs` drives
fresh, typed `vize lsp` processes over stdio and compares complete hover,
references and rename responses from every authored occurrence, then applies
the actual edits and compares all resulting source bytes.

The independent matrix covers all seven reported `void` substrings and all
18 reported unaffected controls, LF/CRLF, an astral prefix, same-name parameter
shadows and non-code text. Versioned diagnostic publications must remain empty
before and after unsaved rename edits. The generator test separately checks
all source and generated expression ranges. Expected objects come from the
authored declaration/type/occurrence contract, rather than captured broken
responses. This corpus grants no native or complete historical LSP credit;
#6883 remains open.

The existing script-only `ref` Composition API documentation card is compared
as a complete separate control response. Its template hover, references and
rename retain the typed binding contract. Documentation precedence for local
API-name shadows remains an explicitly unfinished separate concern.
