<!-- Invoice facts as a named description list. -->
<script setup lang="ts">
import { Descriptions, DescriptionsItem } from "../index.ts";
</script>

<template>
  <Descriptions label="Invoice">
    <DescriptionsItem label="Status">Paid</DescriptionsItem>
    <DescriptionsItem label="Total">$1,280</DescriptionsItem>
  </Descriptions>
</template>
