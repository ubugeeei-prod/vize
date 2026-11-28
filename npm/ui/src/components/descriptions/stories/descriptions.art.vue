<script setup lang="ts">
import { Descriptions, DescriptionsItem } from "../index.ts";
</script>

<art title="Descriptions" status="ready">
  <variant name="Default" default>
    <Descriptions label="Invoice">
      <DescriptionsItem label="Status">Paid</DescriptionsItem>
      <DescriptionsItem label="Total">$1,280</DescriptionsItem>
    </Descriptions>
  </variant>
  <variant name="Single fact">
    <Descriptions label="Build">
      <DescriptionsItem label="Result">Green</DescriptionsItem>
    </Descriptions>
  </variant>
</art>
