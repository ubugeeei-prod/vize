# Native authored-module fixture custody

The historical packets below retain their original source, output bytes, maps,
diagnostics, metadata and producer attribution. Their standalone `render` or
`ssrRender` functions are historical module output. They do not describe the
current authored-module host contract.

| Immutable historical packet                                                                                                                                | SHA-256                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `tests/_fixtures/differential/jsx/falsy-and.fixed-legacy.json`                                                                                             | `6407caced54e618f9a17081770e9144cfe3a1b88b6bfc991863d3106f9f81e74` |
| `tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441/runtime-baseline.json`                                                          | `526a17e43f403f136849d6560ca25ca330d768d550d3cb4958181c24a94ec9b5` |
| `tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441/AfsStepperDialog.stories.vapor.forwarded-slot-boundaries-9455.runtime.json.txt` | `24415129bd50e1f4e1d047dd95e09659b47951576d9ce90c8a8f02eca2305ff3` |
| `tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441/native-nested-controls.vapor.runtime.json.txt`                                  | `1cc74e8645519479e2b80cabeb9fead93d1ef27f0b8b5068c9445049d11748f8` |

`runtime-baseline.json` retains the original nested packet inventory. The
`root-fallthrough-8625-successor.json` and the forwarded-slot #9455 packet remain
separate historical successors; this adoption does not rewrite any of them.

## Independent producer and module controls

`falsy_and::renderer_producers_match_complete_frozen_legacy_bytes_and_metadata`
compares the original native standalone text against the explicit public
`compile_to_vapor` and `compile_to_ssr` producer APIs. Every text and structured
facet remains exact. The VDOM module control remains unchanged. This producer
comparison does not qualify those native standalone functions as authored modules.

`falsy_and::original_functional_vdom_ssr_sources_refuse_the_complete_module`
compiles the four unchanged original sources as VDOM SSR and pins the exact
functional-role refusal, its source span, an empty module, and no module map.
Its per-component producer packets remain available for inspection.

`falsy_and::actual_values_updates_and_hydration_match_jsx_child_semantics` uses
those same sources as real client Vapor `App` objects. Its positive SSR control
uses the same source in the supported Vapor object role. Node links the complete
module and passes the emitted `App` object directly to the official app and
server-renderer APIs. It retains source-read, DOM, identity, diagnostic and
hydration assertions. The original expression-functional VDOM SSR source is not
promoted by this distinct positive role control.

`vue_jsx_vapor_compat_oracle::real_vue_jsx_vapor_mount_and_updates_match_inventory`
also links the complete native module's `App` object. The real published
`vue-jsx-vapor` reference still uses its original functional host. The corpus,
reference generation, verdicts, complete phase comparisons, read/publication
counts and recorded identity divergence remain unchanged.

## Raw Actions capture hooks

Parent workflows create each destination directory and upload the files even
when a test fails. A capture does not accept a new golden packet.

| Environment variable                  | Output                                                                                                                                                                                                                                                                                |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `VIZE_FALSY_AND_CAPTURE`              | Original standalone producer contract, keyed by fixture ID, then `source` and `outputs`; the original `code`, `componentCode`, `functionName`, `diagnostics` and `map` facets remain exact.                                                                                           |
| `VIZE_JSX_FALSY_REFUSAL_CAPTURE`      | All four original VDOM SSR refusal packets: `id`, `source`, `module`, `moduleMap`, complete `components`, structured `diagnostics`. Written before assertions.                                                                                                                        |
| `VIZE_JSX_VALUE_CAPTURE`              | Current complete falsy/ternary module payload consumed by Node, with the actual native `App` entry.                                                                                                                                                                                   |
| `VIZE_JSX_FALSY_RECEIPT`              | Existing raw runtime receipt, now also retaining each positive SSR object's role and server diagnostics.                                                                                                                                                                              |
| `VIZE_JSX_NESTED_RUNTIME_CAPTURE`     | `{ schemaVersion: 1, cases: [{ name, mode, source, historicalPacket, historical, current }] }`. Every historical/current packet contains module, module map, diagnostics and complete component facets. All four cases are compiled and captured before the first equality assertion. |
| `VIZE_JSX_VAPOR_COMPAT_CAPTURE`       | Exact `{ corpus, compiled }` stdin payload, including complete native/VDOM module text and entry names. Written before Node executes.                                                                                                                                                 |
| `VIZE_JSX_VAPOR_COMPAT_TRACE_CAPTURE` | `{ installed, cases, failures, exception? }`; each case retains unmodified `referenceCode`, full `reference` and `actual` phases, plus `babelCode`, `babel`, and `vdom` where applicable. Failure writes preserve completed observations and the raw exception.                       |

The native oracle's existing `--reference` mode still emits the same reference
trace on stdout; workflows can retain that stdout and stderr separately before
or after current-module execution.

The nested test deliberately keeps its complete historical equality checks while
adding capture-before-assertion. A changed precise refusal or scoped-style
producer needs a separately reviewed raw Actions packet and causal successor
inventory before its current expectation changes. None of these captures grants
release or public-consumer qualification.
