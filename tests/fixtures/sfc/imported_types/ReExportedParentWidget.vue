<script lang="ts">
import type { ReExportedBaseProps } from "./re-exported";

export interface ReExportedParentWidgetProps extends ReExportedBaseProps {}
</script>

<script setup lang="ts">
import ReExportedChild from "./ReExportedChild.vue";

const props = defineProps<ReExportedParentWidgetProps>();
</script>

<template>
  <ReExportedChild :as="as" :as-child="props.asChild" />
</template>
