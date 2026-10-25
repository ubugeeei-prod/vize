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

| Environment variable                  | Output                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `VIZE_FALSY_AND_CAPTURE`              | Original standalone producer contract, keyed by fixture ID, then `source` and `outputs`; the original `code`, `componentCode`, `functionName`, `diagnostics` and `map` facets remain exact.                                                                                                                                        |
| `VIZE_JSX_FALSY_REFUSAL_CAPTURE`      | All four original VDOM SSR refusal packets: `id`, `source`, `module`, `moduleMap`, complete `components`, structured `diagnostics`. Written before assertions.                                                                                                                                                                     |
| `VIZE_JSX_VALUE_CAPTURE`              | Current complete falsy/ternary module payload consumed by Node, with the actual native `App` entry.                                                                                                                                                                                                                                |
| `VIZE_JSX_FALSY_RECEIPT`              | Existing raw runtime receipt, now also retaining each positive SSR object's role and server diagnostics.                                                                                                                                                                                                                           |
| `VIZE_JSX_NESTED_RUNTIME_CAPTURE`     | `{ schemaVersion: 3, cases: [{ name, mode, source, historicalPacket, historical, refusalPacket, refusal, expectedPacket, expected, current }] }`. Historical, expected and current packets retain every module, map, diagnostic and component facet. All four cases are compiled and captured before the first equality assertion. |
| `VIZE_JSX_VAPOR_COMPAT_CAPTURE`       | Exact `{ corpus, compiled }` stdin payload, including complete native/VDOM module text and entry names. Written before Node executes.                                                                                                                                                                                              |
| `VIZE_JSX_VAPOR_COMPAT_TRACE_CAPTURE` | `{ installed, cases, failures, exception? }`; each case retains unmodified `referenceCode`, full `reference` and `actual` phases, plus `babelCode`, `babel`, and `vdom` where applicable. Failure writes preserve completed observations and the raw exception.                                                                    |

The native oracle's existing `--reference` mode still emits the same reference
trace on stdout; workflows can retain that stdout and stderr separately before
or after current-module execution.

## Reviewed refusal successors

`authored-module-refusal-10701-successor.json` records two additive Vapor packets
from literal source head `664c7dc3846c068a756b7562e11eab35a9497d9d`, tree
`3e1d3d673c142beb6653ad8ef47109f81850e99d`, Actions run `40221673116`, artifact
`12609354973`. Its full raw nested capture is byte-identical to the independently
reviewed earlier capture. The historical packet files and producer attribution
remain unchanged.

The stories successor changes only the precise unsupported-props diagnostic.
The controls successor changes only the owned-arrow refusal diagnostic and the
former standalone module to an empty module. All per-component code, metadata,
style and map facets stay exact. Both VDOM packets retain their original whole
module expectations. These native refusal packets provide producer inspection;
they do not qualify either nested module for runtime execution.

The Rust test keeps `historicalPacket` and `historical` attached to the original
files. Separate `expectedPacket` and `expected` fields name the two reviewed
Vapor successors. The independent Node custody test pins every file/input hash
and field set, restores only the named changed facets to compare entire packets,
and rejects mutations of component code, maps, styles, setup, ownership,
diagnostics, module publication and the allowed-facet list.

`falsy-and.authored-module-roles-10701.json` is a role sidecar, not an output
successor. It keeps the historical `compile_jsx -> module_code` attribution and
separately names today's explicit standalone producer controls and actual
`App`-object runtime controls. Its hydration evidence uses the VDOM client to
hydrate same-source SSR Vapor-object HTML; it does not claim Vapor hydration.
The four original functional VDOM SSR modules still refuse. Physical provider
hashes were not recorded in this receipt. None of these captures grants release
or public-consumer qualification.

## Static values and adapter execution

`static-values-10701-successor.json` names a later stories Vapor successor. Only
`/components/3/code` and `/components/4/code` change: actual immutable local
component values use static creation, and their slot layout loses its dynamic
boundary. Every other module, diagnostic, component, map, style and setup facet
remains equal to the frozen 664 refusal successor. Both previous files remain
unchanged. The whole module still refuses, so native handled count and runtime
credit remain zero. The capture's `refusalPacket`/`refusal` retain that previous
stage independently of `expectedPacket`/`expected`.

`native_component_stability` covers 14 sources in both native Vapor and default
VDOM modes. Actual semantic symbols distinguish const values, illegal writes,
ambient declarations, live imports, members, repeated values and lexical
shadows. The compiler reuses the existing semantic program and builds its
Vapor-only facts once when a bound identifier needs them. Default VDOM performs
no added fact scan. Five native sources independently reconstruct their entire
decoded token arrays, including Unicode and CRLF positions. All raw packets
retain serialized map bytes and decoded arrays; git goldens retain complete
code/map fields while omitting duplicate decoded arrays. The pinned 664 source
runs the same observer with a strict old dynamic-value contract. Every default
VDOM and conservative native code/map/token packet must match before and after.
These are source controls and do not establish imported/live component SSR
hydration behavior.

The pure Rust adapter DTO testcase is discovered by its exact full name and
must report one passing testcase. Its raw selection, execution and receipt
files distinguish it from actual Node ABI execution. The earlier default-feature
selector matched zero tests and grants no adapter execution credit. Only the
JSX module's test gate exposes its pure implementation without enabling Node
FFI; the `napi` feature retains the public binding contract. Actual installed
Node/Babel and browser behavior remains a separate campaign.

Current experimental Vapor runtime qualification targets the pinned rc.11 API.
Historical rc.9 aliases and positive/reference traces remain regression evidence;
failed older-runtime API rows remain explicitly unsupported. Neither a helper
alias nor an output rewrite upgrades them to current-target support. VDOM and
Vapor browser qualification keep their own exact runtime graphs and raw traces.
