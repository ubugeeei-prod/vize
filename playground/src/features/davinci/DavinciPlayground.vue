<script setup lang="ts">
import "./DavinciPlayground.css";
import "./StageRail.css";
import "./FolioView.css";
import { computed } from "vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import { DAVINCI_PRESET } from "../../shared/presets/davinci";
import StageRail from "./StageRail.vue";
import PassTimeline from "./PassTimeline.vue";
import FolioView from "./FolioView.vue";
import OutputView from "./OutputView.vue";
import type { TimelineStep } from "./ladder";
import { useDavinciLadder } from "./useDavinciLadder";

const props = defineProps<{
  compiler: WasmModule | null;
}>();

const {
  theme,
  source,
  ladder,
  error,
  outputs,
  ladderTime,
  stage,
  rung,
  page,
  lines,
  linkedLines,
  outputTarget,
  selectedLine,
  hoveredLine,
  focusSource,
  highlights,
  selectStage,
  selectPage,
  onCursor,
} = useDavinciLadder(() => props.compiler ?? getWasm());

const pageTabs = computed(() =>
  rung.value && rung.value.pages.length > 1 ? rung.value.pages : [],
);

function selectStep(step: TimelineStep) {
  selectPage(step.rung, step.key);
}

function snippet(text: string): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > 72 ? `${flat.slice(0, 71)}…` : flat;
}
</script>

<template>
  <div class="davinci-playground">
    <section class="davinci-source" aria-label="Source">
      <header class="davinci-bar">
        <h2 class="davinci-title">Source</h2>
        <span class="davinci-hint">Put the cursor on markup to find it in the stage page</span>
        <button type="button" class="davinci-ghost" @click="source = DAVINCI_PRESET">Reset</button>
      </header>
      <div class="davinci-editor">
        <MonacoEditor v-model="source" language="vue" :highlights :theme @cursor="onCursor" />
      </div>
    </section>

    <section class="davinci-stages" aria-label="Davinci stage ladder">
      <header class="davinci-bar">
        <h2 class="davinci-title">Davinci stage ladder</h2>
        <span v-if="ladderTime !== null" class="davinci-badge" title="analyzeSfc wall time"
          >{{ ladderTime.toFixed(2) }} ms</span
        >
        <span class="davinci-badge" title="Spolvero feed schema_version">feed v1</span>
      </header>

      <div v-if="error" class="davinci-message error" role="alert">{{ error }}</div>
      <template v-else-if="ladder">
        <StageRail :rungs="ladder.rungs" :selected="stage" @select="selectStage" />
        <PassTimeline :steps="ladder.timeline" :current="page?.key ?? null" @select="selectStep" />

        <div v-if="pageTabs.length > 0" class="davinci-subtabs" role="tablist" aria-label="Pages">
          <button
            v-for="tab in pageTabs"
            :key="tab.key"
            type="button"
            role="tab"
            :class="['davinci-subtab', { active: page?.key === tab.key }]"
            :aria-selected="page?.key === tab.key"
            @click="selectPage(rung!.id, tab.key)"
          >
            {{ tab.label }}
          </button>
        </div>

        <div class="davinci-body">
          <OutputView v-if="stage === 's4'" v-model:target="outputTarget" :outputs :theme />
          <FolioView
            v-else-if="page"
            :lines
            :kind="page.kind"
            :selected="selectedLine"
            :linked="linkedLines"
            @hover="hoveredLine = $event"
            @select="selectedLine = $event"
          />
          <div v-else class="davinci-message">This stage produced no page for the source.</div>
        </div>

        <footer class="davinci-provenance-bar" aria-live="polite">
          <template v-if="focusSource">
            <span class="davinci-span"
              >{{ focusSource.span.start }}–{{ focusSource.span.end }}</span
            >
            <code class="davinci-snippet">{{ snippet(focusSource.text) }}</code>
          </template>
          <span v-else-if="stage !== 's4'" class="davinci-hint"
            >Point at a line to see the authored source it came from</span
          >
          <span v-else class="davinci-hint"
            >Emitted by the same compiler build, from the same source</span
          >
          <span v-if="ladder.unplaced.length" class="davinci-hint"
            >Unplaced pages: {{ ladder.unplaced.join(", ") }}</span
          >
        </footer>
      </template>
      <div v-else class="davinci-message">Loading the compiler…</div>
    </section>
  </div>
</template>
