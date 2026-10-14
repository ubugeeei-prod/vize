# Boolean casting beside object-literal `withDefaults`

Vue casts an absent `boolean` prop to `false`. `vue-tsc` models that with the
boolean keys of `withDefaults`' result, so in the template a `boolean` prop
with no default is `boolean`, not `boolean | undefined`. A prop whose default
is literally `undefined` stays `boolean | undefined`.

Vize applied the boolean keys only to `defineProps` without defaults. Once an
object-literal `withDefaults` defaulted any other prop, an optional `boolean`
prop read in the template became `boolean | undefined` and reported a `TS2345`
that `vue-tsc` does not.

## Expected diagnostics

Recorded with `vue-tsc` 3.3.11 on TypeScript 6.0.3 and Vue 3.5.43 over these
files copied to `src/` (`tsconfig.json.txt` as `tsconfig.json`,
`expect.ts.txt` as `src/expect.ts`):

| File | `vue-tsc` |
| --- | --- |
| `AbsentBoolean.vue` | none |
| `AbsentNumber.vue` | `(8,28) TS2345` |
| `UndefinedBooleanDefault.vue` | `(8,26) TS2345` |

`crates/vize/tests/check_with_defaults_boolean_casting_cli.rs` asserts the
same positions and codes for `vize check`.
