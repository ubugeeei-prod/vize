import convertSourceMap from "convert-source-map";
import type { SfcCompileResultNapi } from "@vizejs/native";
import { getCompiler } from "./native.ts";
import { composeInputMap } from "./source-map.ts";
import type {
  BabelFile,
  BabelParse,
  FileAst,
  VizeBabelOptions,
  VizeBabelSfcMetadata,
} from "./types.ts";

export interface CompiledSfc {
  ast: FileAst;
  code: string;
  map: string | null;
  metadata: VizeBabelSfcMetadata;
}

export function isVueSfc(filename: string): boolean {
  return /\.vue$/iu.test(filename);
}

export function compileSfcForBabel(
  source: string,
  filename: string,
  parserOptions: Record<string, unknown>,
  parse: BabelParse,
  options: Required<VizeBabelOptions>,
): CompiledSfc {
  const native = getCompiler();
  const scopeId = native.generateSfcScopeId(filename, options.root, options.isProduction, source);
  const result = native.compileSfc(source, {
    filename,
    mode: "module",
    sourceMap: true,
    ssr: options.ssr,
    vapor: options.jsxMode === "vapor",
    scopeId: `data-v-${scopeId}`,
    isProd: options.isProduction,
    inlineTemplate: options.isProduction && !options.ssr,
  });
  assertSupported(result, filename, options.sfcStyles);
  const completed = completeModule(result, scopeId, options.sfcStyles);
  const ast = parse(completed.code, {
    ...parserOptions,
    filename,
    sourceType: "module",
  });
  return {
    ast,
    code: completed.code,
    map: completed.map,
    metadata: {
      defaultMode: options.jsxMode,
      ssr: options.ssr,
      nativePasses: 1,
      warnings: result.warnings,
      css: result.css ?? null,
      styles: result.styles,
      dependencies: result.dependencies ?? [],
    },
  };
}

export function installSfcFileState(file: BabelFile, compiled: CompiledSfc): void {
  file.code = compiled.code;
  if (file.opts.sourceMaps) {
    if (!compiled.map) {
      throw new Error("The native SFC compiler did not provide the requested original-source map");
    }
    const parsed: unknown = JSON.parse(compiled.map);
    if (!isSourceMap(parsed))
      throw new Error("The native SFC compiler returned an invalid source map");
    const upstream = file.inputMap?.toObject();
    file.inputMap = upstream
      ? composeInputMap(compiled.map, upstream)
      : convertSourceMap.fromObject(parsed);
  }
  const old = file.metadata.vize;
  file.metadata.vize = {
    ...(typeof old === "object" && old !== null ? old : {}),
    sfc: compiled.metadata,
  };
  for (const warning of compiled.metadata.warnings) {
    process.emitWarning(`[vize] ${warning}`, { type: "VizeSfcWarning" });
  }
}

function assertSupported(
  result: SfcCompileResultNapi,
  filename: string,
  styleMode: Required<VizeBabelOptions>["sfcStyles"],
): void {
  if (result.errors.length > 0) throw new Error(result.errors.join("\n"));
  if (result.customBlocks.length > 0 || result.macroArtifacts.length > 0) {
    throw new Error(
      `${filename}: custom blocks and emitted macro artifacts need a bundler adapter and are not supported by standalone Babel`,
    );
  }
  for (const style of result.styles) {
    if (style.src) {
      throw new Error(
        `${filename}: <style src> needs a bundler CSS pipeline and is not supported by standalone Babel`,
      );
    }
    if (style.lang && style.lang !== "css") {
      throw new Error(
        `${filename}: <style lang="${style.lang}"> needs a bundler CSS pipeline and is not supported by standalone Babel`,
      );
    }
    if (style.content.includes("@import")) {
      throw new Error(
        `${filename}: CSS @import needs a bundler CSS pipeline and is not supported by standalone Babel`,
      );
    }
  }
  if (result.css && styleMode === "error") {
    throw new Error(
      `${filename}: SFC styles require sfcStyles: "inject" or "metadata" in standalone Babel`,
    );
  }
}

function completeModule(
  result: SfcCompileResultNapi,
  scopeId: string,
  styleMode: Required<VizeBabelOptions>["sfcStyles"],
): { code: string; map: string | null } {
  let code = result.code;
  if (result.hasScoped) code = attachScope(code, scopeId);
  const prefix = result.css && styleMode === "inject" ? styleInjection(result.css, scopeId) : "";
  return { code: prefix + code, map: shiftMap(result.map, prefix) };
}

function attachScope(code: string, scopeId: string): string {
  if (/^export default _sfc_main/mu.test(code)) {
    return code.replace(
      /^export default _sfc_main/mu,
      `_sfc_main.__scopeId = "data-v-${scopeId}";\nexport default _sfc_main`,
    );
  }
  if (/^export default /mu.test(code)) {
    const rewritten = code.replace(/^export default /mu, "const _sfc_main = ");
    return `${rewritten}\n_sfc_main.__scopeId = "data-v-${scopeId}";\nexport default _sfc_main;\n`;
  }
  if (/^export function render\b/mu.test(code)) {
    return `${code}\nconst _sfc_main = { render };\n_sfc_main.__scopeId = "data-v-${scopeId}";\nexport default _sfc_main;\n`;
  }
  return code;
}

function styleInjection(css: string, scopeId: string): string {
  const cssCode = JSON.stringify(css);
  const cssId = JSON.stringify(`vize-style-${scopeId}`);
  return `export const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      document.head.appendChild(style);
    }
    style.textContent = __vize_css__;
  }
})();
`;
}

function shiftMap(mapJson: string | undefined, prefix: string): string | null {
  if (!mapJson) return null;
  if (!prefix) return mapJson;
  const parsed: unknown = JSON.parse(mapJson);
  if (!isSourceMap(parsed)) return null;
  const lines = prefix.split("\n").length - 1;
  return JSON.stringify({ ...parsed, mappings: ";".repeat(lines) + parsed.mappings });
}

function isSourceMap(value: unknown): value is {
  version: 3;
  sources: string[];
  names: string[];
  mappings: string;
} {
  return (
    typeof value === "object" &&
    value !== null &&
    "version" in value &&
    value.version === 3 &&
    "sources" in value &&
    Array.isArray(value.sources) &&
    "names" in value &&
    Array.isArray(value.names) &&
    "mappings" in value &&
    typeof value.mappings === "string"
  );
}
