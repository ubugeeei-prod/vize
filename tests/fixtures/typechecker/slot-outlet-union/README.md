# Slot outlet typechecker regression corpus

Owned by [#6922](https://github.com/ubugeeei-prod/vize/pull/6922), with
fix-history registration under [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
`check_slot_outlet_union_cli` reads the manifest and copies all fourteen registered
sources into its isolated project. The original four component inputs are
preserved byte-exactly. Required TSGO runs require real Vue and fail if missing.

`Many17.vue` / `Many18.vue` and `ManyControl.ts` require first and last payloads
from seventeen/eighteen outlets. Their discriminants use authored
`:kind="'first' as const"` bindings; implicit static attributes widen to string. `ManyWrong.ts` rejects bad first/last values and
discriminants. `OptionalControl.ts` accepts absent/string/number and rejects
boolean; `Single.vue` and its control preserve string widening and reject a bad
number. `StaticRepeatControl.ts` uses strict `IfEquals` to require the repeated
static kind to be exactly `string` (rejecting `any`) and accepts an arbitrary
string payload. Together with the original bad forwarded props, the CLI must report
exactly eleven diagnostics and exit 1.

`typescript-oracle.json` pins separate complete vectors: all eleven CLI diagnostics
and all eight TypeScript negative controls. Both Rust and Node check every CLI
file/line/column/code/message, including the three original Vue records. The
Node test also compiles actual CLI virtual documents plus saved ambient helpers
with pinned TypeScript. Go renders literal unions in lexical order; TypeScript
retains source order, so the exact vectors remain distinct.

The original records were captured from source-built full Actions run
36316835669 at `adc0be82a512b872c290873dfd665461dc2f2091`, with validated
ci-recipe binary/source/version receipt and independently matching coverage
execution. TypeScript 6.0.3 executed those emitted bytes locally and matched all
eight records. The old failed run exposed the oracle display-order difference;
fresh Actions on the final pinned head must pass the Node/Rust checks and full
queue. Raw candidate observations are written to
`target/differential/slot-outlet-union.json` and logged before oracle assertions.

There is no native typechecker adapter or native parity claim. External authored
slot overloads retain the main-line last-signature fallback, while generated
inferred slots use one merged signature per name without a sampling bound.

Installed official `vue-tsc` 3.3.11 (TypeScript 6.0.3, Vue 3.6.0-beta.10)
was run in isolated projects with the actual Vue SFC compiler. It strictly
confirms static single/repeated string widening and all seventeen/eighteen
explicitly bound literal overloads. Its parent binding sees the last overload;
the generated union carrier deliberately repairs that limitation. The original
Options API inputs produce untyped slots and no diagnostics with this toolchain,
so that run receives no original-fixture diagnostic parity credit. The
script-setup equivalent does observe the side string-to-literal error, but is
not a byte-exact oracle for the original inputs.
