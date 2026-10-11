# Retain native JSX roots in typechecked spread children (#6879)

The same original Element Plus native packet as the quoted-prop anchor failure
contains syntax errors in `tab-nav.tsx.ts`: authored JSX inside a spread-child
array/conditional survives into a plain `.ts` projection. This filename is the
intentional opt-in Vue JSX lowering path; changing the extension would bypass
its existing semantic contracts.

`lower_spread_child` now calls the existing native nested-root retention method
before raw spread-expression emission. That method is guarded by the existing
analysis-only `preserve_slot_parameter_types` setting. Canon's existing render
context rewrites the retained roots while preserving authored array,
conditional and callback scopes and component-prop checks. Compile callers and
their unsupported runtime-spread rejection remain unchanged.

The complete pinned authored TabNav and its complete actual original-H native
input are retained in
`tests/_fixtures/differential/typechecker/jsx-spread-nested-roots/` with MIT
license and full source/provider hashes. The differential requires valid
authored TSX, invalid original plain TS, and parseable fresh complete plain TS
with component arguments still source-mapped. The separate callback control
retains a wrong number prop expression rather than erasing semantic checks.

This is the dependent child of the quoted-anchor generator repair. Required
source Actions, genuine native syntax/semantic/repair controls and the original
full consumer suite remain pending. The original malformed Hoppscotch
declaration remains an authored error and is not modified or excluded.
