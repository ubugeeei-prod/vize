<!-- Single-open collapsible accordion bound with v-model to the open item. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
} from "../accordion.ts";

const open = ref<string | null>("shipping");
</script>

<template>
  <AccordionRoot v-model="open" type="single" collapsible>
    <AccordionItem value="shipping">
      <AccordionHeader>
        <AccordionTrigger>Shipping</AccordionTrigger>
      </AccordionHeader>
      <AccordionContent>Orders ship within two business days.</AccordionContent>
    </AccordionItem>
    <AccordionItem value="returns">
      <AccordionHeader>
        <AccordionTrigger>Returns</AccordionTrigger>
      </AccordionHeader>
      <AccordionContent>Return unused items within 30 days for a full refund.</AccordionContent>
    </AccordionItem>
    <AccordionItem value="warranty" disabled>
      <AccordionHeader>
        <AccordionTrigger>Warranty (coming soon)</AccordionTrigger>
      </AccordionHeader>
      <AccordionContent>Warranty details are not available yet.</AccordionContent>
    </AccordionItem>
  </AccordionRoot>
</template>
