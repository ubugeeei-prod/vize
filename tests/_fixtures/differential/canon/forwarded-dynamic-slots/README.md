# Forwarded dynamic slot payload controls

Minimal form of the dominant `primevue-volt` type checker false positive
(87 of 97 in Real Project Matrix run `38396999264`): a wrapper forwards every
slot of a typed host with `v-for="(_, slotName) in $slots"` and
`v-slot:[slotName]="slotProps"`.

`vue-tsc` 3.3.11 leaves that payload unchecked because the broad `string` key
selects no single slot. Vize handed the wrapper the union of all host payloads,
so each named slot of a consumer was rejected with `TS2339`.

| Input            | Expected                                                         |
| ---------------- | ---------------------------------------------------------------- |
| `App.vue`        | no diagnostics for `#option` and `#header` through `Wrapper.vue` |
| `Direct.vue`     | `TS2339` for `payload.missing` on the statically named slot      |
| `Union.vue`      | `TS2339` for `payload.missing` on a literal-union slot name      |
| `IndexedUse.vue` | `TS2339` for `payload.missing` on a host with a string index     |

The Rust control is
`batch::type_checker::tests::recent_issues::external_slot_payloads::forwarded_dynamic_slots` in
`vize_canon`.
