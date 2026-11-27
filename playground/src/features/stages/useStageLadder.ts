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
import type { SfcCompileResult, WasmModule } from "../../wasm/index";
import { negotiateProductCapture } from "../../wasm/types/productCapture";
import type { InspectorDiff } from "../../wasm/types/inspector";
import { DAVINCI_PRESET } from "../../shared/presets/davinci";
import type { EditorHighlight } from "../../shared/monaco-editor.vue";
import {
  compileCodeOutputs,
  createEmptyCodeOutputs,
  type CodeOutputs,
} from "../atelier/codeOutputs";
import type { RungId, LadderPage } from "./ladder";
import { useProductCaptureState, type ProductCapturedResult } from "./productCaptureState";
import { dumpLines, type DumpLine } from "./dumpLines";
import { cursorSpan, traceMatches, tracePages } from "./sourceTrace";
import { sfcOffsetToTemplateBytes, templateBytesToSfcRange, type Range } from "./offsets";
import { remarksAt, type StageRemark } from "./remarks";
import { parseProvenance } from "./provenance";
import { graphLineKinds, partitionKinds } from "./partition";

export type StageId = RungId | "l4";
export type OutputTarget = "dom" | "vapor" | "ssr";
/** What the stage body shows: the page, its diff, or observed remarks. */
export type PageView = "page" | "diff" | "remarks";

const FILENAME = "Component.vue";

/**
 * Each target's stage pages and assembled module come from its own exact
 * `compileSfc` invocation. The stage tab never invokes `analyzeSfc`.
 */
export function useStageLadder(getCompiler: () => WasmModule | null) {
  const injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
  const theme = computed<"dark" | "light">(() => injectedTheme?.value ?? "light");

  const source = ref(DAVINCI_PRESET);
  const captures = shallowRef<Partial<Record<OutputTarget, ProductCapturedResult>>>({});
  const compiledSource = ref<string | null>(null);
  const error = ref<string | null>(null);
  const outputs = shallowRef<CodeOutputs>(createEmptyCodeOutputs());

  const stage = ref<StageId>("l2");
  const pageKeys = ref<Partial<Record<RungId, string>>>({});
  const outputTarget = ref<OutputTarget>("dom");
  const {
    captureError,
    captureStatus,
    captureLabel,
    remarksNote,
    profileNote,
    ladder,
    templateStart,
    syntaxNote,
  } = useProductCaptureState(captures, compiledSource, source, outputs, outputTarget);
  const selectedLine = ref<number | null>(null);
  const hoveredLine = ref<number | null>(null);
  const cursorBytes = ref<number | null>(null);
  const pageView = ref<PageView>("page");
  const pinnedSpan = ref<Range | null>(null);
  const previewSpan = ref<Range | null>(null);
  const remarks = computed<StageRemark[]>(() => ladder.value?.remarks ?? []);

  const rung = computed(() =>
    stage.value === "l4" ? null : (ladder.value?.rungs.find((r) => r.id === stage.value) ?? null),
  );
  const page = computed(() => {
    const current = rung.value;
    if (!current || current.pages.length === 0) return null;
    const key = pageKeys.value[current.id];
    return current.pages.find((p) => p.key === key) ?? current.pages[0];
  });
  const lines = computed(() => (page.value ? dumpLines(page.value.kind, page.value.text) : []));
  /** On the L3 graph page, each op line's exported static/dynamic partition. */
  const lineMarks = computed(() => {
    const shown = page.value;
    const partition = rung.value?.pages.find((p) => p.kind === "partition");
    if (!shown || shown.kind !== "impeto" || !partition) return new Map<number, string>();
    return graphLineKinds(shown.text, partitionKinds(partition.text));
  });
  /** The page this one is compared against: the previous tree page of its stage. */
  const previousPage = computed(() => {
    const current = rung.value;
    const shown = page.value;
    if (!current || !shown || shown.kind !== "disegno") return null;
    const trees = current.pages.filter((p) => p.kind === "disegno");
    const index = trees.indexOf(shown);
    return index > 0 ? trees[index - 1] : null;
  });
  const diff = computed<InspectorDiff | null>(() => {
    const before = previousPage.value;
    const after = page.value;
    if (pageView.value !== "diff" || !before || !after) return null;
    return getCompiler()?.buildInspectorDiff(before.text, after.text) ?? null;
  });
  const overviewPages = computed(() =>
    (ladder.value?.rungs ?? []).map((rung) => {
      const page = rung.pages.find((page) => page.key === pageKeys.value[rung.id]) ?? rung.pages[0];
      const lines = dumpLines(page.kind, page.text);
      const partition = rung.pages.find((page) => page.kind === "partition");
      const marks =
        page.kind === "impeto" && partition
          ? graphLineKinds(page.text, partitionKinds(partition.text))
          : new Map<number, string>();
      return { rung, page, lines, marks };
    }),
  );
  const structuralPages = computed(() => tracePages(ladder.value?.rungs ?? [], page.value));
  const surfaceUnits = computed(() => {
    const semantic = ladder.value?.rungs.find(({ id }) => id === "l2")?.pages;
    const lower =
      semantic?.find((page) => page.kind === "disegno" && page.pass === "lower") ??
      semantic?.find((page) => page.kind === "disegno");
    // Lowering metadata can repeat its owner's span; it is one authored unit.
    const units = new Map<string, DumpLine>();
    for (const line of lower ? dumpLines("disegno", lower.text) : []) {
      if (!line.span) continue;
      const key = `${line.span.start}:${line.span.end}`;
      if (!units.has(key)) units.set(key, line);
    }
    return [...units.values()];
  });
  // Only an explicit selection reveals rows. Hover must never move a scroll viewport.
  const revealSpan = computed(
    () =>
      (selectedLine.value === null ? null : lines.value[selectedLine.value]?.span) ??
      pinnedSpan.value ??
      cursorSpan(structuralPages.value, cursorBytes.value),
  );
  const focusSpan = computed(() => {
    if (previewSpan.value) return previewSpan.value;
    const index = hoveredLine.value ?? selectedLine.value;
    return (
      (index === null ? null : lines.value[index]?.span) ??
      pinnedSpan.value ??
      cursorSpan(structuralPages.value, cursorBytes.value)
    );
  });
  const linkedLines = computed(() => traceMatches(lines.value, focusSpan.value, page.value?.kind));
  const focusLine = computed(
    () => hoveredLine.value ?? selectedLine.value ?? linkedLines.value[0] ?? null,
  );
  const provenance = computed(() => {
    const s2 = ladder.value?.rungs.find((r) => r.id === "l2");
    const page = s2?.pages.find((p) => p.kind === "provenance");
    return page ? parseProvenance(page.text) : [];
  });
  const focusProvenance = computed(() => {
    const span = focusSpan.value;
    return span
      ? provenance.value.filter(
          (record) => record.span.start === span.start && record.span.end === span.end,
        )
      : [];
  });
  const focusRemarks = computed(() =>
    focusSpan.value ? remarksAt(remarks.value, focusSpan.value) : [],
  );
  const highlights = computed<EditorHighlight[]>(() => {
    const span = focusSpan.value;
    if (!span || !ladder.value || templateStart.value === null) return [];
    const range = templateBytesToSfcRange(ladder.value.template, templateStart.value, span);
    return [
      {
        ...range,
        className: "davinci-provenance",
        reveal: hoveredLine.value === null && previewSpan.value === null,
      },
    ];
  });
  const focusSource = computed(() => {
    const span = focusSpan.value;
    if (!span || !ladder.value || templateStart.value === null) return null;
    const range = templateBytesToSfcRange(ladder.value.template, templateStart.value, span);
    const line = source.value.slice(0, range.start).split("\n").length;
    return { span, line, text: source.value.slice(range.start, range.end) };
  });

  function preserveFocus() {
    pinnedSpan.value = focusSpan.value;
    previewSpan.value = null;
    hoveredLine.value = null;
    selectedLine.value = null;
  }

  function selectStage(next: StageId) {
    preserveFocus();
    stage.value = next;
    pageView.value = "page";
    selectedLine.value = null;
    hoveredLine.value = null;
  }

  function selectPage(rungId: RungId, key: string) {
    preserveFocus();
    pageKeys.value = { ...pageKeys.value, [rungId]: key };
    stage.value = rungId;
    if (pageView.value === "remarks") pageView.value = "page";
    selectedLine.value = null;
  }

  function clearFocus() {
    selectedLine.value = null;
    hoveredLine.value = null;
    previewSpan.value = null;
    pinnedSpan.value = null;
    cursorBytes.value = null;
  }

  function selectTraceLine(rungId: RungId, page: LadderPage, line: DumpLine | null) {
    const span = line?.span ?? null;
    const alreadySelected =
      pinnedSpan.value?.start === span?.start &&
      pinnedSpan.value?.end === span?.end &&
      stage.value === rungId &&
      pageKeys.value[rungId] === page.key;
    selectPage(rungId, page.key);
    pageView.value = "page";
    if (alreadySelected || !span) clearFocus();
    else pinnedSpan.value = span;
  }

  function hoverTraceLine(line: DumpLine | null) {
    previewSpan.value = line?.span ?? null;
  }

  function locateRemark(remark: StageRemark) {
    clearFocus();
    pinnedSpan.value = remark.span;
  }

  function onCursor(offset: number) {
    clearFocus();
    const template = templateStart.value === null ? undefined : ladder.value?.template;
    cursorBytes.value =
      template === undefined
        ? null
        : sfcOffsetToTemplateBytes(template, templateStart.value!, offset);
  }

  let version = 0;
  async function run() {
    const compiler = getCompiler();
    if (!compiler) return;
    const current = ++version;
    const input = source.value;
    try {
      const options = {
        mode: "module" as const,
        scriptExt: "preserve" as const,
        filename: FILENAME,
        captureStages: true,
      };
      let sfc: SfcCompileResult | null = null;
      let domError: string | null = null;
      try {
        sfc = compiler.compileSfc(input, options);
      } catch (caught) {
        domError = caught instanceof Error ? caught.message : String(caught);
      }
      const results: Partial<Record<OutputTarget, SfcCompileResult>> = {};
      const compiled = await compileCodeOutputs({
        compiler,
        inputMode: "sfc",
        source: input,
        options,
        baseOutput: null,
        baseSfcResult: sfc,
        onSfcResult: (target, result) => {
          results[target] = result;
        },
        assembledModule: true,
      });
      if (domError) compiled.dom = { ...compiled.dom, error: domError };
      if (current !== version) return;
      const next: Partial<Record<OutputTarget, ProductCapturedResult>> = {};
      for (const target of ["dom", "ssr", "vapor"] as const) {
        const result = results[target];
        if (result) {
          next[target] = {
            result,
            negotiation: negotiateProductCapture(result.stageCapture, target),
          };
        }
      }
      captures.value = next;
      compiledSource.value = input;
      outputs.value = compiled;
      error.value = null;
      const visible = ladder.value?.rungs ?? [];
      if (pinnedSpan.value === null && cursorBytes.value === null) {
        const semantic = overviewPages.value.find(({ rung }) => rung.id === "l2");
        pinnedSpan.value =
          semantic?.lines.find((line) => /ui\.(interpolation|model)\b/.test(line.text))?.span ??
          null;
      }
      if (stage.value !== "l4" && !visible.some(({ id }) => id === stage.value)) {
        stage.value = visible[0]?.id ?? "l4";
      }
    } catch (caught) {
      if (current !== version) return;
      error.value = caught instanceof Error ? caught.message : String(caught);
      captures.value = {};
      compiledSource.value = null;
      outputs.value = createEmptyCodeOutputs();
    }
  }

  let timer: ReturnType<typeof setTimeout> | null = null;
  watch(source, () => {
    clearFocus();
    version++;
    captures.value = {};
    compiledSource.value = null;
    outputs.value = createEmptyCodeOutputs();
    error.value = null;
    pageView.value = "page";
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => void run(), 250);
  });
  watch(outputTarget, () => {
    clearFocus();
    pageView.value = "page";
    pageKeys.value = {};
    selectedLine.value = null;
    const visible = ladder.value?.rungs ?? [];
    if (stage.value !== "l4" && !visible.some(({ id }) => id === stage.value)) {
      stage.value = visible[0]?.id ?? "l4";
    }
  });
  watch(lines, () => {
    selectedLine.value = null;
    hoveredLine.value = null;
  });
  let readyCompiler: WasmModule | null = null;
  function runWithNewCompiler(compiler: WasmModule | null) {
    if (!compiler || compiler === readyCompiler) return;
    readyCompiler = compiler;
    void run();
  }
  watch(getCompiler, runWithNewCompiler);

  let poll: ReturnType<typeof setInterval> | null = null;
  onMounted(() => {
    if (getCompiler()) {
      runWithNewCompiler(getCompiler());
      return;
    }
    poll = setInterval(() => {
      if (!getCompiler()) return;
      if (poll) clearInterval(poll);
      poll = null;
      runWithNewCompiler(getCompiler());
    }, 200);
  });
  onUnmounted(() => {
    version++;
    if (timer) clearTimeout(timer);
    if (poll) clearInterval(poll);
  });

  return {
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
    focusLine,
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
  };
}
