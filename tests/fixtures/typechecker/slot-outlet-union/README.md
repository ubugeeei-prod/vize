# Slot outlet typechecker regression corpus

Owned by [#6922](https://github.com/ubugeeei-prod/vize/pull/6922), with
fix-history registration under [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
`check_slot_outlet_union_cli` reads the manifest and copies all twelve registered
sources into its isolated project. The original four component inputs are
preserved byte-exactly. Required TSGO runs require real Vue and fail if missing.

`Many17.vue` / `Many18.vue` and `ManyControl.ts` require first and last payloads
from seventeen/eighteen outlets. `ManyWrong.ts` rejects bad first/last values and
discriminants. `OptionalControl.ts` accepts absent/string/number and rejects
boolean; `Single.vue` and its control preserve string widening and reject a bad
number. Together with the original bad forwarded props, the CLI must report
exactly ten diagnostics and exit 1.

`typescript-oracle.json` pins all eight added negative diagnostics, including
complete message and column. Its current observation is TypeScript 6.0.3 on
emitted-helper bytes with fixture carrier fragments; it does not pretend that a
private candidate CLI has been built. The T1 tooling test validates the exact
source-build receipt, checks actual CLI diagnostics, then compiles its actual
virtual documents using TypeScript and compares those complete records. It
writes raw candidate observations to `target/differential/slot-outlet-union.json`.
The two original Vue diagnostic identities are pinned; their complete records
must be captured and pinned from fresh Actions before merge.

There is no native typechecker adapter or native parity claim. External authored
slot overloads retain the main-line last-signature fallback, while generated
inferred slots use one merged signature per name without a sampling bound.
