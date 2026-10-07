## Summary

When an inline element's only content is an interpolation that Prettier / Oxfmt broke over lines (`<span>{{` … `}}</span>`, the "hug" layout), `vize fmt` keeps the layout but indents the expression and the closing `}}` one level deeper. The result is stable on later runs, so every such interpolation stays different from Prettier / Oxfmt output.

## Reproduction

`vize.config.json`

```json
{ "formatter": { "printWidth": 80, "semi": true, "singleQuote": false, "trailingComma": "none", "arrowParens": "avoid", "sortAttributes": false } }
```

`Names.vue` (Oxfmt with the same options leaves it unchanged)

```vue
<script setup lang="ts">
const describeEverySelectedItemInTheList = (items: string[]): string =>
  items.join(", ");
const selectedItems = ["a", "b"];
</script>

<template>
  <div>
    <span class="names">{{
      describeEverySelectedItemInTheList(selectedItems)
    }}</span>
  </div>
</template>
```

```sh
vize fmt --write Names.vue
```

```diff
     <span class="names">{{
-      describeEverySelectedItemInTheList(selectedItems)
-    }}</span>
+        describeEverySelectedItemInTheList(selectedItems)
+      }}</span>
```

## Expected

Unchanged: the expression one level inside the element, `}}</span>` at the element's own indentation, as Prettier prints it.

## Why it matters

This layout is what Prettier / Oxfmt produce for every long interpolation inside a `<span>`, `<p>`, `<td>` and similar inline content, so a formatted project gets a diff at each of them (119 places in a ~450-SFC app). Nothing renders differently, it is only indentation, but it blocks using `vize fmt --check` next to an Oxfmt-formatted codebase.

## Environment

- vize 0.432.0
- node 26.8.1, macOS arm64
- compared with oxfmt 0.60.0
