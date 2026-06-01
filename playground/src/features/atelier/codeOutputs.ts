import type { InputMode } from "../../presets";
import type {
  CompilerOptions,
  CompileResult,
  SfcCompileResult,
  WasmModule,
} from "../../wasm/index";
import { formatCode, transpileToJs } from "./formatters";

export const CODE_OUTPUT_TARGETS = ["dom", "ssr", "vapor", "vaporSsr"] as const;
export const SFC_CODE_OUTPUT_TARGETS = ["dom", "ssr", "vapor"] as const;
export const TEMPLATE_CODE_OUTPUT_TARGETS = CODE_OUTPUT_TARGETS;

export type CodeOutputTarget = (typeof CODE_OUTPUT_TARGETS)[number];
type SecondaryCodeOutputTarget = Exclude<CodeOutputTarget, "dom">;
type SfcSecondaryCodeOutputTarget = Exclude<SecondaryCodeOutputTarget, "vaporSsr">;

export interface CodeOutputVariant {
  code: string;
  formattedCode: string;
  formattedJsCode: string;
  isTypeScript: boolean;
  templates: string[];
  warnings: string[];
  error: string | null;
}

export type CodeOutputs = Record<CodeOutputTarget, CodeOutputVariant>;

export const CODE_OUTPUT_LABELS: Record<CodeOutputTarget, string> = {
  dom: "VDOM",
  ssr: "SSR",
  vapor: "Vapor",
  vaporSsr: "Vapor SSR",
};

function createEmptyCodeOutputVariant(): CodeOutputVariant {
  return {
    code: "",
    formattedCode: "",
    formattedJsCode: "",
    isTypeScript: false,
    templates: [],
    warnings: [],
    error: null,
  };
}

export function createEmptyCodeOutputs(): CodeOutputs {
  return {
    dom: createEmptyCodeOutputVariant(),
    ssr: createEmptyCodeOutputVariant(),
    vapor: createEmptyCodeOutputVariant(),
    vaporSsr: createEmptyCodeOutputVariant(),
  };
}

export function getCodeOutputTargets(inputMode: InputMode): readonly CodeOutputTarget[] {
  return inputMode === "template" ? TEMPLATE_CODE_OUTPUT_TARGETS : SFC_CODE_OUTPUT_TARGETS;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

async function formatVariantCode(code: string, isTypeScript: boolean) {
  if (!code) {
    return {
      formattedCode: "",
      formattedJsCode: "",
    };
  }

  const formattedCode = await formatCode(code, isTypeScript ? "typescript" : "babel");
  const formattedJsCode = isTypeScript ? await formatCode(await transpileToJs(code), "babel") : "";

  return {
    formattedCode,
    formattedJsCode,
  };
}

function isSfcTypeScript(result: SfcCompileResult): boolean {
  const scriptSetupLang = result.descriptor.scriptSetup?.lang;
  const scriptLang = result.descriptor.script?.lang;
  return (
    scriptSetupLang === "ts" ||
    scriptSetupLang === "tsx" ||
    scriptLang === "ts" ||
    scriptLang === "tsx"
  );
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function getSfcBindingExpression(name: string, bindingType: string): string | null {
  switch (bindingType) {
    case "props":
    case "props-aliased":
    case "setup-let":
    case "setup-reactive-const":
    case "setup-const":
    case "literal-const":
    case "external-module":
    case "js-global-universal":
    case "js-global-browser":
    case "js-global-node":
    case "js-global-deno":
    case "js-global-bun":
      return name;
    case "setup-ref":
      return `${name}.value`;
    case "setup-maybe-ref":
      return `_unref(${name})`;
    default:
      return null;
  }
}

function normalizeSfcTemplateCode(result: SfcCompileResult): string {
  const code = result.template?.code || result.script?.code || "";
  const bindingMetadata = result.script?.bindings;
  if (!code || !bindingMetadata?.isScriptSetup) {
    return code;
  }

  const bindings = Object.entries(bindingMetadata.bindings || {}).sort(
    ([left], [right]) => right.length - left.length,
  );
  let normalized = code;
  let needsUnrefImport = false;

  for (const [name, bindingType] of bindings) {
    const expression = getSfcBindingExpression(name, String(bindingType));
    if (!expression) {
      continue;
    }
    if (expression.includes("_unref(")) {
      needsUnrefImport = true;
    }
    normalized = normalized.replace(new RegExp(`_ctx\\.${escapeRegExp(name)}\\b`, "g"), expression);
  }

  if (needsUnrefImport && !normalized.includes("unref as _unref")) {
    normalized = `import { unref as _unref } from 'vue'\n${normalized}`;
  }

  return normalized;
}

async function buildTemplateVariant(result: CompileResult): Promise<CodeOutputVariant> {
  const code = result.code || "";
  const { formattedCode, formattedJsCode } = await formatVariantCode(code, false);
  return {
    code,
    formattedCode,
    formattedJsCode,
    isTypeScript: false,
    templates: result.templates || [],
    warnings: [],
    error: null,
  };
}

function getSfcWarnings(result: SfcCompileResult): string[] {
  return result.warnings || [];
}

async function buildSfcScriptVariant(result: SfcCompileResult): Promise<CodeOutputVariant> {
  const code = result.script?.code || result.template?.code || "";
  const isTypeScript = isSfcTypeScript(result);
  const { formattedCode, formattedJsCode } = await formatVariantCode(code, isTypeScript);
  return {
    code,
    formattedCode,
    formattedJsCode,
    isTypeScript,
    templates: result.template?.templates || [],
    warnings: getSfcWarnings(result),
    error: null,
  };
}

async function compileTemplateVariant(
  compiler: WasmModule,
  source: string,
  options: CompilerOptions,
  target: SecondaryCodeOutputTarget,
): Promise<CodeOutputVariant> {
  try {
    const result =
      target === "vapor" || target === "vaporSsr"
        ? compiler.compileVapor(source, {
            ...options,
            ssr: target === "vaporSsr",
          })
        : compiler.compile(source, { ...options, ssr: true });
    return await buildTemplateVariant(result);
  } catch (error) {
    return {
      ...createEmptyCodeOutputVariant(),
      error: getErrorMessage(error),
    };
  }
}

async function buildSfcTemplateVariant(result: SfcCompileResult): Promise<CodeOutputVariant> {
  const code = normalizeSfcTemplateCode(result);
  const { formattedCode, formattedJsCode } = await formatVariantCode(code, false);
  return {
    code,
    formattedCode,
    formattedJsCode,
    isTypeScript: false,
    templates: result.template?.templates || [],
    warnings: getSfcWarnings(result),
    error: null,
  };
}

async function compileSfcVariant(
  compiler: WasmModule,
  source: string,
  options: CompilerOptions,
  target: SfcSecondaryCodeOutputTarget,
): Promise<CodeOutputVariant> {
  try {
    const outputMode = target === "ssr" ? "vdom" : "vapor";
    const result = compiler.compileSfc(source, {
      ...options,
      ssr: target === "ssr",
      outputMode,
    });
    return await buildSfcTemplateVariant(result);
  } catch (error) {
    return {
      ...createEmptyCodeOutputVariant(),
      error: getErrorMessage(error),
    };
  }
}

interface CompileCodeOutputsParams {
  compiler: WasmModule;
  inputMode: InputMode;
  source: string;
  options: CompilerOptions;
  baseOutput: CompileResult | null;
  baseSfcResult: SfcCompileResult | null;
}

export async function compileCodeOutputs({
  compiler,
  inputMode,
  source,
  options,
  baseOutput,
  baseSfcResult,
}: CompileCodeOutputsParams): Promise<CodeOutputs> {
  const outputs = createEmptyCodeOutputs();

  if (inputMode === "sfc") {
    if (baseSfcResult) {
      outputs.dom = await buildSfcScriptVariant(baseSfcResult);
    }

    const [ssr, vapor] = await Promise.all([
      compileSfcVariant(compiler, source, options, "ssr"),
      compileSfcVariant(compiler, source, options, "vapor"),
    ]);

    outputs.ssr = ssr;
    outputs.vapor = vapor;
    return outputs;
  }

  if (baseOutput) {
    outputs.dom = await buildTemplateVariant(baseOutput);
  }

  const [ssr, vapor, vaporSsr] = await Promise.all([
    compileTemplateVariant(compiler, source, options, "ssr"),
    compileTemplateVariant(compiler, source, options, "vapor"),
    compileTemplateVariant(compiler, source, options, "vaporSsr"),
  ]);

  outputs.ssr = ssr;
  outputs.vapor = vapor;
  outputs.vaporSsr = vaporSsr;
  return outputs;
}
