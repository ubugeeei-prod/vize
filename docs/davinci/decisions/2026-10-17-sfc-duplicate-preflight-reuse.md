# SFC duplicate-property preflight: reuse the actual DOM surface

Status: uncompiled production review draft. Base `502073de95bd`.
Local wiring and controls are prepared; no Cargo invocation, push or Actions run yet.
Related: #6826, #6835, #6880. None is completed by this slice.

## Actual ownership and cost

`compile_template.rs::compile_template_block_with_capture` calls
`duplicates::reject_duplicate_props` before DOM/SSR selection for Vue 3.
`find_duplicate_props` performs an additional complete lexical pass, retains
header-local names/arguments/modifiers and formats all duplicate spans into a
fatal `SfcError`. It intentionally permits repeated `@`/`v-on` and object
`v-bind`, and retains prefix spelling and modifier order.

`vize_armature::tokenizer` is only a compatibility re-export of
`vize_l1::markup::lex::compat`: this is NOT a second legacy lexer engine.
Removing an import or facade does not establish native-only execution.
The real opportunity is removing the extra lexical pass on an already parsed
native DOM path while retaining the complete error contract.

The native DOM SFC fast path is:

1. `vize_atelier_dom::compile::sfc` selects the L2 sections path.
2. `stage_options::{emit_l2, emit_l2_captured}` invokes the L1-to-L2 emitter.
3. `vize_l1_to_l2::emit::budget` calls `parse_with_options` exactly once
   (ordinary and capture paths), then lowers the existing `SurfaceTree`.
4. The borrowed `OpenTag::attrs` retain all original attribute-name bytes.
   `VueDirectives::decompose` already supplies checked argument/modifier spans.

The reuse point is immediately after that actual parse and before lowering.
The existing fast-path selector already sends repeated raw attribute names
through compatibility. That diagnostic path remains unchanged; the first
optimization benefits actual native fast-path headers. Different raw spellings
with equivalent callback identity still require the surface check (for example
`v-x:[a]foo` and `v-x:foo`).
Calling `parse`, `parse_component`, or `NativeComponent::parse_in` again for
preflight would be a regression and is excluded.

## Small production integration proposed for review

Keep semantic comparison in SFC's existing `duplicates` module; it can borrow
an existing L1 tree without moving product compatibility policy into L1.
The accompanying surface implementation adds no parser or name scanner, reuses
`Candidate::{same_name, may_repeat}` and `VueDirectives`, and declines the
entire observation on a surface error, missing header, invalid source span,
or unresolved directive spelling. All title-containing observations decline:
namespace-aware native parsing admits SVG title markup children, while the
retained callback treats title bodies as raw text. Upper SFC policy likewise
uses the whole retained callback when experimental in-tag comments are enabled,
because the original preflight always uses default comments-off lexing. No partial duplicate list can be trusted.
It uses the actual generic DOM surface's neutral `v-pre` policy, not the
native dialect's different frozen surface.

The coherent local draft includes an invocation-local preflight hook. This is required because `EmitError` is a copyable refusal,
whereas the complete fatal SFC diagnostic is owned by the caller:

```rust,ignore
// Generic borrowed contract; diagnostics remain owned by the upper consumer.
// Called once after the existing native parse, before any lowering.
preflight(Some((&tree, &errors)))?;
// Called at the actual compatibility parse entry when no native tree was observed.
preflight(None)?;
```

A `None` event means entry to compatibility AST parsing, not final backend
selection. The shared path can still emit L2 for source-map or selector-only
fallbacks. Only `after_refusal` forces compatibility codegen.

The SFC closure retains the complete `Result<(), SfcError>` and a per-call
checked flag. `Some` runs the surface checker. A whole-observation refusal
runs the unchanged callback preflight against the original source. `None`
does the same only when not already checked. A fatal result must stop the
native attempt before structural directives lower, and stop compatibility
compilation before it begins; the outer SFC entry returns that original
complete error. A successful native check must not be repeated if emission
later refuses and the compatibility backend is selected.

Ordinary callers install no hook; existing option literals and public result
structs stay unchanged. The new hidden emitter returns `None` for unobserved
budgets instead of inventing zero counts, while still using `NoObserver`. Capture/no-capture paths use the same check point. Validation rejects before
pages or lowering; only appended DOM option metadata is truncated on a fatal
SFC preflight result, preserving prior sidecar bytes without cloning the
sidecar. Existing capture commit/discard behavior remains in place. This is a DOM-only first integration. SSR keeps its existing
preflight, and Vapor's separate compile path is unchanged.

Deleting the original preflight before those early exits are wired would
lose fatal duplicate diagnostics. Turning duplicates into a generic
`EmitError::Diagnostics` would also lose their full spans and message.
Neither is acceptable.

## Acceptance and validation before delivery

The draft contains eleven original header controls and whole-refusal/frozen
controls comparing complete byte spans with the unchanged callback oracle.
The production hook is wired. The direct provider laws cover actual native
parse callbacks before lowering with NoCapture/CaptureSink and observer off/on.
Upper laws cover exact-once dispatch, late refusal, unrelated equal-text source,
and full error/sidecar preservation. An existing `legacy-dom-differential`
feature test defines 416 complete public Result comparisons over twenty-six
original inputs, all four targets, maps off/on, ordinary and captured entry.
The reference restores the untouched eager duplicate callback at its original
SFC call position and supplies no hook to the ordinary/captured DOM dispatcher.
It preserves default backend selection. Its guard exists only under
`cfg(test)` plus `legacy-dom-differential`, and RAII restores nested and panicking
invocations. SSR/Vapor remain complete negative controls. The existing SFC
differential recipe also executes
`cargo test --locked -p vize_atelier_sfc --features legacy-dom-differential --lib
compile_template::duplicates:: -- --nocapture` under its existing profile and
environment, so these feature unit laws are not omitted by integration-only
selection. Its existing command-transport test also exercises failure propagation.

The actual late-refusal control reuses the committed `#actions="v-slot:actions"`
source whose native emission refuses with `PrefixExpressionRejected`, after the
native preflight ran once and without the retained callback running. It asserts
ordinary refusal notes, captured `Legacy("emit-refused")` with no native pages,
and complete diagnostic/code/preamble/map/section plus public SFC Result parity.
`@[event].stop` is a supported control, not a refusal witness.

### The first MQ comparison exposed an oracle boundary

[The first MQ SFC job](https://github.com/ubugeeei-prod/vize/actions/runs/38938620914/job/117100507469)
passed the real late-refusal law but the complete comparison stopped on
`<template><div :key..prop='a' :key.prop.='b'></div></template>`. Its reference
incorrectly forced every DOM compile to the compatibility backend. That changes
more than the duplicate callback: the compatibility parser reports
`MissingDirectiveModifier`, while the original default native lowering drops
empty modifier segments and emits two `.key` properties. The unchanged callback
accepts these differently ordered modifier runs. This is a pre-existing parser
and codegen frontier, not an optimized-preflight error to repin or omit.

An independent installed official `@vue/compiler-dom` 3.5.41 probe on the exact
inner template reported no errors and emitted one `.key` property using `a`.
Neither Vize branch establishes complete Vue correctness for this input. The
explicit frontier control records the existing difference; it is not the oracle
for the 416 comparisons. Genuine forced-legacy comparison remains appropriate
for the known post-parse refusal, where compatibility codegen actually supplies
the default result.

The first source compiler cohort passed 6,941 tests, including the ordinary
preflight/provider laws. The corrected feature reference and its nested/unwind
control still require a new source run and protected MQ execution. All 26
original configured inputs are retained, and the 416-comparison law collects
complete differences across every input/target/map/capture combination before
failing, so one frontier cannot conceal later failures.

Required shipping validation for the eventual coherent patch:

- Compare complete serialized default Results against the untouched retained
  preflight plus the same downstream compiler, not just a duplicate count.
  Include exact fatal error messages/codes/locations and all diagnostics in
  original byte order; success cases compare complete code/preamble/maps.
- Cover DOM-inline/module, maps on/off, normal/setup/template-only inputs,
  forced legacy selection and native refusal; SSR/Vapor remain full-result
  negative controls. Include the corresponding ordinary/captured stage flows.
- Original controls include repeated structural directives, ASCII-case static
  names, full versus shorthand binds, dynamic arguments, modifier order and
  empty modifiers, repeatable listeners/object binds, raw-text/quoted/comment
  pseudo-tags, Unicode/CRLF, native frozen subtrees and malformed headers.
  Late refusal must discard all earlier native spans and return the entire
  original callback result.
- Pinned official Vue independently records semantics/diagnostics for the
  same original inputs. Differences in retained Vize policy are explicit
  frontiers; they are not silently normalized or repinned.
- Use the existing isolated allocation/walk harness to compare whole
  parse-plus-compile paths for admitted, native-refused and forced-legacy
  cases. Observe the actual native parse boundary and fallback calls through
  the same hook: no second parser and no preflight re-entry on late refusal.
  Keep every existing cap and instruction check.

This advances #6835's consumer migration and #6826's default compiler dependency
boundary under #6880's compatibility gate. It does not close tokenizer,
compiler, fix-history or #6826 product acceptance. The production reach sweep
still directly uses Croquis for SFC descriptors; its honest native-only zero
must not be changed based on this preflight work alone.
