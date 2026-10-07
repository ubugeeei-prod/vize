### Area

`vize fmt` (glyph), Vue templates

### Version

`vize` 0.432.0

### Summary

`printWidth` is not honoured inside templates in two places that Prettier / Oxfmt break:

1. an attribute value that does not fit is kept on one line (Prettier moves the expression inside the quotes onto its own line);
2. whitespace-sensitive inline content (text + interpolation + an inline `<template v-if>`) stays on one line (Prettier breaks inside the tags, so the rendered whitespace is unchanged).

Different from #7236 (single-attribute start tags): here the tag already wraps one attribute per line.

On a real-world app (291 SFCs, `printWidth: 90`) switching from Oxfmt to `vize fmt` raises the number of lines over 90 columns from 1,050 to 1,457.

### Minimal reproduction

`vize.config.ts`

```ts
export default {
  formatter: { printWidth: 80, semi: true, singleQuote: false, trailingComma: "none", arrowParens: "avoid", sortAttributes: false }
};
```

`Example.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import MyList from "./MyList.vue";

const settingsPanelState = ref({ pagination: { pages: [1, 2, 3] }, selectedPageIndex: { value: 0 } });
function openSettingsDialogForTheCurrentlySelectedItem(): void {}
</script>

<template>
  <div>
    <MyList
      :items-grouped-by-page-for-the-current-view="settingsPanelState.pagination.pages"
      :selected-index="settingsPanelState.selectedPageIndex.value"
      @click="() => openSettingsDialogForTheCurrentlySelectedItem()"
    />
    <p>
      Total {{ settingsPanelState.pagination.pages.length }} items<template v-if="settingsPanelState.selectedPageIndex.value > 0"> (page {{ settingsPanelState.selectedPageIndex.value }})</template>
    </p>
  </div>
</template>
```

```sh
vize fmt --write Example.vue
```

### Actual (`vize fmt`)

The template is unchanged: the `:items-grouped-by-page-for-the-current-view` line is 87 columns and the `<p>` content line is 197 columns.

### Expected (Prettier 3.9.9, same options)

```vue
<template>
  <div>
    <MyList
      :items-grouped-by-page-for-the-current-view="
        settingsPanelState.pagination.pages
      "
      :selected-index="settingsPanelState.selectedPageIndex.value"
      @click="() => openSettingsDialogForTheCurrentlySelectedItem()"
    />
    <p>
      Total {{ settingsPanelState.pagination.pages.length }} items<template
        v-if="settingsPanelState.selectedPageIndex.value > 0"
      >
        (page {{ settingsPanelState.selectedPageIndex.value }})</template
      >
    </p>
  </div>
</template>
```
