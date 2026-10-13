# `withDefaults` with defaults that are not an object literal

Vue types `withDefaults(defineProps<T>(), defaults)` from the type of
`defaults`. A prop is non-`undefined` in the template when that type carries a
defined value for it, whether the argument is an object literal, a call, a
conditional or a spread.

Vize used to read the defaulted names from the source only. A defaults object
returned by a call left every optional prop `T | undefined` in the template and
reported errors that `vue-tsc` does not.

## Where it was found

vue-vben-admin (MIT) at `748b182e440863711dc9624924e92440f5ddc4d3`,
`packages/@core/ui-kit/shadcn-ui/src/ui/tree/tree.vue`:

```ts
const props = withDefaults(defineProps<TreeProps>(), treePropsDefaults());
```

The template passes `valueField`, `childrenField` and `labelField` to
functions that take a `string`. The real-project matrix reported nine false
positives for that file (`TS2769` and `TS2538`). The files here are an
independently written reduction of that shape.

## Expected diagnostics

Recorded with `vue-tsc` 3.3.11 on TypeScript 6.0.3 over these files copied to
`src/` (`tsconfig.json.txt` as `tsconfig.json`, `defaults.ts.txt` as
`src/defaults.ts`):

| File | `vue-tsc` |
| --- | --- |
| `CallDefaults.vue` | none |
| `PartialCall.vue` | `(11,14) TS2345` |
| `SpreadCall.vue` | `(10,14) TS2345` |
| `UndefinedCall.vue` | `(10,14) TS2345` |
| `ConditionalDefaults.vue` | `(10,14) TS2345` |
| `BooleanCall.vue` | none |

Each reported position is the one prop on that line whose default the argument
type does not define. `crates/vize/tests/check_opaque_with_defaults_cli.rs`
asserts the same positions and codes for `vize check`.
