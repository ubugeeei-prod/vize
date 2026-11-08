<script setup lang="ts">
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
  profileNote,
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
} = useStageLadder(() => compilerProp ?? getWasm());

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
          <DumpView
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
          <span v-else-if="stage !== 'l4'" class="davinci-hint"
            >Point at a line to see the authored source it came from</span
          >
          <span v-else class="davinci-hint"
            >Module and stages came from the same compileSfc call</span
          >
          <span v-if="remarksNote" class="davinci-hint">{{ remarksNote }}</span>
          <span v-if="profileNote" class="davinci-hint">{{ profileNote }}</span>
          <span v-if="syntaxNote" class="davinci-hint">{{ syntaxNote }}</span>
          <span v-if="ladder.unplaced.length" class="davinci-hint"
            >Unplaced pages: {{ ladder.unplaced.join(", ") }}</span
          >
        </footer>
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
</template>

<style>
/* Davinci tab: source on the left, the stage ladder on the right. The one
   decorative device is the pounce line between stations - the pricked,
   dusted line a pounced transfer leaves - and everything else stays quiet. */

.davinci-playground {
  --dv-ochre: var(--color-notice);
  --dv-ochre-bg: var(--color-notice-bg);
  --dv-ochre-border: var(--color-notice-border);
  --dv-static: var(--color-teal);
  --dv-static-bg: var(--color-teal-bg);

  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: minmax(360px, 5fr) 7fr;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.davinci-source,
.davinci-stages {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.davinci-source {
  border-right: 1px solid var(--border-primary);
}

.davinci-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 48px;
  padding: 0 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.davinci-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.davinci-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.davinci-keys {
  margin-left: auto;
  white-space: nowrap;
}

.davinci-source .davinci-hint {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.davinci-example {
  display: flex;
  align-items: center;
  gap: 6px;
}

.davinci-select {
  padding: 4px 8px;
  font: inherit;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
}

.davinci-ghost {
  padding: 4px 10px;
  font-size: 12px;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
}

.davinci-ghost:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.davinci-badge {
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.davinci-editor {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Provenance highlight painted into Monaco (global: Monaco owns its DOM). */
.davinci-provenance {
  background: var(--color-notice-bg);
  outline: 1.5px solid var(--color-notice);
  outline-offset: -1px;
  border-radius: 2px;
}

/* Page tabs within a stage, and output targets */
.davinci-subtabs {
  display: flex;
  gap: 4px;
  padding: 10px 20px 0;
  flex-shrink: 0;
  overflow-x: auto;
}

/* A tab label never wraps: the row scrolls instead. */
.davinci-subtab {
  flex-shrink: 0;
  white-space: nowrap;
  padding: 6px 12px;
  font-size: 13px;
  font-family: var(--font-mono);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
}

.davinci-subtab:hover {
  color: var(--text-secondary);
  background: var(--bg-secondary);
}

.davinci-subtab.active {
  color: var(--text-primary);
  border-color: var(--border-primary);
  background: var(--bg-secondary);
}

.davinci-subtabs-gap {
  flex: 1;
}

.davinci-count {
  margin-left: 4px;
  padding: 0 6px;
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.davinci-body {
  flex: 1;
  min-height: 0;
  padding: 12px 20px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.davinci-output {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  flex: 1;
}

.davinci-output .davinci-subtabs {
  padding: 0;
}

.davinci-output .davinci-output-code.code-highlight {
  flex: 1;
  min-height: 0;
  --code-line-height: 22px;
  font-size: 14px;
}

.davinci-message {
  margin: 20px;
  padding: 14px 16px;
  font-size: 14px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
}

.davinci-message.error {
  color: var(--color-error);
  background: var(--color-error-bg);
  border-color: var(--color-error-border);
  white-space: pre-wrap;
}

.davinci-provenance-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 40px;
  padding: 0 20px;
  border-top: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  flex-shrink: 0;
  overflow: hidden;
}

.davinci-span {
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--dv-ochre);
  background: var(--dv-ochre-bg);
  border: 1px solid var(--dv-ochre-border);
  border-radius: 4px;
  white-space: nowrap;
}

/* Why the focused op exists: its provenance rules (lowering, then pass facts). */
.davinci-why {
  flex-shrink: 0;
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--hl-keyword);
  background: var(--bg-tertiary);
  border-radius: 4px;
  white-space: nowrap;
}

.davinci-why.fact {
  flex-shrink: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.davinci-why.remark.applied {
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.davinci-why.remark.missed {
  color: var(--color-notice);
  background: var(--color-notice-bg);
}

.davinci-snippet {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 960px) {
  .davinci-playground {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(240px, 40%) 1fr;
  }

  .davinci-source {
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
  }

  .davinci-timeline-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
