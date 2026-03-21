// WASM module loader for vize
// Re-exports all types and provides the WASM loading API

export type {
  CompilerOptions,
  CompileResult,
  SfcBlock,
  SfcScriptBlock,
  SfcStyleBlock,
  SfcDescriptor,
  SfcCompileResult,
  CssCompileOptions,
  CssCompileResult,
  ArtParseOptions,
  ArtMetadata,
  ArtVariant,
  ArtStyleBlock,
  ArtDescriptor,
  CsfOutput,
  LintOptions,
  LocaleInfo,
  LintDiagnostic,
  LintResult,
  LintRule,
  FormatOptions,
  FormatResult,
  CroquisOptions,
  BindingSource,
  BindingMetadata,
  BindingDisplay,
  ScopeKind,
  ScopeDisplay,
  MacroDisplay,
  TypeExportDisplay,
  InvalidExportDisplay,
  PropDisplay,
  EmitDisplay,
  ProvideKey,
  ProvideDisplay,
  InjectPattern,
  InjectDisplay,
  CssDisplay,
  CroquisStats,
  CroquisDiagnostic,
  Croquis,
  CroquisResult,
  TypeCheckOptions,
  TypeCheckRelatedLocation,
  TypeCheckDiagnostic,
  TypeCheckResult,
  TypeCheckCapability,
  TypeCheckCapabilities,
  CrossFileOptions,
  CrossFileDiagnostic,
  CrossFileStats,
  CrossFileResult,
  CrossFileInput,
  WasmModule,
} from "./types";

import type { LintPreset, LintRule, WasmModule } from "./types";
import { createTransformAnalyzeSfc } from "./wasm-transform";

let wasmModule: WasmModule | null = null;
let loadPromise: Promise<WasmModule> | null = null;

function normalizeLintPreset(value: string): LintPreset {
  switch (value) {
    case "happy-path":
    case "happy_path":
    case "happy":
    case "default":
    case "recommended":
    case "GeneralRecommended":
    case "generalRecommended":
      return "general-recommended";
    case "Essential":
      return "essential";
    case "Incremental":
      return "incremental";
    case "Opinionated":
    case "Opnionated":
    case "opnionated":
    case "strict":
    case "all":
      return "opinionated";
    case "Nuxt":
      return "nuxt";
    default:
      return value as LintPreset;
  }
}

function normalizeLintRules(rules: LintRule[]): LintRule[] {
  return rules.map((rule) => ({
    ...rule,
    presets: rule.presets.map(normalizeLintPreset),
  }));
}

export async function loadWasm(): Promise<WasmModule> {
  if (wasmModule) {
    return wasmModule;
  }

  if (loadPromise) {
    return loadPromise;
  }

  loadPromise = (async () => {
    const wasm = await import("./vize_vitrine.js");

    if (wasm.default) {
      await wasm.default();
    }

    const transformAnalyzeSfc = createTransformAnalyzeSfc(wasm.analyzeSfc);

    const module: WasmModule = {
      compile: wasm.compile,
      compileVapor: wasm.compileVapor,
      compileCss: wasm.compileCss,
      parseTemplate: wasm.parseTemplate,
      parseSfc: wasm.parseSfc,
      compileSfc: wasm.compileSfc,
      analyzeSfc: transformAnalyzeSfc,
      analyzeCrossFile: wasm.analyzeCrossFile,
      parseArt: wasm.parseArt,
      artToCsf: wasm.artToCsf,
      lintTemplate: wasm.lintTemplate,
      lintSfc: wasm.lintSfc,
      getLintRules: () => normalizeLintRules(wasm.getLintRules()),
      getLocales: wasm.getLocales,
      formatSfc: wasm.formatSfc,
      formatTemplate: wasm.formatTemplate,
      formatScript: wasm.formatScript,
      typeCheck: wasm.typeCheck,
      getTypeCheckCapabilities: wasm.getTypeCheckCapabilities,
      Compiler: wasm.Compiler as unknown as WasmModule["Compiler"],
    };
    wasmModule = module;
    return module;
  })();

  return loadPromise!;
}

export function isWasmLoaded(): boolean {
  return wasmModule !== null;
}

export function isUsingMock(): boolean {
  return false;
}

export function getWasm(): WasmModule | null {
  return wasmModule;
}
