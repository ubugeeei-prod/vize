# Explicit Vue versions for legacy formatter filters (#6845)

Issue decision: [#6845 comment](https://github.com/ubugeeei-prod/vize/issues/6845#issuecomment-5854598681).

The public formatter previously parsed every interpolation and bound value as
JavaScript. For example, `{{ message | format-date('en') }}` became
`{{ message | (format - date("en")) }}`, destroying the Vue 2 filter asset name.
The same rewrite occurred in `:title` values. A Vue 3 expression with those
bytes is bitwise OR and subtraction, so syntax alone cannot choose the dialect.

## Decision

- Keep the existing public `FormatOptions` fields, struct literals, serialization
  and default formatter entry points unchanged. Their dialect remains Vue 3.
- Add explicit version-aware Rust SFC/template entry points using the existing
  project `VueVersion` type. CLI and LSP formatting use the resolved `vue.version`
  project setting. NAPI/WASM accept an optional `vueVersion` selector, validated
  by the existing version parser; omission preserves Vue 3.
- Only Vue 2 and 2.7 interpolation/bind payloads use filter boundaries. Events
  remain JavaScript. Follow the existing `VueFilterExpr` top-level pipe rule:
  strings, template literals, regex literals, nesting and `||` do not split.
  Keep asset names as authored and format the JS base/argument payloads.
  Restore base parentheses if the JS printer exposes a nested bitwise pipe.
  Refused or malformed payloads remain authored text instead of guessed JS OR.
  Unicode filter asset names use this authored-preserve fallback; they are not
  credited as formatter-aware or native-equivalent coverage.
- Keep the scanner local to the legacy formatter. Add no L2 dependency, product
  stage, serialized intermediate representation or new project model.
- Register three exact byte references: Vue 2 with Unicode/CRLF and chained
  arguments, Vue 2.7 longhand binding, and a Vue 3 bitwise-OR control. Physical
  inputs use `.vue.txt` with explicit Vue/runtime metadata. The CLI executes real
  `App.vue` files in isolated workspaces using one pinned JSON config and a fixed
  argument list; it cannot execute arbitrary manifest commands or config paths.

## Evidence and remaining work

The changed Glyph library and three public integration tests were compiled
directly against the coordinated narrow target's exact dependency fingerprints,
with all output written outside that target. All three tests passed, including
full bytes and `changed` over three passes. Twelve differential runner contract
checks passed, including mutated config hashes, selectors, runtime paths and
invocation evidence. [Actions run 36310533270](https://github.com/ubugeeei-prod/vize/actions/runs/36310533270)
passed at `c953521e42ffa8ea07ec8eaaae2c4d4fbe130a7c`: the actual CLI config/no-config
and exact corpus checks, LSP document/range/on-type requests, native JS selector
and real browser WASM selector tests all executed successfully. The binding
checks cover Vue 2/2.7, default/explicit Vue 3, invalid selectors and complete
fixed-point outputs. The two old CLI captures remain untouched. Native formatter
acceptance and paired comparisons remain zero. Main promotion and merge queue
proof are tracked on [#6952](https://github.com/ubugeeei-prod/vize/pull/6952).

TODO [#6836](https://github.com/ubugeeei-prod/vize/issues/6836): consume the native
FilterChain payload when the formatter migration is allowed by #6882.

The CRLF witness also preserves a preexisting SFC layout `CRCRLF` line ending,
reproduced independently with the old public formatter. This change does not
repair or normalize it. TODO [#6845](https://github.com/ubugeeei-prod/vize/issues/6845):
track the separate newline repair with its own legacy corpus fixture; do not
claim canonical EOL output from this filter regression.
