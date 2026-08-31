export const textRendererFixtures = [
  {
    filename: "CodeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Code } from "./families/typography/code/code.ts";

const tone = ref<"accent" | "muted">("muted");
</script>

<template>
  <Code as="pre" size="lg" variant="block" :tone>
    <template #default="{ size, variant }">
      {{ size }} {{ variant }}
    </template>
  </Code>
</template>
`,
  },
  {
    filename: "HeadingConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Heading } from "./families/typography/heading/heading.ts";

const level = ref<1 | 2 | 3>(2);
</script>

<template>
  <Heading :level size="xl" weight="bold" tone="accent" truncate>
    <template #default="{ level: semanticLevel, size, tone }">
      {{ semanticLevel }} {{ size }} {{ tone }}
    </template>
  </Heading>
</template>
`,
  },
  {
    filename: "KbdConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Kbd } from "./families/typography/kbd/kbd.ts";

const tone = ref<"accent" | "muted">("accent");
</script>

<template>
  <Kbd as="span" size="lg" variant="shortcut" :tone>
    <template #default="{ size, variant }">
      {{ size }} {{ variant }}
    </template>
  </Kbd>
</template>
`,
  },
  {
    filename: "TextConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Text } from "./families/typography/text/text.ts";

const truncate = ref(true);
</script>

<template>
  <Text as="p" size="lg" weight="semibold" tone="accent" :truncate>
    <template #default="{ size, tone, truncate: isTruncated, weight }">
      {{ size }} {{ weight }} {{ tone }} {{ isTruncated }}
    </template>
  </Text>
</template>
`,
  },
] as const;
