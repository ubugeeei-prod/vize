# Canon slot outlet repair preparation

Tracked in [#6922](https://github.com/ubugeeei-prod/vize/pull/6922) and the
typechecker fixture history [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

- Preserve the existing public string type for one inferred slot outlet.
  Literal `as const` is needed only when multiple statically named outlets
  contribute discriminants to the same merged payload. Count static names once
  alongside collected outlets, including outlets in different scopes; dynamic
  single outlets retain widening. Do not alter declared-slot checking literals.
- The latest head `e228a8226386698502fbf7d4973c5b790f77d86c` fails upstream
  `slots` at `main.vue:23:11` and `main.vue:53:15`: the exact type oracle sees
  `str: "str"` where its existing public slot contract requires `str: string`.
  `never` is the exact-type oracle's rejection, not the slot helper's payload.
- Keep test registration files under their existing length ratchets. Move slot
  tests into child modules in a separate move-only commit; do not relax budgets.
- Register the [legacy regression corpus](../../../tests/fixtures/typechecker/slot-outlet-union/README.md)
  through its manifest and existing CLI test. Its five Vue files cover the four
  existing component inputs and direct single-outlet `useSlots` / `$slots` calls.
  The shared differential helper has no native typechecker adapter yet; no native
  acceptance or byte-exact native parity is claimed.
- TypeScript 6.0.3 semantic checks prove the current extractor accepts all sixteen
  signatures but discards the first of seventeen. An optional common key remains
  optional, accepts string/number, and rejects boolean; single primitive payloads
  retain their original type. The committed seventeen-overload probe is pending,
  not green evidence. TODO: replace inferred-child overload sampling with direct
  payload-union composition and explicitly design external authored overloads.
- This preparation has not been published. Rust, upstream diagnostic, CLI and
  queue verification are pending fresh Actions on the reviewed patch. Local
  verification uses Rustfmt, diff checks and pure TypeScript probes; no local
  Cargo build runs under the disk constraint. Post-review issue evidence must be
  posted with this decision record in the same published change.
