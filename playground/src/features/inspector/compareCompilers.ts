import {
  compileScript,
  compileTemplate,
  parse,
  type BindingMetadata,
  type SFCDescriptor,
} from "vue/compiler-sfc";
import type {
  CompilerOptions,
  CroquisDiagnostic,
  TypeCheckDiagnostic,
  WasmModule,
} from "../../wasm/index";
import { formatCode } from "../atelier/formatters";
import { buildLineDiff, getDiffStats } from "./diff";
import { buildInspectorGraph } from "./graph";
import type {
  CompilerRun,
  InspectorFile,
  InspectorGraphRun,
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

function formatTypeCheckDiagnostic(diagnostic: TypeCheckDiagnostic): string {
  const code = diagnostic.code ? ` ${diagnostic.code}` : "";
  return `${diagnostic.severity}${code}: ${diagnostic.message}`;
}

function formatCroquisDiagnostic(diagnostic: CroquisDiagnostic): string {
  const code = diagnostic.code ? ` ${diagnostic.code}` : "";
  return `${diagnostic.severity}${code}: ${diagnostic.message}`;
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

async function inspectVirtualTs(compiler: WasmModule, file: InspectorFile): Promise<CompilerRun> {
  const start = performance.now();

  try {
    const result = compiler.typeCheck(file.source, {
      filename: file.path,
      includeVirtualTs: true,
    });
    const code = result.virtualTs ?? "";
    const formattedCode = await formatRunCode(code, "typescript");

    return {
      label: "Virtual TS",
      code,
      formattedCode,
      parser: "typescript",
      warnings: result.diagnostics.map(formatTypeCheckDiagnostic),
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      label: "Virtual TS",
      code: "",
      formattedCode: "",
      parser: "typescript",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

function inspectVir(compiler: WasmModule, file: InspectorFile): CompilerRun {
  const start = performance.now();

  try {
    const result = compiler.analyzeSfc(file.source, { filename: file.path });
    const code = result.vir ?? "";

    return {
      label: "VIR",
      code,
      formattedCode: code,
      parser: "babel",
      warnings: result.diagnostics.map(formatCroquisDiagnostic),
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      label: "VIR",
      code: "",
      formattedCode: "",
      parser: "babel",
      warnings: [],
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

function inspectCrossFileGraph(compiler: WasmModule, files: InspectorFile[]): InspectorGraphRun {
  const start = performance.now();
  const graph = buildInspectorGraph(files);

  try {
    const result = compiler.analyzeCrossFile(
      files.map((file) => ({ path: file.path, source: file.source })),
      { all: true, maxImportDepth: 10 },
    );
    const issueCounts = new Map<string, number>();
    for (const diagnostic of result.diagnostics) {
      issueCounts.set(diagnostic.file, (issueCounts.get(diagnostic.file) ?? 0) + 1);
    }

    return {
      nodes: graph.nodes.map((node) => ({
        ...node,
        issueCount: issueCounts.get(node.path) ?? 0,
      })),
      edges: graph.edges,
      diagnostics: result.diagnostics,
      circularDependencies: result.circularDependencies,
      stats: result.stats,
      error: null,
      timeMs: performance.now() - start,
    };
  } catch (error) {
    return {
      nodes: graph.nodes,
      edges: graph.edges,
      diagnostics: [],
      circularDependencies: [],
      stats: null,
      error: toErrorMessage(error),
      timeMs: performance.now() - start,
    };
  }
}

export async function compileInspectorReport({
  compiler,
  file,
  files,
  target,
  options,
}: {
  compiler: WasmModule;
  file: InspectorFile;
  files?: InspectorFile[];
  target: InspectorTarget;
  options?: Partial<InspectorOptions>;
}): Promise<InspectorReport> {
  const normalizedOptions = { ...DEFAULT_OPTIONS, ...options };
  const inspectedFiles = files?.length ? files : [file];
  const [official, vize, virtualTs, vir, graph] = await Promise.all([
    compileOfficialVue(file, target),
    compileVize(compiler, file, target, normalizedOptions),
    inspectVirtualTs(compiler, file),
    Promise.resolve(inspectVir(compiler, file)),
    Promise.resolve(inspectCrossFileGraph(compiler, inspectedFiles)),
  ]);
  const diff = buildLineDiff(outputText(official), outputText(vize));

  return {
    filename: file.path,
    target,
    official,
    vize,
    virtualTs,
    vir,
    graph,
    diff,
    stats: getDiffStats(diff),
  };
}
