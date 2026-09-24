export const a11yI18nRendererFixtures = [
  {
    filename: "A11yAuditConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useTemplateRef } from "vue";

import { useA11yAudit } from "./families/accessibility/a11y-audit/a11y-audit.ts";

const root = useTemplateRef<HTMLDivElement>("root");
const audit = useA11yAudit(root, { ignore: ["image-alt"] });
</script>

<template>
  <div ref="root" :data-issues="audit.issues.value.length">
    <button type="button" aria-label="Save">S</button>
  </div>
</template>
`,
  },
  {
    filename: "DirectionConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { DirectionProvider } from "./families/i18n/direction/direction.ts";
</script>

<template>
  <DirectionProvider v-slot="{ dir }" dir="rtl" as="section">
    <p>{{ dir }}</p>
  </DirectionProvider>
</template>
`,
  },
] as const;
