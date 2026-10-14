# Honor per-file `@ts-check` and keep open external slot hosts opaque

Tracking: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The full Vue parity job failed on two type checker false results
(Check run `38356181245`):

1. A JavaScript SFC block starting with `// @ts-check` reported nothing when
   the project configuration omitted `allowJs`. `vue-tsc` checks that block.
2. `<UCard><template #header>` reported
   `TS2339: Property 'header' does not exist on type '{}'` for an adapter
   declared as `DefineComponent<Record<string, unknown>, {}, any>`.
   `vue-tsc` accepts it.

## Decision

- A JavaScript SFC is denied only when `allowJs` is explicitly `false`, or when
  the block carries no `@ts-check` opt-in. An omitted option no longer hides a
  per-file opt-in.
- The empty-slot navigation helper treats a component whose `setup` props have
  an open string index as opaque (`any`). A component with closed props and no
  declared slots keeps the empty contract and still rejects unknown slots.

## Evidence

`vize_canon` tests `release_opaque_slot_hosts` and `pragma_projection_tests`
fail before the change and pass after it. The corpus inputs are in
`tests/_fixtures/differential/canon/release-js-pragma-opaque-slots/`.
