# Model payloads of generic constructor intersections

piyota ([piyota6](https://github.com/piyota6)) reported
[`update:modelValue` payloads of a generic constructor intersection typed as `unknown`](https://github.com/ubugeeei-prod/vize/issues/13856),
as Vuetify's generic components (`VChipGroup` with a `string[]` model) are typed.

TypeScript infers from the last construct signature of such a type with its
type parameters erased, so the listener read `Props<unknown>`. vue-tsc
constructs the component with the bound props instead, which infers `T` from
`modelValue`. The fixture checks that the payload is the bound `string[]` for
both listener spellings and under `v-if`, that a lone generic constructor
without `$emit` is inferred too, and that a non-generic constructor keeps its
declared `number` payload. The four diagnostics are the ones vue-tsc 3.3.14
reports on the same file.
