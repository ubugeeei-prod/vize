/** Layout component fixtures compiled by every supported renderer lane. */
export const layoutRendererFixtures = [
  {
    filename: "ClusterConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Cluster } from "./cluster.ts";

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
import { Container } from "./container.ts";

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
import { Grid } from "./grid.ts";

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
