# Format comment-bearing JSON config files

Issue: [#7865](https://github.com/ubugeeei-prod/vize/issues/7865).

The reported `.devcontainer/devcontainer.json`, `turbo.json` and
`trailing.json` fail default formatter discovery even though the same content
works under the known `tsconfig.json` name. File-name heuristics omit common
authored tool configurations.

Select the existing JSONC formatter for every `.json` and `.jsonc` CLI input.
Keep comments in authored order and accept trailing commas on input while
removing them in printed output. Ordinary strict JSON retains identical
output through the existing shared parser/printer. The public `format_json`
API remains strict: this correction changes CLI file selection policy only.
Malformed values, invalid numbers, invalid escapes and unterminated comments
remain errors and must not modify source files.

The sixteen-row legacy corpus retains all four original inputs, then adds
four previously unrecognised config names, ordinary JSON, empty collections,
array trailing commas and quoted comment-like scalar content. Four complete
error controls preserve the existing grammar. Rust checks all file selectors,
whole output, changed flags, fixed points and strict-public-API rejection.
The existing source-built tooling jobs also run check, three writes and
recheck on every original input, retaining complete raw process streams and
document bytes under `target/differential/`. Failed checks stay read-only.

Hosted source qualification, protected full suites, unchanged instruction
ceilings, actual merge and release are still required. No Davinci formatter
migration or measured performance improvement is credited.
