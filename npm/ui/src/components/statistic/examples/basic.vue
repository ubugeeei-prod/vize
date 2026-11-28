<!-- Revenue and request-volume statistics, including a negative amount and compact notation. -->
<script setup lang="ts">
import { Statistic } from "../index.ts";
</script>

<template>
  <div>
    <Statistic title="Revenue" :value="1280" prefix="$" suffix="/mo" />
    <Statistic title="Refunds" :value="-240" prefix="$" />
    <Statistic title="Requests" :value="1200" notation="compact" :maximum-fraction-digits="1" />
  </div>
</template>
