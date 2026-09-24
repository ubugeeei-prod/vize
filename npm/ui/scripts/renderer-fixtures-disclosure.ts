export const disclosureRendererFixtures = [
  {
    filename: "AccordionConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import {
  AccordionContent,
  AccordionHeader,
  AccordionItem,
  AccordionRoot,
  AccordionTrigger,
} from "./families/disclosure/accordion/accordion.ts";

const open = ref<readonly ("shipping" | "returns")[]>(["shipping"]);
</script>

<template>
  <AccordionRoot v-model="open" type="multiple" id="renderer-accordion" hidden-until-found>
    <AccordionItem v-for="value in ['shipping', 'returns'] as const" :key="value" :value>
      <AccordionHeader>
        <AccordionTrigger v-slot="{ state }">{{ value }} ({{ state }})</AccordionTrigger>
      </AccordionHeader>
      <AccordionContent>{{ value }} details</AccordionContent>
    </AccordionItem>
  </AccordionRoot>
</template>
`,
  },
  {
    filename: "HoverCardConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  HoverCardArrow,
  HoverCardContent,
  HoverCardRoot,
  HoverCardTrigger,
} from "./families/overlays/hover-card/hover-card.ts";
</script>

<template>
  <p>
    Written by
    <HoverCardRoot id="renderer-hover-card" :open-delay="400" touch-behavior="long-press">
      <HoverCardTrigger href="/users/ada">@ada</HoverCardTrigger>
      <HoverCardContent v-slot="{ reason }" portal-disabled placement="top">
        Ada Lovelace ({{ reason }})
        <HoverCardArrow />
      </HoverCardContent>
    </HoverCardRoot>
  </p>
</template>
`,
  },
] as const;
