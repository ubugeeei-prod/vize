# Formatter fix-history output fixtures

Tracked in [#6882](https://github.com/ubugeeei-prod/vize/issues/6882), before
replacing the legacy formatter path. The issue's 56 past fix-title commits
remain the minimum audit scope. A complete input/output witness is required;
fixed-point or substring assertions alone do not establish full output parity.

## Decisions

- Retain existing semantic and fixed-point assertions. Add full outputs where
  those assertions leave a gap; reference existing complete witnesses without
  claiming duplicate coverage.
- Use Insta's binary snapshots for new output references. Compare the exact
  returned UTF-8 bytes, including CR/LF and the final newline. No trimming,
  newline normalization, sorting, or production formatter changes are added.
  Git text conversion is disabled for prepared assets and binary references.
- Distinguish public `format_script` and `format_sfc` observations from CLI and
  private helper observations. An internal helper accepting a token sequence
  does not imply that the public CSS parser accepts it.
- Captured outputs are current legacy observations, not inferred historical
  complete goldens. Source, executable, options and input hashes stay explicit.
- Prepared SFC input assets use `.vue.txt` with explicit `kind: Vue` metadata.
  They do not silently enter the global `.vue` census before shared corpus
  registration and its acceptance accounting are implemented.

## First script slice

`crates/vize_glyph/tests/fix_history_script.rs` adds six byte comparisons:

| Public API      | Input                       | Options                   | Existing requirement                  |
| --------------- | --------------------------- | ------------------------- | ------------------------------------- |
| `format_script` | TypeScript return signature | default                   | #2035 fixed point                     |
| `format_script` | Chained Zod regex           | default                   | #2035 fixed point and regex           |
| `format_sfc`    | SFC return signature        | default                   | #1965/#2035 fixed point               |
| `format_sfc`    | SFC chained Zod regex       | default                   | #2035 fixed point and regex           |
| `format_script` | Return signature            | internal single-pass flag | #2035 check output                    |
| `format_sfc`    | SFC return signature        | internal single-pass flag | #1974 check output and change verdict |

The four authored inputs are copied byte-for-byte from the existing regression
tests. Default cases retain three real formatting passes; check cases retain
the intermediate-versus-canonical distinction and actual change verdicts.
The single-pass flag is an internal runtime option, not user configuration.

The fixture manifest records the input and full-output digests, public API,
options, original witness and product source pin. The capture receipt records
the real Cargo-selected test executable, profile, features, source hash,
toolchain and raw log hashes. Logs and the frozen executable remain at the
receipt's local paths; no Actions artifact or remote publication is claimed.

At base `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`, the product source tree
stayed unchanged. A narrow offline build in an isolated target directory took
62 seconds. All six tests passed, then passed again against stored snapshots
with updates disabled; a frozen executable replay also passed all six.
Actions validation and shared corpus registration remain pending.

## Remaining work

- Finish the commit-by-commit audit of all 56 original fix-title commits and
  supplementary behavioral changes; preserve superseded contracts explicitly.
- Add missing complete public outputs for CSS, opaque templates, script block
  identity, directive entities, raw regions and source-owned root tags.
- Keep call-count/performance helper requirements alongside output witnesses;
  a public output snapshot cannot prove the number of formatting passes.
- Register prepared inputs and expected bytes in the shared differential
  runner, with fail-closed execution and exact result accounting.
- Verify full Actions checks and the merge-queue corpus before closing #6882
  or replacing the legacy formatter path.

These six prepared comparisons receive no shared corpus or native acceptance
credit yet. Native formatter support is unavailable; handled, equivalent and
paired native comparisons remain zero. #6882 remains open.
