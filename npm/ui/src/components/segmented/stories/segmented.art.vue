<script setup lang="ts">
import { Segmented, SegmentedItem } from "../index.ts";
</script>

<art title="Segmented" status="ready">
  <variant name="Default" default>
    <Segmented model-value="day" label="Billing">
      <SegmentedItem value="day">Day</SegmentedItem>
      <SegmentedItem value="week">Week</SegmentedItem>
      <SegmentedItem value="month">Month</SegmentedItem>
    </Segmented>
  </variant>
  <variant name="Vertical">
    <Segmented model-value="week" label="Billing" orientation="vertical">
      <SegmentedItem value="day">Day</SegmentedItem>
      <SegmentedItem value="week">Week</SegmentedItem>
    </Segmented>
  </variant>
  <variant name="Disabled">
    <Segmented model-value="day" label="Billing" disabled>
      <SegmentedItem value="day">Day</SegmentedItem>
      <SegmentedItem value="week">Week</SegmentedItem>
    </Segmented>
  </variant>
</art>
