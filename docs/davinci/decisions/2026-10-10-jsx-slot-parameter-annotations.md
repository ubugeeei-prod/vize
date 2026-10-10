# Native JSX slot callback parameter types

Issue: [#8419](https://github.com/ubugeeei-prod/vize/issues/8419).
Baseline: signed main `e5702be6d0119a716499827fa7b918ce8c9dcbae`.

With JSX typechecking enabled, native lowering retained only the callback's
binding-pattern span. A required `{ value: number }` slot therefore accepted
an explicitly annotated `{ value: string }` callback. The unchanged inferred
slot public-contract test also exposed this as an unused `@ts-expect-error`
when the proposed config defaults enabled JSX.

The single existing OXC parse retains both binding and full FormalParameter
spans. Native typecheck lowering and its existing editor recovery select the
formal span; compiler consumers retain the binding span. The generated callback
opener has a one-byte mapping to the actual callback AST start. Fine parameter
and body mappings retain their coordinates and reverse-lookup preference. No new parse stage,
serialized intermediate, synthetic type heuristic, or backend substitute is
introduced.

## Qualification

- Before the repair, the native span law fails with actual `props` instead of
  authored `props: { value: string }`; after the repair it retains exact UTF-8
  coordinates for named, computed and render-prop slots, including arrow,
  function, destructured, optional and default parameters.
- Pre-fix complete VDOM/Vapor module code, component code, preambles, setup
  metadata, scoped styles, template arrays, diagnostics and source maps are
  frozen in differential snapshots. The repaired compiler must match them.
- Authenticated public 0.439.0 CLI probes retain all raw stdout/stderr, exact
  process outcomes and the original native loader's successful return with
  automatically selected public Corsa 7.0.2. Incompatible arrow, function and
  destructured callbacks incorrectly pass; correct and untyped callbacks
  pass, while an independent number-body error still reports TS2339.
- All nine source CLI diagnostic/control packets pass with genuine public
  Corsa 7.0.2, including complete messages, authored coordinates, counts,
  compiler options and program membership. The unchanged inferred-slot
  five-file public contract also passes with explicit JSX enabled; its
  original `@ts-expect-error` and complete clean diagnostic oracle are retained.
  Both required-native Rust laws run on Actions and retain raw stdout, stderr
  and process outcomes. Source #8428 actually merged as signed commit
  `cdc31c192503b992dc9568e81ebae5cd0aa7c696` after protected Check
  [38034277187](https://github.com/ubugeeei-prod/vize/actions/runs/38034277187).

## Installed 0.441.0 acceptance

The terminal public release run
[38037918678](https://github.com/ubugeeei-prod/vize/actions/runs/38037918678)
attempt 2 binds C `61e0e3309a85dcb319fd5133d36c7d7f4f44964f` to H
`bb83dd9817837ab4a87b09b32646389fb570a705`. Release source PR #8473 names
the frozen H and closed without merging; the annotated tag names H. The
signed source repair above is an actual ancestor of C and H.

One fresh public npm installation, the approved eight-file collector and an
independent comparison of all 202 installed public archive files establish
the Node/native/automatic Corsa identities. All nine unchanged authored CLI
packets pass: incompatible arrow/function/destructured slot parameters report
TS2345, compatible and untyped controls are clean, and the independent body
error retains TS2339. Complete file/program membership, compiler options,
messages, diagnostic coordinates, counts, stdout/stderr and exit outcomes are
compared. Every invocation retains its actual PID/argv and original native
loader's successful return. Inputs shared with the authenticated 0.439.0
before-repair archive remain byte-identical; that archive is reused unchanged.

The [corpus](../../../tests/_fixtures/differential/typecheck/jsx-slot-parameter-annotations-8419)
retains portable derivatives of all 160 original artifact records; authored diagnostic packets remain byte-identical.
Its digest manifest pins the exact release and archive. Initial runtime
overrides were empty; npm user/global configuration files were not inspected
or hashed, and provenance signature cryptography was not verified. This
acceptance covers the nine whole CLI packets and authored diagnostic ranges;
it grants no installed source-map API, config-default, LSP or performance
credit. Acceptance-record Actions and actual merge remain required.

Broader JSX contracts, multiple/rest slot parameters and generic callback
signatures remain unfinished under #1497. This slice does not qualify the
proposed default switch by itself or close that broader issue.

## Portable publication boundary

The Git archive is an expressly labeled portable derivative of all 160 sealed original records. Only physical-root prefixes are replaced by stable placeholders; each member records its original and derived SHA256, size and verified whole-byte inverse correspondence. Original execution bytes and the private root map stay outside Git. Authored inputs and complete diagnostic stdout/stderr remain unchanged. Archived derivative scripts were not executed; embedded authority hashes describe the original execution, not a new run or byte-exact derivative script.
