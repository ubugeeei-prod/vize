export const primitiveRendererFixtures = [
  {
    filename: "AlertDialogConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./alert-dialog.ts";
</script>

<template>
  <AlertDialogRoot>
    <AlertDialogTrigger>Delete</AlertDialogTrigger>
    <AlertDialogPortal>
      <AlertDialogOverlay />
      <AlertDialogContent>
        <AlertDialogTitle>Delete project?</AlertDialogTitle>
        <AlertDialogDescription>This action cannot be undone.</AlertDialogDescription>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction>Delete</AlertDialogAction>
      </AlertDialogContent>
    </AlertDialogPortal>
  </AlertDialogRoot>
</template>
`,
  },
  {
    filename: "AspectRatioConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { AspectRatio } from "./aspect-ratio.ts";

const ratio = ref(16 / 9);
</script>

<template>
  <AspectRatio as="figure" :ratio>
    <template #default="{ invalid, ratio: normalizedRatio }">
      <img alt="" src="/poster.png" :data-invalid="invalid || undefined" />
      <figcaption>{{ normalizedRatio }}</figcaption>
    </template>
  </AspectRatio>
</template>
`,
  },
  {
    filename: "MeterConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Meter } from "./meter.ts";

const value = ref(64);
</script>

<template>
  <Meter aria-label="Storage usage" :value :min="0" :max="100" :low="30" :high="90" :optimum="50">
    <template #default="{ percent, state }">
      <span>{{ percent }} {{ state }}</span>
    </template>
  </Meter>
</template>
`,
  },
  {
    filename: "CollapsibleConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "./collapsible.ts";

const open = ref(false);
</script>

<template>
  <CollapsibleRoot id="filters" v-model:open="open">
    <template #default="{ state }">
      <CollapsibleTrigger aria-label="Filters">
        <span>{{ state }}</span>
      </CollapsibleTrigger>
      <CollapsibleContent aria-describedby="filters-help">
        <p id="filters-help">Filter controls</p>
      </CollapsibleContent>
    </template>
  </CollapsibleRoot>
</template>
`,
  },
  {
    filename: "SeparatorConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Separator } from "./separator.ts";

const orientation = ref<"horizontal" | "vertical">("vertical");
</script>

<template>
  <Separator as="div" :orientation aria-label="Pane boundary" />
</template>
`,
  },
  {
    filename: "SkeletonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Skeleton } from "./skeleton.ts";

const loading = ref(true);
</script>

<template>
  <Skeleton aria-label="Loading profile" as="section" :loading block-size="2rem">
    <template #default="{ state }">
      <span>{{ state }}</span>
    </template>
  </Skeleton>
</template>
`,
  },
  {
    filename: "SpacerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Spacer } from "./spacer.ts";

const blockSize = ref("2rem");
</script>

<template>
  <Spacer as="div" :block-size="blockSize" inline-size="100%" />
</template>
`,
  },
  {
    filename: "StackConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Stack } from "./stack.ts";

const axis = ref<"block" | "inline">("inline");
</script>

<template>
  <Stack as="section" :axis gap="1rem" align="center" justify="space-between">
    <template #default="{ direction }">
      <span>{{ direction }}</span>
      <button type="button">Continue</button>
    </template>
  </Stack>
</template>
`,
  },
] as const;
