# Preserve native click PointerEvent diagnostics in the Canon history

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The historical fix `e7d8b1c82a8badec89ec9779838dfa50c1b9b790`
changed the native `click` and `auxclick` payload contracts from their broader
legacy types to `PointerEvent`. The original regression covered generated text
and a multi-file batch test whose assertions filtered selected diagnostic
codes. This slice records the product contract as two independently runnable
projects instead.

Each project passes a `PointerEvent` handler directly to the corresponding
native listener and requires the complete production `BatchTypeChecker`
diagnostic list to remain empty. A generator that returns `click` to
`MouseEvent` or `auxclick` to `Event` makes the narrower handler incompatible
under strict function checking. The fixtures therefore exercise the semantic
reason for the historical change without pinning virtual TypeScript bytes.

The pack uses the existing required-T1 fixture binary and the shared
typechecker differential manifest. Its input bytes, pack bytes, test identity,
source revision, and complete returned diagnostic lists are part of the
existing authenticated workspace observation. The PR profile defers this exact
real-TSGO test name; the required T1 Actions lane executes it without a runtime
absence bypass.

This admits only the two native listener payload requirements from #936. It
does not cover other DOM events, component listener tuples, end ranges, related
information, raw backend responses, native L4 equivalence, or the remaining
historical ledger. Those requirements and full #6879 completion remain open.
