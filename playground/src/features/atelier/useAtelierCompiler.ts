import { ref, computed, watch, onScopeDispose } from "vue";
import { PRESETS, type PresetKey, type InputMode } from "../../presets";
import type {
  CompilerOptions,
  CompileResult,
  SfcCompileResult,
  CssCompileResult,
  CssCompileOptions,
  WasmModule,
} from "../../wasm/index";
import { formatCss } from "./formatters";
import {
  compileCodeOutputs,
  createEmptyCodeOutputs,
  getCodeOutputTargets,
  type CodeOutputTarget,
} from "./codeOutputs";
import { mapToObject, filterAstProperties } from "./astHelpers";
import { useClipboard } from "../../utils/useClipboard";
import { useExperimentalFeatures } from "../../shared/experimentalFeatures";

type TabType = "code" | "ast" | "bindings" | "tokens" | "helpers" | "sfc" | "css";

export function useAtelierCompiler(getCompiler: () => WasmModule | null) {
  const { copyToClipboard } = useClipboard();

  const inputMode = ref<InputMode>("sfc");
  const source = ref(PRESETS.propsDestructure.code);
  const { options: experimentals, loadExample } = useExperimentalFeatures("compiler", source);
  const error = ref<string | null>(null);
  const options = ref<CompilerOptions>({
    mode: "module",
    ssr: false,
    scriptExt: "preserve",
  });
  const activeTab = ref<TabType>("code");
  const isCompiling = ref(false);
  const selectedPreset = ref<PresetKey>("propsDestructure");
  const cssOptions = ref<CssCompileOptions>({
    scoped: false,
    scopeId: "data-v-12345678",
    minify: false,
  });
  const codeViewMode = ref<"ts" | "js">("ts");
  const codeOutputTarget = ref<CodeOutputTarget>("dom");
  const result = ref<{
    output: CompileResult | null;
    sfc: SfcCompileResult | null;
    css: CssCompileResult | null;
    formattedCss: string;
    codeOutputs: ReturnType<typeof createEmptyCodeOutputs>;
    compileTime: number;
  } | null>(null);
  const output = computed(() => result.value?.output ?? null);
  const sfcResult = computed(() => result.value?.sfc ?? null);
  const cssResult = computed(() => result.value?.css ?? null);
  const formattedCss = computed(() => result.value?.formattedCss ?? "");
  const codeOutputs = computed(() => result.value?.codeOutputs ?? createEmptyCodeOutputs());
  const compileTime = computed(() => result.value?.compileTime ?? null);
  const codeOutputVersion = ref(0);
  const activeCodeOutput = computed(() => codeOutputs.value[codeOutputTarget.value]);
  const availableCodeOutputTargets = computed(() => getCodeOutputTargets(inputMode.value));
  const astHideLoc = ref(true);
  const astHideSource = ref(true);
  const astCollapsed = ref(false);

  const editorLanguage = computed(() => (inputMode.value === "sfc" ? "vue" : "html"));

  const astJson = computed(() => {
    if (!output.value) return "{}";
    const ast = mapToObject(output.value.ast);
    const filtered = filterAstProperties(ast, astHideLoc.value, astHideSource.value);
    return JSON.stringify(filtered, null, astCollapsed.value ? 0 : 2);
  });

  const bindingsSummary = computed(() => {
    const bindings = sfcResult.value?.script?.bindings?.bindings;
    if (!bindings) return {};
    const summary: Record<string, number> = {};
    for (const type of Object.values(bindings)) {
      summary[type as string] = (summary[type as string] || 0) + 1;
    }
    return summary;
  });

  const groupedBindings = computed(() => {
    const bindings = sfcResult.value?.script?.bindings?.bindings;
    if (!bindings) return {};
    const groups: Record<string, string[]> = {};
    for (const [name, type] of Object.entries(bindings)) {
      if (!groups[type as string]) groups[type as string] = [];
      groups[type as string].push(name);
    }
    return groups;
  });

  async function compileCssFromSfcResult(compiler: WasmModule, sfc: SfcCompileResult | null) {
    if (!sfc?.descriptor?.styles?.length) return { css: null, formattedCss: "" };

    const allCss = sfc.descriptor.styles.map((style) => style.content).join("\n");
    const hasScoped = sfc.descriptor.styles.some((style) => style.scoped);
    const css = compiler.compileCss(allCss, {
      ...cssOptions.value,
      scoped: hasScoped || cssOptions.value.scoped,
    });
    return { css, formattedCss: await formatCss(css.code) };
  }

  let compileVersion = 0;
  async function compile() {
    const version = ++compileVersion;
    result.value = null;
    codeOutputVersion.value += 1;
    error.value = null;
    isCompiling.value = false;
    const compiler = getCompiler();
    if (!compiler) return;

    isCompiling.value = true;
    const currentSource = source.value;
    const currentMode = inputMode.value;
    const compilerOptions = { ...options.value, ...experimentals.value };

    try {
      const startTime = performance.now();
      let compiledOutput: CompileResult | null;
      let sfc: SfcCompileResult | null = null;
      if (currentMode === "sfc") {
        sfc = compiler.compileSfc(currentSource, compilerOptions);
        if (sfc.errors?.length) throw new Error(sfc.errors.join("\n"));
        compiledOutput = sfc.script?.code
          ? {
              code: sfc.script.code,
              preamble: sfc.template?.preamble || "",
              ast: sfc.template?.ast || {},
              helpers: sfc.template?.helpers || [],
            }
          : (sfc.template ?? null);
      } else {
        compiledOutput = compiler.compile(currentSource, compilerOptions);
      }
      const css = await compileCssFromSfcResult(compiler, sfc);
      if (version !== compileVersion) return;
      const outputs = await compileCodeOutputs({
        compiler,
        inputMode: currentMode,
        source: currentSource,
        options: compilerOptions,
        baseOutput: compiledOutput,
        baseSfcResult: sfc,
      });
      if (version !== compileVersion) return;
      result.value = {
        output: compiledOutput,
        sfc,
        ...css,
        codeOutputs: outputs,
        compileTime: performance.now() - startTime,
      };
      codeOutputVersion.value += 1;
    } catch (e) {
      if (version !== compileVersion) return;
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      if (version === compileVersion) isCompiling.value = false;
    }
  }

  function handlePresetChange(key: string) {
    const preset = PRESETS[key];
    selectedPreset.value = key;
    inputMode.value = preset.mode;
    source.value = preset.code;
    if (preset.mode === "sfc") {
      activeTab.value = "code";
    }
  }

  function copyFullOutput() {
    const currentOutput = activeCodeOutput.value;
    if (!currentOutput.code && !currentOutput.error) return;
    const fullOutput = `
=== COMPILER OUTPUT ===
Compile Time: ${compileTime?.value?.toFixed(4) ?? "N/A"}ms

=== TARGET ===
${codeOutputTarget.value}

=== CODE ===
${currentOutput.error || currentOutput.code}

=== HELPERS ===
${output.value?.helpers?.join("\n") || "None"}`.trim();
    copyToClipboard(fullOutput);
  }

  let compileTimer: ReturnType<typeof setTimeout> | null = null;
  onScopeDispose(() => {
    compileVersion += 1;
    if (compileTimer) clearTimeout(compileTimer);
  });

  watch(
    [source, options, inputMode, experimentals, cssOptions],
    () => {
      compileVersion += 1;
      isCompiling.value = false;
      result.value = null;
      error.value = null;
      codeOutputVersion.value += 1;
      if (compileTimer) clearTimeout(compileTimer);
      if (!getCompiler()) return;
      compileTimer = setTimeout(compile, 300);
    },
    { deep: true, flush: "sync" },
  );

  watch(availableCodeOutputTargets, (targets) => {
    if (!targets.includes(codeOutputTarget.value)) {
      codeOutputTarget.value = "dom";
    }
  });

  return {
    experimentals,
    loadExample,
    inputMode,
    source,
    output,
    sfcResult,
    error,
    options,
    activeTab,
    isCompiling,
    selectedPreset,
    compileTime,
    cssResult,
    cssOptions,
    formattedCss,
    codeViewMode,
    codeOutputTarget,
    availableCodeOutputTargets,
    codeOutputs,
    codeOutputVersion,
    activeCodeOutput,
    astHideLoc,
    astHideSource,
    astCollapsed,
    editorLanguage,
    astJson,
    bindingsSummary,
    groupedBindings,
    compile,
    handlePresetChange,
    copyFullOutput,
  };
}
