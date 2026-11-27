<script setup lang="ts">
import "./stage-playground-styles.vue";
import "./workbench-styles.vue";
import AllLayers from "./all-layers.vue";
import PaneResizer from "./pane-resizer.vue";
import SurfaceView from "./surface-view.vue";
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import MonacoEditor from "../../shared/monaco-editor.vue";
import { DAVINCI_EXAMPLES } from "../../shared/presets/davinci";
import StageRail from "./stage-rail.vue";
import PassTimeline from "./pass-timeline.vue";
import DumpView from "./dump-view.vue";
import OutputView from "./output-view.vue";
import DumpDiffView from "./dump-diff-view.vue";
import RemarksPanel from "./remarks-panel.vue";
import type { TimelineStep } from "./ladder";
import { useStageLadder, type StageId } from "./useStageLadder";
import { stepKeyAction } from "./keys";
import { formatArg } from "./remarks";

const { compiler: compilerProp } = defineProps<{
  compiler: WasmModule | null;
}>();

const targets = ["dom", "ssr", "vapor"] as const;
const layout = ref<"all" | "focus">("all");
const notation = ref<"dsl" | "raw">("dsl");
const wrap = ref(false);
const overviewWidths = ref([1.05, 1, 1, 1, 1]);
const focusWidths = ref([1, 2]);
const isOverview = computed(() => layout.value === "all" && ladder.value !== null);
const paneColumns = computed(() =>
  (isOverview.value ? overviewWidths.value : focusWidths.value)
    .map((width, index) => `minmax(${index === 0 ? 180 : 160}px, ${width}fr)`)
    .join(" "),
);
function resizePanes(widths: number[]) {
  if (isOverview.value) overviewWidths.value = widths;
  else focusWidths.value = widths;
}
function resetWidths() {
  if (isOverview.value) overviewWidths.value = [1.05, 1, 1, 1, 1];
  else focusWidths.value = [1, 2];
}
function expandStage(id: StageId) {
  selectStage(id);
  layout.value = "focus";
}

const {
  theme,
  source,
  ladder,
  error,
  captureError,
  captureStatus,
  captureLabel,
  remarksNote,
  syntaxNote,
  outputs,
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
  focusSpan,
  overviewPages,
  surfaceUnits,
  revealSpan,
  clearFocus,
  selectTraceLine,
  hoverTraceLine,
  selectStage,
  selectPage,
  locateRemark,
  onCursor,
} = useStageLadder(() => compilerProp ?? getWasm());

// SSR currently captures all four levels in the same product run.
outputTarget.value = "ssr";

const showTabs = computed(
  () => rung.value !== null && (rung.value.pages.length > 1 || rung.value.id !== "l1"),
);

function toggleView(view: "diff" | "remarks") {
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

// Presenter keys: 1-4 jump to a stage, arrows walk the pass timeline.
function onKey(event: KeyboardEvent) {
  const action = stepKeyAction(event, ladder.value?.timeline ?? [], page.value?.key ?? null);
  if (!action) return;
  event.preventDefault();
  if (action.kind === "stage") {
    if (action.stage === "l4" || ladder.value?.rungs.some((r) => r.id === action.stage)) {
      selectStage(action.stage as StageId);
    }
  } else selectStep(action.step);
}

onMounted(() => window.addEventListener("keydown", onKey));
onUnmounted(() => window.removeEventListener("keydown", onKey));

function snippet(text: string): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > 72 ? `${flat.slice(0, 71)}…` : flat;
}
</script>

<template>
  <div :class="['davinci-playground', 'dv-workbench', { 'dv-wrap': wrap }]">
    <header class="dv-toolbar">
      <div class="dv-brand">
        <h2>Davinci <span>IR Observatory</span></h2>
      </div>
      <div class="dv-controls">
        <div class="dv-toggle" aria-label="Compile target">
          <button
            v-for="target in targets"
            :key="target"
            type="button"
            :class="{ active: outputTarget === target }"
            :aria-pressed="outputTarget === target"
            @click="outputTarget = target"
          >
            {{ target.toUpperCase() }}
          </button>
        </div>
        <div class="dv-toggle" aria-label="Layout">
          <button
            type="button"
            :class="{ active: layout === 'all' }"
            :aria-pressed="layout === 'all'"
            @click="layout = 'all'"
          >
            All layers</button
          ><button
            type="button"
            :class="{ active: layout === 'focus' }"
            :aria-pressed="layout === 'focus'"
            @click="layout = 'focus'"
          >
            Focus
          </button>
        </div>
        <div class="dv-toggle" aria-label="Dump notation">
          <button
            type="button"
            :class="{ active: notation === 'dsl' }"
            :aria-pressed="notation === 'dsl'"
            @click="notation = 'dsl'"
          >
            IR DSL</button
          ><button
            type="button"
            :class="{ active: notation === 'raw' }"
            :aria-pressed="notation === 'raw'"
            @click="notation = 'raw'"
          >
            Raw
          </button>
        </div>
        <button
          type="button"
          :class="['davinci-ghost', { active: wrap }]"
          :aria-pressed="wrap"
          @click="wrap = !wrap"
        >
          Wrap
        </button>
      </div>
    </header>
    <div class="dv-run-strip">
      <span class="dv-live-dot" aria-hidden="true"></span
      ><span class="davinci-badge">{{ captureLabel ?? "Compiling" }}</span
      ><span v-if="ladder"
        >{{ ladder.rungs.length + (ladder.l4Pages.length ? 1 : 0) }} layers <b>·</b>
        {{ ladder.timeline.length }} steps <b>·</b>
        {{ remarksNote ? "remarks unavailable" : `${remarks.length} remarks` }}</span
      ><span class="dv-run-hint"
        >Hover to trace <b>·</b> Click to pin <b>·</b> Drag dividers to resize</span
      >
    </div>
    <section class="dv-trace-inspector" aria-label="Source correspondence">
      <div class="dv-trace-heading">
        <span class="dv-trace-label">TRACE</span
        ><span v-if="focusSource" class="davinci-span"
          >L{{ focusSource.line }} · @{{ focusSource.span.start }}..{{ focusSource.span.end }}</span
        >
        <div class="dv-trace-content">
          <code v-if="focusSource" class="dv-trace-source">{{ snippet(focusSource.text) }}</code>
          <span v-else class="davinci-hint"
            >Hover an analysis unit to trace it across layers. Click to pin.</span
          >
          <span
            v-for="record in focusProvenance"
            :key="`${record.span.start}:${record.span.end}:${record.rule}:${record.node}`"
            :class="['davinci-why', { fact: record.rule.startsWith('pass.') }]"
            :title="`${record.before} → ${record.after}`"
            >{{ record.rule }} <span class="dv-rule-arrow">→</span>
            {{ snippet(record.after) }}</span
          >
          <span
            v-for="(remark, index) in focusRemarks"
            :key="index"
            :class="['davinci-why', 'remark', remark.kind]"
            :title="remark.args.map(formatArg).join(' ')"
            >{{ remark.kind }} {{ remark.name }}</span
          >
        </div>
        <button v-if="focusSpan" type="button" class="davinci-ghost" @click="clearFocus">
          Clear trace
        </button>
      </div>
      <div v-if="syntaxNote" class="dv-trace-note">{{ syntaxNote }}</div>
      <div v-if="ladder?.unplaced.length" class="dv-trace-note">
        Additional captured pages: {{ ladder.unplaced.join(", ") }}
      </div>
      <p class="dv-dump-notice">
        Diagnostic dumps are not a stable interface. Do not use the DSL or raw format as an API or
        depend on their structure in external tools.
      </p>
    </section>
    <div
      :class="['dv-workspace', { 'dv-overview': isOverview }]"
      :style="{ gridTemplateColumns: paneColumns }"
    >
      <section class="davinci-source" data-pane="source" aria-label="Source">
        <div class="davinci-bar">
          <span class="dv-source-tag">SFC</span>
          <h2 class="davinci-title">Source</h2>
          <div class="davinci-example">
            <label class="dv-sr-only" :for="exampleId">Example</label>
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

      <AllLayers
        v-if="layout === 'all' && ladder"
        :pages="overviewPages"
        :ladder
        :span="focusSpan"
        :reveal="revealSpan"
        :surface-units="surfaceUnits"
        :target="outputTarget"
        :outputs
        :theme
        :stage
        :notation
        @page="selectPage"
        @select="selectTraceLine"
        @preview="hoverTraceLine"
        @expand="expandStage"
        @target="outputTarget = $event"
        @resize="resizePanes"
        @reset-widths="resetWidths"
      />
      <section v-else class="davinci-stages" data-pane="focus" aria-label="Davinci stage ladder">
        <PaneResizer
          label="Resize Source and stage panes"
          @resize="resizePanes"
          @reset="resetWidths"
        />
        <div class="davinci-bar">
          <h2 class="davinci-title">Davinci stage ladder</h2>
          <span v-if="captureLabel" class="davinci-badge" title="Same-run product capture">{{
            captureLabel
          }}</span>
          <span v-if="captureStatus" class="davinci-badge" title="Product capture schema_version"
            >feed v2</span
          >
          <span class="davinci-hint davinci-keys">Keys 1–4 pick a stage, ← → walk the steps</span>
        </div>

        <div class="davinci-subtabs" role="tablist" aria-label="Compile target">
          <button
            v-for="target in targets"
            :key="target"
            type="button"
            role="tab"
            :class="['davinci-subtab', { active: outputTarget === target }]"
            :aria-selected="outputTarget === target"
            @click="() => (outputTarget = target)"
          >
            {{ target.toUpperCase() }}
          </button>
        </div>

        <div v-if="error" class="davinci-message error" role="alert">{{ error }}</div>
        <div v-else-if="captureError" class="davinci-body">
          <div class="davinci-message error" role="alert">{{ captureError }}</div>
          <OutputView v-model:target="outputTarget" :outputs :theme />
        </div>
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
            <span v-if="remarksNote" class="davinci-hint">Remarks unavailable</span>
            <button
              v-else
              type="button"
              :class="['davinci-subtab', { active: pageView === 'remarks' }]"
              :aria-pressed="pageView === 'remarks'"
              @click="() => toggleView('remarks')"
            >
              Remarks <span class="davinci-count">{{ remarks.length }}</span>
            </button>
          </div>

          <div class="davinci-body">
            <OutputView
              v-if="stage === 'l4'"
              v-model:target="outputTarget"
              :outputs
              :theme
              :backend-pages="ladder.l4Pages"
            />
            <RemarksPanel v-else-if="pageView === 'remarks'" :remarks @locate="locateRemark" />
            <DumpDiffView
              v-else-if="pageView === 'diff' && diff && previousPage && page"
              :diff
              :before="previousPage.label"
              :after="page.label"
            />
            <SurfaceView
              v-else-if="page?.kind === 'surface'"
              :text="page.text"
              :units="surfaceUnits"
              :span="focusSpan"
              :reveal="revealSpan"
              @preview="hoverTraceLine"
              @select="selectTraceLine('l1', page!, $event)"
            />
            <DumpView
              v-else-if="page"
              :lines
              :reveal="revealSpan"
              :notation
              :marks="lineMarks"
              :kind="page.kind"
              :selected="selectedLine"
              :linked="linkedLines"
              @highlight="($event) => (hoveredLine = $event)"
              @select="($event) => (selectedLine = $event)"
            />
            <div v-else class="davinci-message">This stage produced no page for the source.</div>
          </div>
        </template>
        <div v-else-if="captureStatus" class="davinci-body">
          <div class="davinci-message">
            {{ captureStatus }}. Native stage pages are unavailable for this target.
          </div>
          <OutputView v-model:target="outputTarget" :outputs :theme />
        </div>
        <div v-else class="davinci-message">Loading the compiler…</div>
      </section>
    </div>
  </div>
</template>
