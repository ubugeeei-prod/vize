<script setup lang="ts">
import "./stage-playground-styles.vue";
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import MonacoEditor from "../../shared/monaco-editor.vue";
import { LEVEL_EXAMPLES } from "../../shared/presets/levels";
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

const example = ref(LEVEL_EXAMPLES[0].key);
const exampleId = useId();

function loadExample(key: string) {
  const found = LEVEL_EXAMPLES.find((item) => item.key === key);
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
  <div class="level-playground">
    <section class="level-source" aria-label="Source">
      <div class="level-bar">
        <h2 class="level-title">Source</h2>
        <span class="level-hint">Put the cursor on markup to find it in the stage page</span>
        <div class="level-example">
          <label class="level-hint" :for="exampleId">Example</label>
          <select :id="exampleId" v-model="example" class="level-select">
            <option v-for="item in LEVEL_EXAMPLES" :key="item.key" :value="item.key">
              {{ item.label }}
            </option>
          </select>
        </div>
        <button type="button" class="level-ghost" @click="() => loadExample(example)">Reset</button>
      </div>
      <div class="level-editor">
        <MonacoEditor v-model="source" language="vue" :highlights :theme @cursor="onCursor" />
      </div>
    </section>

    <section class="level-stages" aria-label="Levels stage ladder">
      <div class="level-bar">
        <h2 class="level-title">Levels stage ladder</h2>
        <span v-if="captureLabel" class="level-badge" title="Same-run product capture">{{
          captureLabel
        }}</span>
        <span v-if="captureStatus" class="level-badge" title="Product capture schema_version"
          >feed v2</span
        >
        <span class="level-hint level-keys">Keys 1–4 pick a stage, ← → walk the steps</span>
      </div>

      <div class="level-subtabs" role="tablist" aria-label="Compile target">
        <button
          v-for="target in targets"
          :key="target"
          type="button"
          role="tab"
          :class="['level-subtab', { active: outputTarget === target }]"
          :aria-selected="outputTarget === target"
          @click="() => (outputTarget = target)"
        >
          {{ target.toUpperCase() }}
        </button>
      </div>

      <div v-if="error" class="level-message error" role="alert">{{ error }}</div>
      <div v-else-if="captureError" class="level-body">
        <div class="level-message error" role="alert">{{ captureError }}</div>
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

        <div v-if="showTabs && rung" class="level-subtabs" role="tablist" aria-label="Pages">
          <button
            v-for="tab in rung.pages"
            :key="tab.key"
            type="button"
            role="tab"
            :class="['level-subtab', { active: pageView === 'page' && page?.key === tab.key }]"
            :aria-selected="page?.key === tab.key"
            @click="() => selectPage(rung.id, tab.key)"
          >
            {{ tab.label }}
          </button>
          <span class="level-subtabs-gap" aria-hidden="true"></span>
          <button
            v-if="previousPage"
            type="button"
            :class="['level-subtab', { active: pageView === 'diff' }]"
            :aria-pressed="pageView === 'diff'"
            @click="() => toggleView('diff')"
          >
            Diff vs {{ previousPage.label }}
          </button>
          <span v-if="remarksNote" class="level-hint">Remarks unavailable</span>
          <button
            v-else
            type="button"
            :class="['level-subtab', { active: pageView === 'remarks' }]"
            :aria-pressed="pageView === 'remarks'"
            @click="() => toggleView('remarks')"
          >
            Remarks <span class="level-count">{{ remarks.length }}</span>
          </button>
        </div>

        <div class="level-body">
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
          <div v-else class="level-message">This stage produced no page for the source.</div>
        </div>

        <footer class="level-provenance-bar" aria-live="polite">
          <template v-if="focusSource">
            <span class="level-span">{{ focusSource.span.start }}–{{ focusSource.span.end }}</span>
            <code class="level-snippet">{{ snippet(focusSource.text) }}</code>
            <span
              v-for="record in focusProvenance"
              :key="`${record.span.start}:${record.span.end}:${record.rule}:${record.node}`"
              :class="['level-why', { fact: record.rule.startsWith('pass.') }]"
              :title="`${record.rule}: ${record.before} → ${record.after}`"
              >{{ record.rule
              }}<template v-if="record.rule.startsWith('pass.')">
                {{ snippet(record.after) }}</template
              ></span
            >
            <span
              v-for="(remark, index) in focusRemarks"
              :key="`remark-${index}`"
              :class="['level-why', 'remark', remark.kind]"
              :title="remark.args.map(formatArg).join(' ')"
              >{{ remark.kind }} {{ remark.name }}</span
            >
          </template>
          <span v-else-if="stage !== 'l4'" class="level-hint"
            >Point at a line to see the authored source it came from</span
          >
          <span v-else class="level-hint"
            >Module and stages came from the same compileSfc call</span
          >
          <span v-if="remarksNote" class="level-hint">{{ remarksNote }}</span>
          <span v-if="profileNote" class="level-hint">{{ profileNote }}</span>
          <span v-if="syntaxNote" class="level-hint">{{ syntaxNote }}</span>
          <span v-if="ladder.unplaced.length" class="level-hint"
            >Unplaced pages: {{ ladder.unplaced.join(", ") }}</span
          >
        </footer>
      </template>
      <div v-else-if="captureStatus" class="level-body">
        <div class="level-message">
          {{ captureStatus }}. Native stage pages are unavailable for this target.
        </div>
        <OutputView v-model:target="outputTarget" :outputs :theme />
      </div>
      <div v-else class="level-message">Loading the compiler…</div>
    </section>
  </div>
</template>
