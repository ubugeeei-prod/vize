/** Layout component fixtures compiled by every supported renderer lane. */
export const layoutRendererFixtures = [
  {
    filename: "ClusterConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Cluster } from "./families/layout/cluster/cluster.ts";

const wrap = ref(true);
</script>

<template>
  <Cluster as="nav" gap="0.75rem" align="center" justify="space-between" :wrap>
    <template #default="{ direction, wrapMode }">
      <span>{{ direction }} {{ wrapMode }}</span>
      <button type="button">Apply</button>
    </template>
  </Cluster>
</template>
`,
  },
  {
    filename: "ContainerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Container } from "./families/layout/container/container.ts";

const centered = ref(true);
</script>

<template>
  <Container as="main" size="lg" :centered padding-inline="1rem">
    <template #default="{ maxInlineSize, paddingInline }">
      <h1>{{ maxInlineSize }}</h1>
      <p>{{ paddingInline }}</p>
    </template>
  </Container>
</template>
`,
  },
  {
    filename: "GridConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Grid } from "./families/layout/grid/grid.ts";

const columns = ref(3);
</script>

<template>
  <Grid as="section" :columns gap="0.75rem" align="center" justify="stretch" auto-flow="row dense">
    <template #default="{ autoFlow, columns: resolvedColumns }">
      <article>{{ resolvedColumns }}</article>
      <article>{{ autoFlow }}</article>
    </template>
  </Grid>
</template>
`,
  },
  {
    filename: "SpacerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Spacer } from "./families/layout/spacer/spacer.ts";

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
import { Stack } from "./families/layout/stack/stack.ts";

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
  {
    filename: "ScrollAreaConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { ScrollArea } from "./families/layout/scroll-area/scroll-area.ts";

const orientation = ref<"vertical" | "horizontal" | "both">("both");
</script>

<template>
  <ScrollArea
    aria-label="Activity"
    block-size="12rem"
    dir="rtl"
    focusable
    :orientation
    overscroll-behavior="contain"
    scrollbar-gutter="stable"
    scrollbar-width="thin"
  >
    <template #default="{ dir, overflowX, overflowY }">
      <p>{{ dir }} {{ overflowX }} {{ overflowY }}</p>
    </template>
  </ScrollArea>
</template>
`,
  },
  {
    filename: "SurfaceConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Surface } from "./families/layout/surface/surface.ts";

const tone = ref<"neutral" | "muted" | "accent" | "info" | "success" | "warning" | "danger">(
  "accent",
);
</script>

<template>
  <Surface
    aria-describedby="release-help"
    aria-labelledby="release-title"
    as="article"
    elevation="floating"
    :tone
  >
    <template #default="{ as, labelled, described }">
      <h2 id="release-title">{{ as }} {{ labelled }}</h2>
      <p id="release-help">{{ described }}</p>
    </template>
  </Surface>
</template>
`,
  },
] as const;
