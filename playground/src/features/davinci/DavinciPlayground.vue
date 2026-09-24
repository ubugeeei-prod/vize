<script setup lang="ts">
import "./DavinciPlayground.css";
import "./StageRail.css";
import "./FolioView.css";
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import { DAVINCI_EXAMPLES } from "../../shared/presets/davinci";
import StageRail from "./StageRail.vue";
import PassTimeline from "./PassTimeline.vue";
import FolioView from "./FolioView.vue";
import OutputView from "./OutputView.vue";
import FolioDiffView from "./FolioDiffView.vue";
import RemarksPanel from "./RemarksPanel.vue";
import FlameView from "./FlameView.vue";
import type { TimelineStep } from "./ladder";
import { useDavinciLadder, type StageId } from "./useDavinciLadder";
import { stepKeyAction } from "./keys";
import { formatArg } from "./remarks";

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
  profileNote,
  profile,
  baseline,
  pinBaseline,
  clearBaseline,
  stage,
  rung,
  page,
  lines,
  lineMarks,
  previousPage,
  diff,
  pageView,
  remarks,
  linkedLines,
  outputTarget,
  selectedLine,
  hoveredLine,
  focusSource,
  focusProvenance,
  focusRemarks,
  highlights,
  selectStage,
  selectPage,
  locateRemark,
  onCursor,
} = useDavinciLadder(() => props.compiler ?? getWasm());

const showTabs = computed(
  () => rung.value !== null && (rung.value.pages.length > 1 || rung.value.id !== "s1"),
);

function toggleView(view: "diff" | "remarks" | "flame") {
  pageView.value = pageView.value === view ? "page" : view;
}

const example = ref(DAVINCI_EXAMPLES[0].key);
const exampleId = useId();

function loadExample(key: string) {
  const found = DAVINCI_EXAMPLES.find((item) => item.key === key);
  if (found) source.value = found.code;
}

watch(example, loadExample);

function selectStep(step: TimelineStep) {
  selectPage(step.rung, step.key);
}

/** A flame frame opens its stage, or the step page its stage/pass names. */
function selectFrame([stageName, pass]: string[]) {
  const step = ladder.value?.timeline.find((s) => s.key === `${stageName}/${pass}`);
  if (step) selectStep(step);
  else if (ladder.value?.rungs.some((r) => r.id === stageName)) selectStage(stageName as StageId);
}

// Presenter keys: 1-4 jump to a stage, arrows walk the pass timeline.
function onKey(event: KeyboardEvent) {
  const action = stepKeyAction(event, ladder.value?.timeline ?? [], page.value?.key ?? null);
  if (!action) return;
  event.preventDefault();
  if (action.kind === "stage") selectStage(action.stage as StageId);
  else selectStep(action.step);
}

onMounted(() => window.addEventListener("keydown", onKey));
onUnmounted(() => window.removeEventListener("keydown", onKey));

function snippet(text: string): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > 72 ? `${flat.slice(0, 71)}…` : flat;
}
</script>

<template>
  <div class="davinci-playground">
    <section class="davinci-source" aria-label="Source">
      <div class="davinci-bar">
        <h2 class="davinci-title">Source</h2>
        <span class="davinci-hint">Put the cursor on markup to find it in the stage page</span>
        <div class="davinci-example">
          <label class="davinci-hint" :for="exampleId">Example</label>
          <select :id="exampleId" v-model="example" class="davinci-select">
            <option v-for="item in DAVINCI_EXAMPLES" :key="item.key" :value="item.key">
              {{ item.label }}
            </option>
          </select>
        </div>
        <button type="button" class="davinci-ghost" @click="() => loadExample(example)">
          Reset
        </button>
      </div>
      <div class="davinci-editor">
        <MonacoEditor v-model="source" language="vue" :highlights :theme @cursor="onCursor" />
      </div>
    </section>

    <section class="davinci-stages" aria-label="Davinci stage ladder">
      <div class="davinci-bar">
        <h2 class="davinci-title">Davinci stage ladder</h2>
        <span v-if="ladderTime !== null" class="davinci-badge" title="analyzeSfc wall time"
          >{{ ladderTime.toFixed(2) }} ms</span
        >
        <span class="davinci-badge" title="Spolvero feed schema_version">feed v1</span>
        <span class="davinci-hint davinci-keys">Keys 1–4 pick a stage, ← → walk the steps</span>
      </div>

      <div v-if="error" class="davinci-message error" role="alert">{{ error }}</div>
      <template v-else-if="ladder">
        <StageRail :rungs="ladder.rungs" :selected="stage" @select="selectStage" />
        <PassTimeline
          :steps="ladder.timeline"
          :walks="ladder.walks"
          :current="page?.key ?? null"
          @select="selectStep"
        />

        <div v-if="showTabs && rung" class="davinci-subtabs" role="tablist" aria-label="Pages">
          <button
            v-for="tab in rung.pages"
            :key="tab.key"
            type="button"
            role="tab"
            :class="['davinci-subtab', { active: pageView === 'page' && page?.key === tab.key }]"
            :aria-selected="page?.key === tab.key"
            @click="() => selectPage(rung.id, tab.key)"
          >
            {{ tab.label }}
          </button>
          <span class="davinci-subtabs-gap" aria-hidden="true"></span>
          <button
            v-if="previousPage"
            type="button"
            :class="['davinci-subtab', { active: pageView === 'diff' }]"
            :aria-pressed="pageView === 'diff'"
            @click="() => toggleView('diff')"
          >
            Diff vs {{ previousPage.label }}
          </button>
          <button
            type="button"
            :class="['davinci-subtab', { active: pageView === 'remarks' }]"
            :aria-pressed="pageView === 'remarks'"
            @click="() => toggleView('remarks')"
          >
            Remarks <span class="davinci-count">{{ remarks.length }}</span>
          </button>
          <button
            type="button"
            :class="['davinci-subtab', { active: pageView === 'flame' }]"
            :aria-pressed="pageView === 'flame'"
            @click="() => toggleView('flame')"
          >
            Flame
          </button>
        </div>

        <div class="davinci-body">
          <OutputView v-if="stage === 's4'" v-model:target="outputTarget" :outputs :theme />
          <RemarksPanel v-else-if="pageView === 'remarks'" :remarks @locate="locateRemark" />
          <FlameView
            v-else-if="pageView === 'flame'"
            :profile
            :baseline
            @select="selectFrame"
            @pin="pinBaseline"
            @unpin="clearBaseline"
          />
          <FolioDiffView
            v-else-if="pageView === 'diff' && diff && previousPage && page"
            :diff
            :before="previousPage.label"
            :after="page.label"
          />
          <FolioView
            v-else-if="page"
            :lines
            :marks="lineMarks"
            :kind="page.kind"
            :selected="selectedLine"
            :linked="linkedLines"
            @hover="($event) => (hoveredLine = $event)"
            @select="($event) => (selectedLine = $event)"
          />
          <div v-else class="davinci-message">This stage produced no page for the source.</div>
        </div>

        <footer class="davinci-provenance-bar" aria-live="polite">
          <template v-if="focusSource">
            <span class="davinci-span"
              >{{ focusSource.span.start }}–{{ focusSource.span.end }}</span
            >
            <code class="davinci-snippet">{{ snippet(focusSource.text) }}</code>
            <span
              v-for="record in focusProvenance"
              :key="`${record.span.start}:${record.span.end}:${record.rule}:${record.node}`"
              :class="['davinci-why', { fact: record.rule.startsWith('pass.') }]"
              :title="`${record.rule}: ${record.before} → ${record.after}`"
              >{{ record.rule
              }}<template v-if="record.rule.startsWith('pass.')">
                {{ snippet(record.after) }}</template
              ></span
            >
            <span
              v-for="(remark, index) in focusRemarks"
              :key="`remark-${index}`"
              :class="['davinci-why', 'remark', remark.kind]"
              :title="remark.args.map(formatArg).join(' ')"
              >{{ remark.kind }} {{ remark.name }}</span
            >
          </template>
          <span v-else-if="stage !== 's4'" class="davinci-hint"
            >Point at a line to see the authored source it came from</span
          >
          <span v-else class="davinci-hint"
            >Emitted by the same compiler build, from the same source</span
          >
          <span v-if="profileNote" class="davinci-hint"
            >Step timings unavailable: {{ profileNote }}</span
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
