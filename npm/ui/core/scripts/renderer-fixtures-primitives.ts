import { avatarRendererFixtures } from "./renderer-fixtures-avatar.ts";
import { spinnerRendererFixtures } from "./renderer-fixtures-spinner.ts";
import { textRendererFixtures } from "./renderer-fixtures-text.ts";
import { toggleGroupRendererFixtures } from "./renderer-fixtures-toggle-group.ts";

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
  ...avatarRendererFixtures,
  {
    filename: "BadgeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Badge } from "./badge.ts";

const count = ref(12);
</script>

<template>
  <Badge as="sup" tone="danger" variant="count">
    {{ count }}
  </Badge>
</template>
`,
  },
  {
    filename: "BlockUIConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { BlockUI } from "./block-ui.ts";

const blocked = ref(true);
</script>

<template>
  <BlockUI :blocked reason="saving" interaction="inert" announce="polite" label="Saving changes">
    <template #default="{ state, reason }">
      <span>{{ state }} {{ reason }}</span>
    </template>
  </BlockUI>
</template>
`,
  },
  {
    filename: "BlockquoteConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Blockquote } from "./blockquote.ts";

const cite = ref("https://example.com/source");
</script>

<template>
  <Blockquote :cite size="lg" tone="muted">
    <template #default="{ cite: source, size, tone }">
      {{ size }} {{ tone }} {{ source }}
    </template>
  </Blockquote>
</template>
`,
  },
  {
    filename: "CardConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Card } from "./card.ts";

const tone = ref<"neutral" | "accent" | "info" | "success" | "warning" | "danger">("info");
</script>

<template>
  <Card as="article" variant="panel" density="compact" :tone role="region" aria-label="Release summary">
    <template #default="{ density, variant }">
      <h2>{{ variant }}</h2>
      <p>{{ density }}</p>
    </template>
  </Card>
</template>
`,
  },
  ...textRendererFixtures,
  {
    filename: "ListConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { List } from "./list.ts";

const spacing = ref<"compact" | "normal" | "loose">("loose");
</script>

<template>
  <List as="ol" marker="decimal" :spacing tone="muted">
    <template #default="{ marker, spacing: gap, tone }">
      <li>{{ marker }} {{ gap }} {{ tone }}</li>
    </template>
  </List>
</template>
`,
  },
  {
    filename: "EmptyStateConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { EmptyState } from "./empty-state.ts";

const tone = ref<"info" | "warning">("info");
</script>

<template>
  <EmptyState as="article" density="compact" orientation="inline" :tone>
    <template #default="{ state }">
      <span>{{ state }}</span>
    </template>
  </EmptyState>
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
    filename: "RadioGroupConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { RadioGroup, RadioGroupItem } from "./radio-group.ts";

const value = ref<string | null>("weekly");
</script>

<template>
  <RadioGroup v-model="value" aria-label="Email frequency" name="frequency" orientation="horizontal" required>
    <label><RadioGroupItem value="daily" />Daily</label>
    <label><RadioGroupItem value="weekly" />Weekly</label>
  </RadioGroup>
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
  ...toggleGroupRendererFixtures,
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
  ...spinnerRendererFixtures,
] as const;
