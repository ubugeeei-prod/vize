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
import { DAVINCI_PRESET } from "../../shared/presets/davinci";
import type { EditorHighlight } from "../../shared/MonacoEditor.vue";
import {
  compileCodeOutputs,
  createEmptyCodeOutputs,
  type CodeOutputs,
} from "../atelier/codeOutputs";
import { buildLadder, type RungId, type StageLadder } from "./ladder";
import { folioLines, linesCovering } from "./folioLines";
import { sfcOffsetToTemplateBytes, templateBytesToSfcRange, templateStartInSfc } from "./offsets";

export type StageId = RungId | "s4";
export type OutputTarget = "dom" | "vapor" | "ssr";

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

  const stage = ref<StageId>("s2");
  const pageKeys = ref<Partial<Record<RungId, string>>>({});
  const outputTarget = ref<OutputTarget>("dom");
  const selectedLine = ref<number | null>(null);
  const hoveredLine = ref<number | null>(null);
  const cursorBytes = ref<number | null>(null);

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
  const linkedLines = computed(() =>
    cursorBytes.value === null ? [] : linesCovering(lines.value, cursorBytes.value).slice(0, 1),
  );

  const focusLine = computed(() => hoveredLine.value ?? selectedLine.value);
  const focusSpan = computed(() => {
    const index = focusLine.value;
    return index === null ? null : (lines.value[index]?.span ?? null);
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
    selectedLine.value = null;
    hoveredLine.value = null;
  }

  function selectPage(rungId: RungId, key: string) {
    pageKeys.value = { ...pageKeys.value, [rungId]: key };
    stage.value = rungId;
    selectedLine.value = null;
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
      ladder.value = buildLadder(negotiated.feed, FILENAME);
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
    stage,
    rung,
    page,
    lines,
    linkedLines,
    outputTarget,
    selectedLine,
    hoveredLine,
    focusLine,
    focusSource,
    highlights,
    selectStage,
    selectPage,
    onCursor,
  };
}
