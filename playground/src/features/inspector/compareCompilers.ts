import {
  compileScript,
  compileTemplate,
  parse,
  type BindingMetadata,
  type SFCDescriptor,
} from "vue/compiler-sfc";
import type { CompilerOptions, WasmModule } from "../../wasm/index";
import { formatCode } from "../atelier/formatters";
import { buildLineDiff, getDiffStats } from "./diff";
import type {
  CompilerRun,
  InspectorFile,
  InspectorOptions,
  InspectorReport,
  InspectorTarget,
} from "./types";

const DEFAULT_OPTIONS: InspectorOptions = {
  customRenderer: false,
  vueParserQuirks: false,
};

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function normalizeCompilerMessages(messages: unknown[] | undefined): string[] {
  return (messages ?? []).map((message) => {
    if (message instanceof Error) return message.message;
    if (typeof message === "object" && message && "message" in message) {
      return String((message as { message: unknown }).message);
    }
    return String(message);
  });
}

function descriptorUsesTypeScript(descriptor: SFCDescriptor): boolean {
  const langs = [descriptor.script?.lang, descriptor.scriptSetup?.lang];
  return langs.some((lang) => lang === "ts" || lang === "tsx");
}

function outputText(run: CompilerRun): string {
  return run.error ?? run.formattedCode ?? run.code;
}

async function formatRunCode(code: string, parser: CompilerRun["parser"]): Promise<string> {
  if (!code) return "";
  return formatCode(code, parser);
}

async function compileOfficialVue(
  file: InspectorFile,
  target: InspectorTarget,
): Promise<CompilerRun> {
  const start = performance.now();

  try {
    const parsed = parse(file.source, { filename: file.path });
    const descriptor = parsed.descriptor;
    const isTypeScript = descriptorUsesTypeScript(descriptor);
    const parser = isTypeScript ? "typescript" : "babel";
    const warnings = normalizeCompilerMessages(parsed.errors);
    let bindingMetadata: BindingMetadata = {};
    let scriptCode = "";

    if (descriptor.script || descriptor.scriptSetup) {
      const script = compileScript(descriptor, {
        id: file.path,
        inlineTemplate: false,
      });
      scriptCode = script.content;
      bindingMetadata = script.bindings;
    }

    let templateCode = "";
    if (descriptor.template) {
      const template = compileTemplate({
        source: descriptor.template.content,
        filename: file.path,
        id: file.path,
        scoped: descriptor.styles.some((style) => style.scoped),
        ssr: target === "ssr",
        compilerOptions: {
          bindingMetadata,
          expressionPlugins: isTypeScript ? ["typescript"] : undefined,
        },
      });
      templateCode = template.code;
      warnings.push(...normalizeCompilerMessages(template.errors));
      warnings.push(...normalizeCompilerMessages(template.tips));
    }

    const code = [scriptCode, templateCode].filter(Boolean).join("\n\n");
    const formattedCode = await formatRunCode(code, parser);
    return {
      label: "@vue/compiler-sfc",
      code,
      formattedCode,
      parser,
      warnings,
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      label: "@vue/compiler-sfc",
      code: "",
      formattedCode: "",
      parser: "babel",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

async function compileVize(
  compiler: WasmModule,
  file: InspectorFile,
  target: InspectorTarget,
  options: InspectorOptions,
): Promise<CompilerRun> {
  const start = performance.now();

  try {
    const compileOptions: CompilerOptions = {
      mode: "module",
      filename: file.path,
      ssr: target === "ssr",
      scriptExt: "preserve",
      outputMode: "vdom",
      customRenderer: options.customRenderer,
      vueParserQuirks: options.vueParserQuirks,
    };
    const result = compiler.compileSfc(file.source, compileOptions);
    const code = result.script?.code || result.template?.code || "";
    const parser = descriptorUsesTypeScript(result.descriptor as SFCDescriptor)
      ? "typescript"
      : "babel";
    const formattedCode = await formatRunCode(code, parser);
    return {
      label: "Vize",
      code,
      formattedCode,
      parser,
      warnings: result.warnings ?? [],
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      label: "Vize",
      code: "",
      formattedCode: "",
      parser: "babel",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

export async function compileInspectorReport({
  compiler,
  file,
  target,
  options,
}: {
  compiler: WasmModule;
  file: InspectorFile;
  target: InspectorTarget;
  options?: Partial<InspectorOptions>;
}): Promise<InspectorReport> {
  const normalizedOptions = { ...DEFAULT_OPTIONS, ...options };
  const [official, vize] = await Promise.all([
    compileOfficialVue(file, target),
    compileVize(compiler, file, target, normalizedOptions),
  ]);
  const diff = buildLineDiff(outputText(official), outputText(vize));

  return {
    filename: file.path,
    target,
    official,
    vize,
    diff,
    stats: getDiffStats(diff),
  };
}
