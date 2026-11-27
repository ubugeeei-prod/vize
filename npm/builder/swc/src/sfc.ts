import { createRequire } from "node:module";
import { transform, transformSync, type Options, type Output } from "@swc/core";
import type { SfcCompileResultNapi } from "@vizejs/native";
import { MappedModule, parseSourceMap } from "@vizejs/builder-shared/mapped-module";
import { bindDefaultExport } from "@vizejs/builder-shared/native-shape";
import { createGeneratedNameAllocator } from "@vizejs/builder-shared/module-output";

/** Options shared by the JSX Wasm plugin. */
export interface VizeSwcOptions {
  /** Default renderer, overridden by component mode directives. @default "vdom" */
  jsxMode?: "vdom" | "vapor";
  /** JSX compatibility semantics. @default "native" */
  jsxCompat?: "native" | "babel";
  /** Generate server render functions. @default false */
  ssr?: boolean;
}

/** SFC compilation runs before SWC's JavaScript parser. */
export interface VizeSfcOptions {
  /** Renderer for the component. @default "vdom" */
  mode?: "vdom" | "vapor";
  /** Generate server render functions. @default false */
  ssr?: boolean;
  /** CSS handling; metadata consumers must emit the CSS themselves. @default "inject" */
  styles?: "inject" | "metadata" | "error";
  /** Project root for stable scope IDs. @default process.cwd() */
  root?: string;
  /** Production compilation. @default NODE_ENV === "production" */
  isProduction?: boolean;
}

/** SWC options with an explicit Vue filename and native compiler options. */
export interface VueTransformOptions extends Options {
  filename: string;
  vize?: VizeSfcOptions;
}

/** Style and dependency data for hosts that consume a standalone SFC transform. */
export interface VueTransformOutput extends Output {
  vize: {
    css: string;
    scopeId: string;
    warnings: string[];
    dependencies: string[];
  };
}

/** Compile a Vue SFC, then run SWC's configured target/module passes. */
export async function transformVue(
  source: string,
  options: VueTransformOptions,
): Promise<VueTransformOutput> {
  const prepared = prepare(source, options);
  return { ...(await transform(prepared.code, prepared.swc)), vize: prepared.metadata };
}

/** Synchronous counterpart to transformVue. */
export function transformVueSync(source: string, options: VueTransformOptions): VueTransformOutput {
  const prepared = prepare(source, options);
  return { ...transformSync(prepared.code, prepared.swc), vize: prepared.metadata };
}

function prepare(source: string, options: VueTransformOptions) {
  if (!options || !/\.vue$/iu.test(options.filename)) {
    throw new TypeError("transformVue requires a .vue filename");
  }
  const { vize = {}, ...swc } = options;
  validateOptions(vize);
  // The input to SWC is generated TypeScript. An implicit .swcrc selection can
  // exclude .vue or replace that parser; callers pass their SWC options here.
  if (swc.swcrc || swc.configFile)
    throw new Error(
      "Pass SWC configuration directly to transformVue; swcrc/configFile are unsupported",
    );
  if (swc.inputSourceMap)
    throw new Error(
      "SFC inputSourceMap is unsupported; the compiler maps directly to the authored .vue file",
    );
  const require = createRequire(typeof __filename === "string" ? __filename : import.meta.url);
  const native = require("@vizejs/native") as typeof import("@vizejs/native");
  const parsed = native.parseSfc(source, { filename: options.filename, includeAttrs: true });
  for (const block of [parsed.template, parsed.script, parsed.scriptSetup, ...parsed.styles]) {
    if (block?.attrs?.some((attr) => attr.name === "src")) {
      throw new Error("External SFC blocks require a bundler adapter");
    }
  }
  const ssr = vize.ssr ?? false;
  const styles = vize.styles ?? "inject";
  const production = vize.isProduction ?? process.env.NODE_ENV === "production";
  const scope = native.generateSfcScopeId(
    options.filename,
    vize.root ?? process.cwd(),
    production,
    source,
  );
  const result = native.compileSfc(source, {
    filename: options.filename,
    mode: "module",
    sourceMap: Boolean(swc.sourceMaps ?? swc.sourceMap),
    ssr,
    vapor: vize.mode === "vapor",
    scopeId: `data-v-${scope}`,
    isTs: true,
    isProd: production,
    inlineTemplate: production && !ssr,
  });
  assertSupported(result, styles);
  const maps = Boolean(swc.sourceMaps ?? swc.sourceMap);
  const nativeMap = parseSourceMap(result.map);
  if (maps && result.map && !nativeMap) throw new Error("Vize returned an invalid SFC source map");
  // Macro-only/static modules can have no authored runtime origins. Preserve
  // that fact with an empty map instead of mapping generated JS to SFC offsets.
  const module = new MappedModule(
    result.code,
    maps
      ? (nativeMap ?? {
          version: 3,
          sources: [options.filename],
          sourcesContent: [source],
          names: [],
          mappings: "",
        })
      : null,
  );
  const shape = result.moduleShape;
  if (!shape) throw new Error("Vize did not return the compiled SFC module shape");
  let component = shape.defaultExportLocal;
  if (shape.defaultExportIsSfcMain) component = "_sfc_main";
  if (!shape.hasDefaultExport || (result.hasScoped && !component)) {
    component = createGeneratedNameAllocator(module.code)("__vize_swc_component");
    if (shape.hasDefaultExport) {
      bindDefaultExport(module, shape, component);
      module.append(`\nexport default ${component};\n`);
    } else {
      const render = shape.hasNamedSsrRenderExport
        ? "ssrRender"
        : shape.hasNamedRenderExport
          ? "render"
          : "";
      module.append(`\nconst ${component} = { ${render} };\nexport default ${component};\n`);
    }
  }
  if (result.hasScoped && component)
    module.append(`\n${component}.__scopeId = ${JSON.stringify(`data-v-${scope}`)};\n`);
  if (result.css && styles === "inject" && !ssr) {
    // Function-local names cannot collide with authored module bindings.
    module.append(`\n(function () {
  if (typeof document === "undefined") return;
  const id = ${JSON.stringify(`vize-swc-${scope}`)};
  let style = document.getElementById(id);
  if (!style) { style = document.createElement("style"); style.id = id; document.head.appendChild(style); }
  style.textContent = ${JSON.stringify(result.css)};
})();\n`);
  }
  const inputMap =
    module.map ??
    (!nativeMap?.mappings
      ? {
          version: 3,
          sources: [options.filename],
          sourcesContent: [source],
          names: [],
          mappings: "",
        }
      : null);
  if (maps && !inputMap) throw new Error("SFC module assembly lost the native source map");
  for (const warning of result.warnings)
    process.emitWarning(`[vize] ${warning}`, { type: "VizeSfcWarning" });
  return {
    code: module.code,
    swc: {
      ...swc,
      swcrc: false,
      configFile: false,
      inputSourceMap: maps ? JSON.stringify(inputMap) : false,
      jsc: { ...swc.jsc, parser: { syntax: "typescript", tsx: true, decorators: true } },
    } satisfies Options,
    metadata: {
      css: result.css ?? "",
      scopeId: `data-v-${scope}`,
      warnings: result.warnings,
      dependencies: result.dependencies ?? [],
    },
  };
}

function assertSupported(
  result: SfcCompileResultNapi,
  styles: "inject" | "metadata" | "error",
): void {
  if (result.errors.length) throw new Error(result.errors.join("\n"));
  if (result.customBlocks.length || result.macroArtifacts.length)
    throw new Error("SFC custom blocks and emitted macro artifacts require a bundler adapter");
  for (const style of result.styles) {
    if (
      style.src ||
      (style.lang && style.lang !== "css") ||
      style.module ||
      /@import\b/iu.test(style.content)
    ) {
      throw new Error(
        "External styles, preprocessors, CSS modules and CSS @import require a bundler adapter",
      );
    }
  }
  if (styles === "error" && result.css)
    throw new Error('SFC styles require styles: "inject" or "metadata"');
}

function validateOptions(value: VizeSfcOptions): void {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new TypeError("Vize SFC options must be an object");
  for (const key of Object.keys(value)) {
    if (!["mode", "ssr", "styles", "root", "isProduction"].includes(key))
      throw new TypeError(`Unknown Vize SFC option: ${key}`);
  }
  if (value.mode !== undefined && value.mode !== "vdom" && value.mode !== "vapor")
    throw new TypeError('mode must be "vdom" or "vapor"');
  if (value.styles !== undefined && !["inject", "metadata", "error"].includes(value.styles))
    throw new TypeError('styles must be "inject", "metadata", or "error"');
  for (const key of ["ssr", "isProduction"] as const) {
    if (value[key] !== undefined && typeof value[key] !== "boolean")
      throw new TypeError(`${key} must be a boolean`);
  }
  if (value.root !== undefined && (typeof value.root !== "string" || !value.root))
    throw new TypeError("root must be a non-empty string");
}
