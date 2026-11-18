## Summary

`script/require-valid-default-prop` dropped any union part that was not a native keyword. That produced a false claim in element-plus `tour.vue` (Real Project Matrix, #12336):

```ts
// tour.ts
scrollIntoViewOptions?: boolean | ScrollIntoViewOptions
// tour.vue
withDefaults(defineProps<TourProps>(), { scrollIntoViewOptions: () => ({ block: 'center' }) })
```

The rule reported "Type of the default value for 'scrollIntoViewOptions' prop must be a boolean". `ScrollIntoViewOptions` is a DOM interface, so the object factory is valid.

Each union part now maps to the constructor Vue checks:
- keywords, string, number and boolean literals, `{ ... }`, arrays, tuples and function types;
- `null`, `undefined`, `void` and `never` add nothing;
- `Record`, `Partial`, `Readonly`, `Pick`, `Omit` and `Required` are objects;
- a same-file interface is an object (eslint-plugin-vue does the same), and so is an interface the type world resolves for an imported props type;
- a resolved non-generic alias is expanded.

A reference that resolves to nothing known (a DOM or global type, `any`, a type parameter, a qualified name) may be an object. The prop is then left unchecked instead of reported. eslint-plugin-vue drops such a part and still reports `must be a boolean`. If that shows up on a same-file type in the matrix, it will be classified as an upstream false positive.

## Tests

- `a_reference_in_the_union_is_an_object_or_unchecked`: same-file `boolean | Gap` accepts an object factory and still rejects `'wide'`. `boolean | ScrollIntoViewOptions` is unchecked. A plain `boolean` still rejects an object factory.
- `an_imported_reference_uses_its_resolved_shape`: covers an imported `boolean | ScrollIntoViewOptions` (unchecked), `boolean | GapOptions` (an interface, so `'wide'` is reported), an alias `Size = 'small' | 'large'` (`1` is reported) and `number`.
- Differential case `linter/valid-default-unknown-reference-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
