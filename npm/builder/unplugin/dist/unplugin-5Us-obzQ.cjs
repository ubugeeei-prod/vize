//#region \0rolldown/runtime.js
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __copyProps = (to, from, except, desc) => {
  if ((from && typeof from === "object") || typeof from === "function")
    for (var keys = __getOwnPropNames(from), i = 0, n = keys.length, key; i < n; i++) {
      key = keys[i];
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, {
          get: ((k) => from[k]).bind(null, key),
          enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable,
        });
    }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (
  (target = mod != null ? __create(__getProtoOf(mod)) : {}),
  __copyProps(
    isNodeMode || !mod || !mod.__esModule
      ? __defProp(target, "default", {
          value: mod,
          enumerable: true,
        })
      : target,
    mod,
  )
);
//#endregion
let node_fs = require("node:fs");
node_fs = __toESM(node_fs, 1);
let node_module = require("node:module");
let unplugin = require("unplugin");
let node_crypto = require("node:crypto");
let _vizejs_native = require("@vizejs/native");
_vizejs_native = __toESM(_vizejs_native, 1);
let _vizejs_source_map = require("@vizejs/source-map");
let oxc_transform = require("oxc-transform");
//#region src/filter.ts
function createFilter(include, exclude) {
  const includePatterns = include
    ? Array.isArray(include)
      ? include
      : [include]
    : [/\.vue$/, /\.[jt]sx$/];
  const excludePatterns = exclude
    ? Array.isArray(exclude)
      ? exclude
      : [exclude]
    : [/node_modules/];
  return (id) => {
    const matchInclude = includePatterns.some((pattern) => matchesPattern(pattern, id));
    const matchExclude = excludePatterns.some((pattern) => matchesPattern(pattern, id));
    return matchInclude && !matchExclude;
  };
}
function matchesPattern(pattern, id) {
  if (typeof pattern === "string") return id.includes(pattern);
  pattern.lastIndex = 0;
  const matches = pattern.test(id);
  pattern.lastIndex = 0;
  return matches;
}
//#endregion
//#region src/style.ts
const PREPROCESSOR_LANGS = new Set(["scss", "sass", "less", "stylus", "styl"]);
function needsPreprocessor(block) {
  return block.lang !== null && PREPROCESSOR_LANGS.has(block.lang);
}
function isCssModule(block) {
  return block.module !== false;
}
function hasDelegatedStyles(compiled) {
  return compiled.styles.some((style) => needsPreprocessor(style) || isCssModule(style));
}
function generateScopeId(filename, root, isProduction, source) {
  return (0, _vizejs_native.generateSfcScopeId)(filename, root, isProduction, source);
}
function supportsTemplateOnlyHmr(output) {
  return /(?:^|\n)(?:_sfc_main|__sfc__)\.render\s*=\s*render\b/m.test(output);
}
/**
 * Prepend a runtime `<style>` injection for plain CSS to a module's output.
 *
 * This is the same inline-CSS path plain SFC `<style>` blocks use (see
 * {@link generateOutput}): a guarded `document.createElement("style")` keyed by
 * a stable id, so the rule is appended once and idempotently. `styleKey` seeds
 * the element id (deduping re-injection across HMR/multiple imports). Used for
 * both SFC plain CSS and JSX `<style scoped>` CSS (#1495, #1533), whose content
 * is already scope-rewritten by the compiler.
 */
function prependInlineStyleInjection(output, css, styleKey) {
  return `
export const __vize_css__ = ${JSON.stringify(css)};
const __vize_css_id__ = ${JSON.stringify(`vize-style-${styleKey}`)};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${output}`;
}
function generateOutput(compiled, options) {
  const { isProduction, isDev, extractCss, filePath } = options;
  let output = compiled.code;
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasNamedRenderExport = /^export function render\b/m.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);
  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    if (compiled.hasScoped) output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined && compiled.hasScoped)
    output = output.replace(
      /^export default _sfc_main/m,
      `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
    );
  else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    output += "\nconst _sfc_main = {};";
    if (compiled.hasScoped) output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    output += "\n_sfc_main.render = render;";
    output += "\nexport default _sfc_main;";
  }
  if (hasDelegatedStyles(compiled) && filePath) {
    const styleImports = [];
    const cssModuleImports = [];
    for (const block of compiled.styles) {
      const lang = block.lang ?? "css";
      const params = new URLSearchParams();
      params.set("vue", "");
      params.set("type", "style");
      params.set("index", String(block.index));
      params.set("lang", lang);
      if (block.scoped) params.set("scoped", `data-v-${compiled.scopeId}`);
      const importUrl = `${filePath}?${params.toString()}`;
      if (isCssModule(block)) {
        const bindingName = typeof block.module === "string" ? block.module : "$style";
        const moduleParams = new URLSearchParams(params);
        moduleParams.set("module", typeof block.module === "string" ? block.module : "");
        cssModuleImports.push(
          `import ${bindingName} from ${JSON.stringify(`${filePath}?${moduleParams.toString()}`)};`,
        );
      } else styleImports.push(`import ${JSON.stringify(importUrl)};`);
    }
    const allImports = [...styleImports, ...cssModuleImports].join("\n");
    if (allImports) output = `${allImports}\n${output}`;
    if (cssModuleImports.length > 0) {
      const cssModuleSetup = compiled.styles
        .filter((block) => isCssModule(block))
        .map((block) => {
          const bindingName = typeof block.module === "string" ? block.module : "$style";
          return `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(bindingName)}] = ${bindingName};`;
        })
        .join("\n");
      output = output.replace(
        /^export default _sfc_main;?$/m,
        `${cssModuleSetup}\nexport default _sfc_main;`,
      );
    }
  } else if (compiled.css && !(isProduction && extractCss))
    output = prependInlineStyleInjection(output, compiled.css, compiled.scopeId);
  if (!isProduction && isDev && hasExportDefault && supportsTemplateOnlyHmr(output)) output += "";
  return output;
}
function wrapScopedPreprocessorStyle(content, scoped, lang) {
  return (0, _vizejs_native.wrapSfcScopedPreprocessorStyle)(content, scoped, lang);
}
function toStyleBlockInfo(block) {
  return {
    content: block.content,
    src: block.src ?? null,
    lang: block.lang ?? null,
    scoped: block.scoped,
    module: block.module ? (block.moduleName ?? true) : false,
    index: block.index,
  };
}
//#endregion
//#region src/compiler.ts
const { compileSfc, compileJsx } = _vizejs_native;
function buildSignature(options) {
  return [
    options.isProduction ? "1" : "0",
    options.ssr ? "1" : "0",
    options.vapor ? "1" : "0",
    options.customRenderer ? "1" : "0",
    options.templateSyntax,
    options.experimentalInTagComments ? "1" : "0",
    options.experimentalPatternedTemplate ? "1" : "0",
    options.experimentalServerScript ? "1" : "0",
    options.sourceMap ? "1" : "0",
    options.mode,
    options.runtimeModuleName,
    options.runtimeGlobalName,
    String(options.vueVersion),
    options.hostCompiler ? "1" : "0",
    options.root,
  ].join(":");
}
function buildSourceHash(source) {
  return (0, node_crypto.createHash)("sha256").update(source).digest("hex");
}
function compileVueModule(filePath, source, options, cache) {
  const sourceHash = buildSourceHash(source);
  const signature = buildSignature(options);
  const cached = cache.get(filePath);
  if (cached && cached.sourceHash === sourceHash && cached.signature === signature)
    return {
      compiled: cached.compiled,
      warnings: [],
    };
  const scopeId = generateScopeId(filePath, options.root, options.isProduction, source);
  const result = compileSfc(source, {
    filename: filePath,
    mode: options.mode,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    vapor: options.vapor,
    customRenderer: options.customRenderer,
    templateSyntax: options.templateSyntax,
    experimentalInTagComments: options.experimentalInTagComments,
    experimentalPatternedTemplate: options.experimentalPatternedTemplate,
    experimentalServerScript: options.experimentalServerScript,
    runtimeModuleName: options.runtimeModuleName,
    runtimeGlobalName: options.runtimeGlobalName,
    vueVersion: String(options.vueVersion),
    scopeId: `data-v-${scopeId}`,
  });
  if (result.errors.length > 0) throw new Error(result.errors.join("\n"));
  const compiled = {
    code: result.code,
    map: (0, _vizejs_source_map.parseSourceMap)(result.map),
    css: result.css,
    scopeId,
    hasScoped: result.hasScoped,
    templateHash: result.templateHash,
    styleHash: result.styleHash,
    scriptHash: result.scriptHash,
    macroArtifacts: result.macroArtifacts ?? [],
    styles: result.styles.map(toStyleBlockInfo),
  };
  cache.set(filePath, {
    compiled,
    sourceHash,
    signature,
  });
  return {
    compiled,
    warnings: result.warnings,
  };
}
function compileJsxModule(filePath, source, options) {
  const result = compileJsx(source, {
    filename: filePath,
    lang: filePath.endsWith(".tsx") ? "tsx" : "jsx",
    jsxMode: options.jsxMode,
    vapor: options.vapor,
    sourceMap: options.sourceMap,
  });
  if (result.errors.length > 0) throw new Error(result.errors.join("\n"));
  const css = (result.scopedStyles ?? []).map((style) => style.css).join("\n");
  let code = result.code;
  if (css) {
    const styleKey = result.scopedStyles[0].scopeId.replace(/^data-v-/, "");
    code = prependInlineStyleInjection(code, css, styleKey);
  }
  const map = (0, _vizejs_source_map.offsetEmbeddedSourceMap)(result.code, code, result.map);
  return {
    code,
    map,
    warnings: result.warnings,
  };
}
//#endregion
//#region src/request.ts
const STYLE_MARKER = ".__vize_style_";
function isVueFile(id) {
  return id.endsWith(".vue");
}
function isJsxFile(id) {
  return id.endsWith(".jsx") || id.endsWith(".tsx");
}
function isVueStyleRequest(id) {
  if (!id.includes("?vue")) return false;
  const { query } = parseVueRequest(id);
  return query.vue && query.type === "style";
}
function isVirtualStyleId(id) {
  return id.includes(STYLE_MARKER);
}
function parseVueRequest(id) {
  const [path, rawQuery = ""] = id.split("?", 2);
  const params = new URLSearchParams(rawQuery);
  const filename = params.get("vize-file") ?? path;
  const moduleValue = params.has("module") ? params.get("module") || true : false;
  const indexValue = params.get("index");
  return {
    filename,
    path,
    query: {
      vue: params.has("vue"),
      type: params.get("type"),
      index: indexValue === null ? null : Number.parseInt(indexValue, 10),
      lang: params.get("lang"),
      module: moduleValue,
      scoped: params.get("scoped"),
      vizeFile: params.get("vize-file"),
    },
  };
}
function createVirtualStyleId(id) {
  const { filename, query } = parseVueRequest(id);
  const index = query.index ?? 0;
  const lang = query.lang ?? "css";
  const suffix = query.module !== false ? `.module.${lang}` : `.${lang}`;
  const params = new URLSearchParams();
  params.set("vue", "");
  params.set("type", "style");
  params.set("index", String(index));
  params.set("lang", lang);
  params.set("vize-file", filename);
  if (query.scoped) params.set("scoped", query.scoped);
  if (query.module !== false)
    params.set("module", typeof query.module === "string" ? query.module : "");
  return `${filename}${STYLE_MARKER}${index}${suffix}?${params.toString()}`;
}
//#endregion
//#region src/strip-types.ts
function formatErrorMessage(error) {
  const parts = [error.message];
  if (error.helpMessage) parts.push(error.helpMessage);
  if (error.codeframe) parts.push(error.codeframe);
  return parts.join("\n");
}
async function stripTypeScript(filePath, code, sourceMap) {
  const result = await (0, oxc_transform.transform)(filePath, code, {
    lang: "ts",
    sourcemap: sourceMap,
    sourceType: "module",
  });
  const errors = result.errors ?? [];
  if (errors.length > 0) throw new Error(errors.map(formatErrorMessage).join("\n\n"));
  return {
    code: result.code,
    map: result.map ?? null,
  };
}
//#endregion
//#region src/unplugin.ts
const require$1 = (0, node_module.createRequire)(require("url").pathToFileURL(__filename).href);
function normalizeVueVersion(version) {
  return version ?? 3;
}
function isLegacyVueVersion(version) {
  return (
    version === "legacy" || version === 0.11 || version === 1 || version === 2 || version === "2.7"
  );
}
function normalizeTemplateSyntax(templateSyntax) {
  return templateSyntax ?? "standard";
}
function normalizeOptions(rawOptions = {}) {
  const isProduction = rawOptions.isProduction ?? process.env.NODE_ENV === "production";
  const compatibility = rawOptions.compatibility ?? {};
  const vueVersion = normalizeVueVersion(rawOptions.vueVersion ?? compatibility.vueVersion);
  const mode =
    rawOptions.mode ?? (compatibility.scriptSetupInStandalone === true ? "function" : "module");
  const hostCompiler = compatibility.hostCompiler ?? isLegacyVueVersion(vueVersion);
  const templateSyntax = normalizeTemplateSyntax(rawOptions.templateSyntax);
  return {
    include: rawOptions.include,
    exclude: rawOptions.exclude,
    compatibility,
    isProduction,
    ssr: rawOptions.ssr ?? false,
    sourceMap: rawOptions.sourceMap ?? !isProduction,
    mode,
    vapor: rawOptions.vapor ?? false,
    experimentalInTagComments: rawOptions.experimentalInTagComments ?? false,
    experimentalPatternedTemplate: rawOptions.experimentalPatternedTemplate ?? false,
    experimentalServerScript: rawOptions.experimentalServerScript ?? false,
    jsxMode: rawOptions.jsxMode,
    customRenderer: rawOptions.customRenderer ?? false,
    templateSyntax,
    runtimeModuleName: rawOptions.runtimeModuleName ?? "vue",
    runtimeGlobalName: rawOptions.runtimeGlobalName ?? "Vue",
    vueVersion,
    hostCompiler,
    root: rawOptions.root ?? process.cwd(),
    debug: rawOptions.debug ?? false,
  };
}
function createVueDefineMap(isProduction) {
  return {
    __VUE_OPTIONS_API__: JSON.stringify(true),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(!isProduction),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(!isProduction),
  };
}
function resolveWebpackDefinePlugin(compiler, webpackVersion) {
  if (webpackVersion !== 4 && compiler.webpack?.DefinePlugin) return compiler.webpack.DefinePlugin;
  try {
    return require$1("webpack").DefinePlugin ?? null;
  } catch {
    return null;
  }
}
function injectWebpackVueDefines(compiler, isProduction, webpackVersion, definePluginConstructor) {
  const DefinePlugin =
    definePluginConstructor ?? resolveWebpackDefinePlugin(compiler, webpackVersion);
  if (!DefinePlugin)
    throw new Error(
      "[vize] Could not resolve webpack DefinePlugin. Install webpack in the host project or disable the Vize compiler with compatibility.hostCompiler.",
    );
  const existingDefines = /* @__PURE__ */ new Set();
  for (const plugin of compiler.options.plugins ?? []) {
    const definitions = plugin.definitions;
    if (!definitions) continue;
    for (const key of Object.keys(definitions)) existingDefines.add(key);
  }
  const definitions = createVueDefineMap(isProduction);
  const missingDefinitions = {};
  for (const [key, value] of Object.entries(definitions))
    if (!existingDefines.has(key)) missingDefinitions[key] = value;
  if (Object.keys(missingDefinitions).length > 0)
    new DefinePlugin(missingDefinitions).apply(compiler);
}
async function loadStyleBlock(id, options, cache) {
  const request = parseVueRequest(id);
  const index = request.query.index ?? -1;
  if (index < 0) return "";
  let compiled = cache.get(request.filename)?.compiled;
  if (!compiled && node_fs.default.existsSync(request.filename)) {
    const source = node_fs.default.readFileSync(request.filename, "utf8");
    compiled = compileVueModule(request.filename, source, options, cache).compiled;
  }
  const block = compiled?.styles[index];
  if (!block) return "";
  return wrapScopedPreprocessorStyle(block.content, request.query.scoped, block.lang);
}
const vizeUnplugin = (0, unplugin.createUnplugin)((rawOptions = {}) => {
  const options = normalizeOptions(rawOptions);
  const filter = createFilter(options.include, options.exclude);
  const cache = /* @__PURE__ */ new Map();
  return {
    name: "unplugin-vize",
    resolveId(id) {
      if (options.hostCompiler) return null;
      if (isVueStyleRequest(id)) return createVirtualStyleId(id);
      return null;
    },
    loadInclude(id) {
      if (options.hostCompiler) return false;
      return isVirtualStyleId(id);
    },
    async load(id) {
      if (options.hostCompiler) return null;
      if (!isVirtualStyleId(id)) return null;
      return {
        code: await loadStyleBlock(id, options, cache),
        map: null,
      };
    },
    transformInclude(id) {
      if (options.hostCompiler) return false;
      if (isJsxFile(id)) return filter(id);
      if (!id.includes(".vue")) return false;
      const request = parseVueRequest(id);
      return !request.query.vue && isVueFile(request.filename) && filter(request.filename);
    },
    async transform(code, id) {
      if (options.hostCompiler) return null;
      if (isJsxFile(id)) {
        if (!filter(id)) return null;
        const { code: jsxCode, map: jsxMap, warnings } = compileJsxModule(id, code, options);
        for (const warning of warnings) this.warn(`[vize] ${warning}`);
        return {
          code: jsxCode,
          map: jsxMap,
        };
      }
      if (!isVueFile(id) || !filter(id)) return null;
      const { compiled, warnings } = compileVueModule(id, code, options, cache);
      for (const warning of warnings) this.warn(`[vize] ${warning}`);
      const generated = generateOutput(compiled, {
        isProduction: options.isProduction,
        isDev: false,
        filePath: id,
      });
      const generatedMap = (0, _vizejs_source_map.offsetEmbeddedSourceMap)(
        compiled.code,
        generated,
        compiled.map,
      );
      const transformed = await stripTypeScript(id, generated, options.sourceMap);
      return {
        code: transformed.code,
        map: (0, _vizejs_source_map.composeSourceMaps)(transformed.map, generatedMap),
      };
    },
    watchChange(id) {
      if (isVueFile(id)) cache.delete(id);
    },
    webpack(compiler) {
      if (!options.hostCompiler)
        injectWebpackVueDefines(
          compiler,
          options.isProduction,
          options.compatibility.webpackVersion,
        );
    },
    esbuild: {
      onResolveFilter: /\.(?:vue|[jt]sx)(?:$|\?)/,
      onLoadFilter: /\.(?:vue|[jt]sx)(?:$|\?)/,
      loader(_code, id) {
        const request = parseVueRequest(id);
        if (request.query.type === "style")
          return request.query.module !== false ? "local-css" : "css";
        return "js";
      },
      config(buildOptions) {
        if (options.hostCompiler) return;
        buildOptions.define = {
          ...createVueDefineMap(options.isProduction),
          ...buildOptions.define,
        };
      },
    },
  };
});
//#endregion
Object.defineProperty(exports, "compileVueModule", {
  enumerable: true,
  get: function () {
    return compileVueModule;
  },
});
Object.defineProperty(exports, "createFilter", {
  enumerable: true,
  get: function () {
    return createFilter;
  },
});
Object.defineProperty(exports, "generateOutput", {
  enumerable: true,
  get: function () {
    return generateOutput;
  },
});
Object.defineProperty(exports, "normalizeOptions", {
  enumerable: true,
  get: function () {
    return normalizeOptions;
  },
});
Object.defineProperty(exports, "vizeUnplugin", {
  enumerable: true,
  get: function () {
    return vizeUnplugin;
  },
});
