# Editor Corsa timeout and Rust source compatibility

Decision for [#7030](https://github.com/ubugeeei-prod/vize/issues/7030) before v0.429.2.

The released `vize_carton::config::TypeCheckerConfig` is a public Rust struct whose fields can be listed in an exhaustive literal. Adding `lsp_request_timeout_ms` to it breaks source compatibility for external callers, even when the field is optional. The editor timeout added for #7030 remains a config-file key, `typeChecker.lspRequestTimeoutMs`, but is parsed by the private raw config model. The public shared struct keeps its v0.429.1 field set.

The editor config loader evaluates the raw JSON, JS/TS, or PKL config once and derives the type-checker model, timeout, linter options, and language-server flags from that snapshot. Separate loader calls could run a function config more than once and combine settings from different evaluations. Each LSP `ServerState` stores the timeout in its own atomic field, defaults to 60,000 ms, clamps zero to one millisecond, and passes it to the Corsa bridge. The CLI type-checker model and its serialization do not gain an editor-only field. The generated TypeScript type, PKL config, and JSON schema retain the config-file key.

Validation: an external-style exhaustive Rust literal compiles; a JSON config loads 90,000 ms, zero clamps to one, and removal restores the default. Function-config tests check that one load evaluates it once and keeps `strict` and the timeout from the same result. A regression test also covers two LSP server states with different workspace settings and a reload of only one state. The focused `vize_carton` integration test passed 3/3 locally with VP-managed Node; the LSP test, full suite, and release checks remain subject to Actions.
