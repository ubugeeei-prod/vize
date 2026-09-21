import {
  computed,
  inject,
  onMounted,
  onUnmounted,
  ref,
  shallowRef,
  watch,
  type ComputedRef,
} from "vue";
import type { WasmModule } from "../../wasm/index";
import { negotiateSpolveroFeed } from "../../wasm/types/spolvero";
import { ladderStepTimings, negotiateProfileExport } from "../../wasm/types/profile";
import type { InspectorDiff } from "../../wasm/types/inspector";
import { DAVINCI_PRESET } from "../../shared/presets/davinci";
import type { EditorHighlight } from "../../shared/MonacoEditor.vue";
import {
  compileCodeOutputs,
  createEmptyCodeOutputs,
  type CodeOutputs,
} from "../atelier/codeOutputs";
import { buildLadder, type RungId, type StageLadder } from "./ladder";
import { folioLines, linesCovering } from "./folioLines";
import {
  sfcOffsetToTemplateBytes,
  templateBytesToSfcRange,
  templateStartInSfc,
  type Range,
} from "./offsets";
import type { SpolveroRemark } from "./remarks";
import { parseProvenance, recordsForNode } from "./provenance";

export type StageId = RungId | "s4";
export type OutputTarget = "dom" | "vapor" | "ssr";
/** What the stage body shows: the page, its diff to the previous page, or remarks. */
export type PageView = "page" | "diff" | "remarks";

const FILENAME = "Component.vue";

/**
 * The Davinci tab's state: one compile of the source through the real wasm
 * compiler (the Spolvero feed from `analyzeSfc`, the emitted code from
 * `compileSfc`), plus the selection that links stage lines to source spans.
 */
export function useDavinciLadder(getCompiler: () => WasmModule | null) {
  const injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
  const theme = computed<"dark" | "light">(() => injectedTheme?.value ?? "light");

  const source = ref(DAVINCI_PRESET);
  const ladder = shallowRef<StageLadder | null>(null);
  const error = ref<string | null>(null);
  const outputs = shallowRef<CodeOutputs>(createEmptyCodeOutputs());
  const templateStart = ref(0);
  const ladderTime = ref<number | null>(null);
  /** Why step timings are missing, when the profile did not negotiate. */
  const profileNote = ref<string | null>(null);

  const stage = ref<StageId>("s2");
  const pageKeys = ref<Partial<Record<RungId, string>>>({});
  const outputTarget = ref<OutputTarget>("dom");
  const selectedLine = ref<number | null>(null);
  const hoveredLine = ref<number | null>(null);
  const cursorBytes = ref<number | null>(null);
  const pageView = ref<PageView>("page");
  const pinnedSpan = ref<Range | null>(null);
  // The feed carries no remark pages yet (the pass manager's remark channel
  // exists; nothing emits into the feed), so the panel shows its empty state.
  const remarks = computed<SpolveroRemark[]>(() => []);

  const rung = computed(() =>
    stage.value === "s4" ? null : (ladder.value?.rungs.find((r) => r.id === stage.value) ?? null),
  );
  const page = computed(() => {
    const current = rung.value;
    if (!current || current.pages.length === 0) return null;
    const key = pageKeys.value[current.id];
    return current.pages.find((p) => p.key === key) ?? current.pages[0];
  });
  const lines = computed(() => (page.value ? folioLines(page.value.kind, page.value.text) : []));
  /** The page this one is compared against: the previous page of its stage. */
  const previousPage = computed(() => {
    const current = rung.value;
    const shown = page.value;
    if (!current || !shown || shown.kind !== "disegno") return null;
    const index = current.pages.indexOf(shown);
    return index > 0 ? current.pages[index - 1] : null;
  });
  const diff = computed<InspectorDiff | null>(() => {
    const before = previousPage.value;
    const after = page.value;
    if (pageView.value !== "diff" || !before || !after) return null;
    return getCompiler()?.buildInspectorDiff(before.text, after.text) ?? null;
  });
  const linkedLines = computed(() =>
    cursorBytes.value === null ? [] : linesCovering(lines.value, cursorBytes.value).slice(0, 1),
  );

  const focusLine = computed(() => hoveredLine.value ?? selectedLine.value);
  const provenance = computed(() => {
    const s2 = ladder.value?.rungs.find((r) => r.id === "s2");
    const page = s2?.pages.find((p) => p.kind === "provenance");
    return page ? parseProvenance(page.text) : [];
  });
  /** Why the focused S2 op exists: its lowering record, then pass facts. */
  const focusProvenance = computed(() => {
    const index = focusLine.value;
    const node = index === null ? null : (lines.value[index]?.node ?? null);
    return node === null ? [] : recordsForNode(provenance.value, node);
  });
  const focusSpan = computed(() => {
    const index = focusLine.value;
    return index === null ? pinnedSpan.value : (lines.value[index]?.span ?? null);
  });
  const highlights = computed<EditorHighlight[]>(() => {
    const span = focusSpan.value;
    if (!span || !ladder.value) return [];
    const range = templateBytesToSfcRange(ladder.value.template, templateStart.value, span);
    return [{ ...range, className: "davinci-provenance", reveal: hoveredLine.value === null }];
  });
  const focusSource = computed(() => {
    const span = focusSpan.value;
    if (!span || !ladder.value) return null;
    const range = templateBytesToSfcRange(ladder.value.template, templateStart.value, span);
    return { span, text: source.value.slice(range.start, range.end) };
  });

  function selectStage(next: StageId) {
    stage.value = next;
    pageView.value = "page";
    selectedLine.value = null;
    hoveredLine.value = null;
  }

  function selectPage(rungId: RungId, key: string) {
    pageKeys.value = { ...pageKeys.value, [rungId]: key };
    stage.value = rungId;
    if (pageView.value === "remarks") pageView.value = "page";
    selectedLine.value = null;
  }

  function locateRemark(remark: SpolveroRemark) {
    pinnedSpan.value = remark.span;
  }

  function onCursor(offset: number) {
    const template = ladder.value?.template;
    cursorBytes.value =
      template === undefined
        ? null
        : sfcOffsetToTemplateBytes(template, templateStart.value, offset);
  }

  let version = 0;
  async function run() {
    const compiler = getCompiler();
    if (!compiler) return;
    const current = ++version;
    try {
      const started = performance.now();
      const analysis = compiler.analyzeSfc(source.value, { filename: FILENAME });
      ladderTime.value = performance.now() - started;
      const negotiated = negotiateSpolveroFeed(analysis.spolvero);
      if (!negotiated.ok) {
        error.value = negotiated.error;
        ladder.value = null;
        return;
      }
      const options = {
        mode: "module" as const,
        scriptExt: "preserve" as const,
        filename: FILENAME,
      };
      const sfc = compiler.compileSfc(source.value, options);
      const start = sfc.descriptor.template?.loc.start;
      templateStart.value = start === undefined ? 0 : templateStartInSfc(source.value, start);
      const profile = negotiateProfileExport(analysis.spolveroProfile);
      profileNote.value = profile.ok ? null : profile.error;
      const timings = profile.ok ? ladderStepTimings(profile.profile) : new Map<string, number>();
      ladder.value = buildLadder(negotiated.feed, FILENAME, timings);
      error.value = null;
      const compiled = await compileCodeOutputs({
        compiler,
        inputMode: "sfc",
        source: source.value,
        options,
        baseOutput: null,
        baseSfcResult: sfc,
      });
      if (current === version) outputs.value = compiled;
    } catch (caught) {
      if (current !== version) return;
      error.value = caught instanceof Error ? caught.message : String(caught);
      ladder.value = null;
    }
  }

  let timer: ReturnType<typeof setTimeout> | null = null;
  watch(source, () => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => void run(), 250);
  });
  watch(lines, () => {
    selectedLine.value = null;
    hoveredLine.value = null;
    pinnedSpan.value = null;
  });
  watch(getCompiler, (compiler) => {
    if (compiler) void run();
  });

  let poll: ReturnType<typeof setInterval> | null = null;
  onMounted(() => {
    if (getCompiler()) {
      void run();
      return;
    }
    poll = setInterval(() => {
      if (!getCompiler()) return;
      if (poll) clearInterval(poll);
      poll = null;
      void run();
    }, 200);
  });
  onUnmounted(() => {
    if (timer) clearTimeout(timer);
    if (poll) clearInterval(poll);
  });

  return {
    theme,
    source,
    ladder,
    error,
    outputs,
    ladderTime,
    profileNote,
    stage,
    rung,
    page,
    lines,
    previousPage,
    diff,
    pageView,
    remarks,
    linkedLines,
    outputTarget,
    selectedLine,
    hoveredLine,
    focusLine,
    focusSource,
    focusProvenance,
    highlights,
    selectStage,
    selectPage,
    locateRemark,
    onCursor,
  };
}
