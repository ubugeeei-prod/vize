## One choice, with keys that wrap

`Segmented` is an exclusive radiogroup. Items are buttons with `role="radio"`
and a roving tabindex, so arrow keys work even when the platform does not move
between radios. Left, right, up, and down all move, and the list wraps. Home
and End jump to the enabled ends. A disabled item is skipped.

```vue
<script setup lang="ts">
import { ref } from "vue";

import { Segmented, SegmentedItem } from "@vizejs/ui/segmented";

const period = ref("day");
</script>

<template>
  <Segmented v-model="period" label="Billing">
    <SegmentedItem value="day">Day</SegmentedItem>
    <SegmentedItem value="week">Week</SegmentedItem>
    <SegmentedItem value="month">Month</SegmentedItem>
  </Segmented>
</template>
```

`orientation` is a styling hook. It does not change which arrows work.
