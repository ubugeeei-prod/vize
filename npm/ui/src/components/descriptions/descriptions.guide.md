## Pair a term with one detail

`Descriptions` is a `dl` with an accessible name. Each `DescriptionsItem` is a
term plus a definition, so the pair is announced even when CSS changes the
visual columns.

```vue
<script setup lang="ts">
import { Descriptions, DescriptionsItem } from "@vizejs/ui/descriptions";
</script>

<template>
  <Descriptions label="Invoice">
    <DescriptionsItem label="Status">Paid</DescriptionsItem>
    <DescriptionsItem label="Total">$1,280</DescriptionsItem>
  </Descriptions>
</template>
```

The list is headless. Layout, borders, and columns belong to the consumer.
