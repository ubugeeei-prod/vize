<script setup lang="ts">
import { computed } from "vue";
import DumpView from "./dump-view.vue";
import OutputView from "./output-view.vue";
import TraceLinks from "./trace-links.vue";
import PaneResizer from "./pane-resizer.vue";
import SurfaceView from "./surface-view.vue";
import type { DumpLine } from "./dumpLines";
import type { LadderPage, Rung, RungId, StageLadder } from "./ladder";
import type { Range } from "./offsets";
import type { CodeOutputs } from "../atelier/codeOutputs";
import type { OutputTarget, StageId } from "./useStageLadder";
import { traceMatches } from "./sourceTrace";
import { linkedSurfaceUnits } from "./surfaceUnits";

const { pages, ladder, span, reveal, surfaceUnits, target, outputs, theme, stage, notation } =
  defineProps<{
    pages: { rung: Rung; page: LadderPage; lines: DumpLine[]; marks: Map<number, string> }[];
    ladder: StageLadder;
    span: Range | null;
    reveal: Range | null;
    surfaceUnits: DumpLine[];
    target: OutputTarget;
    outputs: CodeOutputs;
    theme: "dark" | "light";
    stage: StageId;
    notation: "dsl" | "raw";
  }>();

const emit = defineEmits<{
  page: [RungId, string];
  select: [RungId, LadderPage, DumpLine | null];
  preview: [DumpLine | null];
  expand: [StageId];
  target: [OutputTarget];
  resize: [widths: number[]];
  resetWidths: [];
}>();

const definitions = [
  { id: "l1", name: "Surface", description: "Lossless authored template", dialect: "vue.template" },
  { id: "l2", name: "Disegno", description: "Semantic operations & passes", dialect: "ui.*" },
  { id: "l3", name: "Impeto", description: "Reactive graph & partition", dialect: "l3.*" },
] as const;
const layers = computed(() =>
  definitions.map((definition) => {
    const data = pages.find(({ rung }) => rung.id === definition.id);
    return {
      ...definition,
      data,
      linked:
        definition.id === "l1"
          ? [...linkedSurfaceUnits(surfaceUnits, span)]
          : data
            ? traceMatches(data.lines, span, data.page.kind)
            : [],
    };
  }),
);

function changePage(id: RungId, event: Event) {
  emit("page", id, (event.target as HTMLSelectElement).value);
}
</script>

<template>
  <div class="davinci-layers" aria-label="All intermediate representations">
    <TraceLinks :span :revision="`${notation}:${pages.map(({ page }) => page.key).join(',')}`" />
    <section
      v-for="layer in layers"
      :key="layer.id"
      :class="[
        'dv-layer',
        `dv-${layer.id}`,
        { 'dv-traced': layer.linked.length > 0, 'dv-active': stage === layer.id },
      ]"
      :data-layer="layer.id"
      :data-pane="layer.id"
      :aria-label="`${layer.id.toUpperCase()} ${layer.name}`"
    >
      <PaneResizer
        :label="`Resize ${layer.name} panes`"
        @resize="emit('resize', $event)"
        @reset="emit('resetWidths')"
      />
      <div class="dv-layer-header">
        <div class="dv-layer-heading">
          <h3>{{ layer.id.toUpperCase() }} IR</h3>
          <span class="dv-layer-alias">{{ layer.name }}</span>
          <button
            type="button"
            class="dv-expand"
            :aria-label="`Expand ${layer.id.toUpperCase()} IR`"
            @click="emit('expand', layer.id)"
          >
            ↗
          </button>
        </div>
        <p>{{ layer.description }}</p>
        <div class="dv-layer-facts">
          <code>{{ layer.dialect }}</code
          ><span>{{
            layer.id === "l1"
              ? `${surfaceUnits.length} analysis units`
              : (layer.data?.rung.facts.join(" · ") ?? "not captured")
          }}</span>
        </div>
      </div>
      <template v-if="layer.data">
        <div class="dv-page-picker">
          <select
            class="davinci-select"
            :aria-label="`${layer.id.toUpperCase()} IR dump`"
            :value="layer.data.page.key"
            @change="changePage(layer.id, $event)"
          >
            <option v-for="page in layer.data.rung.pages" :key="page.key" :value="page.key">
              {{ page.label }}
            </option>
          </select>
          <span :class="['dv-match-count', { matched: layer.linked.length }]"
            >{{ layer.linked.length }} linked</span
          >
        </div>
        <SurfaceView
          v-if="layer.id === 'l1'"
          :text="layer.data.page.text"
          :units="surfaceUnits"
          :span
          :reveal
          @preview="emit('preview', $event)"
          @select="emit('select', layer.id, layer.data!.page, $event)"
        />
        <DumpView
          v-else
          :notation
          :reveal
          :lines="layer.data.lines"
          :kind="layer.data.page.kind"
          :marks="layer.data.marks"
          :selected="null"
          :linked="layer.linked"
          @highlight="emit('preview', $event === null ? null : layer.data!.lines[$event])"
          @select="
            emit(
              'select',
              layer.id,
              layer.data!.page,
              $event === null ? null : layer.data!.lines[$event],
            )
          "
        />
        <div class="dv-layer-footer">
          <span>{{ layer.data.lines.length }} lines</span>
          <span v-if="layer.data.marks.size" class="dv-partition-legend"
            ><span class="dv-static-dot">●</span>
            {{ [...layer.data.marks.values()].filter((kind) => kind === "static").length }} static
            <span class="dv-dynamic-dot">●</span>
            {{ [...layer.data.marks.values()].filter((kind) => kind === "dynamic").length }}
            dynamic</span
          >
          <span v-else
            >{{
              layer.id === "l1"
                ? surfaceUnits.length
                : layer.data.lines.filter((line) => line.span).length
            }}
            source spans</span
          >
        </div>
      </template>
      <div v-else class="dv-layer-empty">
        <span class="dv-empty-glyph">∅</span>
        <strong>No {{ layer.id.toUpperCase() }} capture</strong>
        <p>This {{ target.toUpperCase() }} pipeline does not expose this layer.</p>
        <button
          v-if="layer.id === 'l3' && target !== 'ssr'"
          type="button"
          class="davinci-ghost"
          @click="emit('target', 'ssr')"
        >
          Inspect the SSR pipeline →
        </button>
      </div>
    </section>
    <section class="dv-layer dv-l4" data-layer="l4" data-pane="l4" aria-label="L4 Emit">
      <PaneResizer
        label="Resize Emit panes"
        @resize="emit('resize', $event)"
        @reset="emit('resetWidths')"
      />
      <div class="dv-layer-header">
        <div class="dv-layer-heading">
          <h3>L4 IR</h3>
          <span class="dv-layer-alias">Emit</span>
          <button
            type="button"
            class="dv-expand"
            aria-label="Expand L4 IR"
            @click="emit('expand', 'l4')"
          >
            ↗
          </button>
        </div>
        <p>Backend code & assembled module</p>
        <div class="dv-layer-facts">
          <code>{{ target }}.emit</code
          ><span
            >{{ ladder.l4Pages.length }} backend page{{
              ladder.l4Pages.length === 1 ? "" : "s"
            }}</span
          >
        </div>
      </div>
      <OutputView
        :target
        :outputs
        :theme
        :backend-pages="ladder.l4Pages"
        hide-targets
        @update:target="emit('target', $event)"
      />
      <div
        class="dv-layer-footer"
        title="The capture exposes emitted code without per-operation source spans."
      >
        <span>Same compile run</span><span>No span feed</span>
      </div>
    </section>
  </div>
</template>
