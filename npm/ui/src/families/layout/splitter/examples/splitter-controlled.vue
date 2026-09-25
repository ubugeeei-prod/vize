<!-- Controlled splitter whose layout is bound with v-model:layout. -->
<script setup lang="ts">
import { ref } from "vue";

import { SplitterGroup, SplitterHandle, SplitterPanel } from "../splitter.ts";
import type { SplitterLayout } from "../splitter.ts";

const layout = ref<SplitterLayout>([60, 40]);
</script>

<template>
  <div>
    <SplitterGroup v-model:layout="layout">
      <SplitterPanel :min-size="25">
        <p>Editor</p>
      </SplitterPanel>
      <SplitterHandle aria-label="Resize terminal" />
      <SplitterPanel :min-size="15">
        <p>Terminal</p>
      </SplitterPanel>
    </SplitterGroup>
    <output>Editor {{ layout[0] }}% / Terminal {{ layout[1] }}%</output>
    <button type="button" @click="() => (layout = [50, 50])">Reset layout</button>
  </div>
</template>
