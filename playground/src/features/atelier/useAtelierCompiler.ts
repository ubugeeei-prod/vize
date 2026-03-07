import { ref, computed, watch } from "vue";
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
import { compileCodeOutputs, createEmptyCodeOutputs, type CodeOutputTarget } from "./codeOutputs";
import { mapToObject, filterAstProperties } from "./astHelpers";
import { useClipboard } from "../../utils/useClipboard";

type TabType = "code" | "ast" | "bindings" | "tokens" | "helpers" | "sfc" | "css";

export function useAtelierCompiler(getCompiler: () => WasmModule | null) {
  const { copyToClipboard } = useClipboard();

  const inputMode = ref<InputMode>("sfc");
  const source = ref(PRESETS.propsDestructure.code);
  const output = ref<CompileResult | null>(null);
  const sfcResult = ref<SfcCompileResult | null>(null);
  const error = ref<string | null>(null);
  const options = ref<CompilerOptions>({
    mode: "module",
    ssr: false,
    scriptExt: "preserve",
  });
  const activeTab = ref<TabType>("code");
  const isCompiling = ref(false);
  const selectedPreset = ref<PresetKey>("propsDestructure");
  const compileTime = ref<number | null>(null);
  const cssResult = ref<CssCompileResult | null>(null);
  const cssOptions = ref<CssCompileOptions>({
    scoped: false,
    scopeId: "data-v-12345678",
    minify: false,
  });
  const formattedCss = ref<string>("");
  const codeViewMode = ref<"ts" | "js">("ts");
  const codeOutputTarget = ref<CodeOutputTarget>("dom");
  const codeOutputs = ref(createEmptyCodeOutputs());
  const codeOutputVersion = ref(0);
  const activeCodeOutput = computed(() => codeOutputs.value[codeOutputTarget.value]);
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

  async function compileCssFromSfcResult(compiler: WasmModule, result: SfcCompileResult | null) {
    if (!result?.descriptor?.styles?.length) {
      cssResult.value = null;
      formattedCss.value = "";
      return;
    }

    const allCss = result.descriptor.styles
      .map((style: { content: string }) => style.content)
      .join("\n");
    const hasScoped = result.descriptor.styles.some((style: { scoped?: boolean }) => style.scoped);
    const css = compiler.compileCss(allCss, {
      ...cssOptions.value,
      scoped: hasScoped || cssOptions.value.scoped,
    });
    cssResult.value = css;
    formattedCss.value = await formatCss(css.code);
  }

  async function compile() {
    const compiler = getCompiler();
    if (!compiler) return;

    isCompiling.value = true;
    error.value = null;

    try {
      const startTime = performance.now();

      if (inputMode.value === "sfc") {
        try {
          const result = compiler.compileSfc(source.value, options.value);
          sfcResult.value = result;
          await compileCssFromSfcResult(compiler, result);

          if (result?.script?.code) {
            output.value = {
              code: result.script.code,
              preamble: result.template?.preamble || "",
              ast: result.template?.ast || {},
              helpers: result.template?.helpers || [],
            };
          } else if (result?.template) {
            output.value = result.template;
          } else {
            output.value = null;
          }

          codeOutputs.value = await compileCodeOutputs({
            compiler,
            inputMode: inputMode.value,
            source: source.value,
            options: options.value,
            baseOutput: output.value,
            baseSfcResult: sfcResult.value,
          });
          codeOutputVersion.value += 1;
          compileTime.value = performance.now() - startTime;
        } catch (sfcError) {
          console.error("SFC compile error:", sfcError);
          throw sfcError;
        }
      } else {
        const result = compiler.compile(source.value, options.value);
        output.value = result;
        sfcResult.value = null;
        cssResult.value = null;
        formattedCss.value = "";
        codeOutputs.value = await compileCodeOutputs({
          compiler,
          inputMode: inputMode.value,
          source: source.value,
          options: options.value,
          baseOutput: result,
          baseSfcResult: null,
        });
        codeOutputVersion.value += 1;
        compileTime.value = performance.now() - startTime;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      codeOutputs.value = createEmptyCodeOutputs();
    } finally {
      isCompiling.value = false;
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

  watch(
    [source, options, inputMode],
    () => {
      if (!getCompiler()) return;
      if (compileTimer) clearTimeout(compileTimer);
      compileTimer = setTimeout(compile, 300);
    },
    { deep: true },
  );

  watch(
    cssOptions,
    () => {
      const compiler = getCompiler();
      if (compiler && sfcResult.value?.descriptor?.styles?.length) {
        void compileCssFromSfcResult(compiler, sfcResult.value);
      }
    },
    { deep: true },
  );

  return {
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
