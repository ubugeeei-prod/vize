# Slot outlet typechecker regression corpus

Owned by [#6922](https://github.com/ubugeeei-prod/vize/pull/6922), with
fix-history registration under [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
The existing `check_slot_outlet_union_cli` integration test reads `manifest.json`
and copies every registered Vue source into its isolated project before running
its source-built CLI. The original four component inputs are preserved exactly.
`Single.vue` adds direct `useSlots` and `$slots` calls with a different string,
which must remain valid for one widened string outlet.

The legacy oracle requires the existing two bad forwarded-prop diagnostics,
no other diagnostics, and exit status 1. This is legacy regression evidence;
no native typechecker comparison is implemented or claimed.

`seventeen-overloads.ts` is an explicitly pending semantic probe, rather than a
passing CLI source: prepend the emitted slot helper aliases to typecheck it.
Both first/last payloads should pass. The current helper rejects the first with
TS2322 because it samples only the last sixteen signatures. Do not increase an
arbitrary bound or treat this probe as passing evidence. Replace inferred-child
carrier sampling with direct payload-union composition and design the handling
of externally authored overloads before claiming all outlets are preserved.
