## Lead with the number, keep the sign honest

`Statistic` formats one number with `Intl.NumberFormat` and publishes the title
as the accessible name. The minus sign is placed before the prefix, so a
negative amount reads `-$1,280` rather than `$-1,280`.

```vue
<script setup lang="ts">
import { Statistic } from "@vizejs/ui/statistic";
</script>

<template>
  <Statistic title="Revenue" :value="1280" prefix="$" suffix="/mo" />
  <Statistic title="Requests" :value="1200" notation="compact" :maximum-fraction-digits="1" />
</template>
```

`notation="compact"` uses the locale's short form (`1.2K` in `en-US`). Grouping
and decimal marks follow `locale`. The component renders no stylesheet. Style
`[data-vize-ui="statistic"]` and `[data-part="title"|"value"]`.
