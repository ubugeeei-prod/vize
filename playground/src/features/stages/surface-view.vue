<script setup lang="ts">
import "./dump-view-styles.vue";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import type { DumpLine } from "./dumpLines";
import type { Range } from "./offsets";
import { surfaceLines, linkedSurfaceUnits } from "./surfaceUnits";
import { withTokenOffsets } from "../../utils/withTokenOffsets";

const { text, units, span, reveal } = defineProps<{
  text: string;
  units: DumpLine[];
  span: Range | null;
  reveal: Range | null;
}>();
const emit = defineEmits<{ preview: [DumpLine | null]; select: [DumpLine] }>();
const root = useTemplateRef<HTMLElement>("root");
const lines = computed(() => surfaceLines(text, units));
const linked = computed(() => linkedSurfaceUnits(units, span));

watch(
  () => [reveal, text, units],
  async () => {
    const unit = [...linkedSurfaceUnits(units, reveal)][0];
    if (unit === undefined) return;
    await nextTick();
    root.value
      ?.querySelector(`[data-unit="${unit}"]`)
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  },
  { immediate: true },
);
</script>

<template>
  <div ref="root" class="davinci-folio page-surface" aria-label="L1 IR surface">
    <div
      v-for="line in lines"
      :key="line.index"
      :class="[
        'davinci-line',
        { linked: line.fragments.some(({ unit }) => unit && linked.has(unit.index)) },
      ]"
      :data-line="line.index"
    >
      <span class="davinci-mark" aria-hidden="true"></span>
      <span class="davinci-ln">{{ line.index + 1 }}</span>
      <span class="davinci-code"
        ><template v-for="fragment in line.fragments" :key="fragment.offset"
          ><button
            v-if="fragment.unit"
            type="button"
            :class="['dv-surface-unit', { linked: linked.has(fragment.unit.index) }]"
            :data-unit="fragment.unit.index"
            :data-span-start="fragment.unit.span!.start"
            :data-span-end="fragment.unit.span!.end"
            :title="`${fragment.unit.text.trim().split(' ')[0]} · authored bytes ${fragment.unit.span!.start}–${fragment.unit.span!.end}`"
            @mouseenter="emit('preview', fragment.unit)"
            @mouseleave="emit('preview', null)"
            @focus="emit('preview', fragment.unit)"
            @blur="emit('preview', null)"
            @click="emit('select', fragment.unit)"
          >
            <span
              v-for="token in withTokenOffsets(fragment.tokens)"
              :key="token.offset"
              :class="`tk-${token.type}`"
              >{{ token.text }}</span
            ></button
          ><span v-else>{{ fragment.text }}</span></template
        ><template v-if="!line.fragments.length">&#160;</template></span
      >
    </div>
  </div>
</template>
