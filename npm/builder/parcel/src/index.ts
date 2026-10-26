import path from "node:path";
import { Transformer } from "@parcel/plugin";
import type { TransformerResult } from "@parcel/types";
import {
  collectSfcTemplateAssetUrls,
  compileSfc,
  extractSfcSrcInfo,
  generateSfcScopeId,
  parseSfc,
  rewriteSfcTemplateAssetReferences,
} from "@vizejs/native";
import { fail, loadConfig, type VizeParcelOptions } from "./config.ts";
import { blockMap, editMap, nativeMap } from "./maps.ts";
import { hmrRuntime, instrumentHmr } from "./hmr.ts";
import { styleRequest } from "./style-request.ts";

export type { VizeParcelOptions } from "./config.ts";

interface ParsedSfc {
  styles: Array<{ loc: { start: number; end: number } }>;
}

/** The adapter only owns host integration; parsing, code generation and CSS are native. */
export default new Transformer<VizeParcelOptions>({
  loadConfig,
  async transform({ asset, config, options, logger }) {
    const source = await asset.getCode();
    const src = extractSfcSrcInfo(source, asset.filePath);
    if (src.scriptSrc || src.templateSrc) {
      fail(
        "External script/template blocks are not supported by the Parcel adapter yet. Inline the block or import its module explicitly.",
        asset.filePath,
      );
    }
    const ssr = asset.env.isNode();
    const isProd = options.mode === "production";
    const filename = path.relative(options.projectRoot, asset.filePath).replaceAll("\\", "/");
    const scope = `data-v-${generateSfcScopeId(asset.filePath, options.projectRoot, isProd, source)}`;
    const result = compileSfc(source, {
      ...config.compiler,
      filename,
      mode: "module",
      sourceMap: Boolean(asset.env.sourceMap),
      ssr,
      isProd,
      isTs: false,
      scopeId: scope,
    });
    if (result.errors.length) fail(result.errors.join("\n"), asset.filePath);
    if (result.customBlocks.length || result.macroArtifacts.length) {
      fail(
        "Custom blocks and emitted macro artifacts require an explicit Parcel adapter and are not supported yet.",
        asset.filePath,
      );
    }
    if (result.styles.some((style) => style.module)) {
      fail(
        "SFC CSS modules are not supported by the Parcel adapter yet. Use a separate .module.css import.",
        asset.filePath,
      );
    }
    for (const message of result.warnings) logger.warn({ message });

    const componentKey = `${asset.id}-vize-component`;
    const runtimeKey = `${asset.id}-vize-hmr`;
    const needsHmr = !ssr && !isProd && Boolean(options.hmrOptions);
    const children: TransformerResult[] = [];
    const styleImports: string[] = [];
    const descriptor = asset.env.sourceMap
      ? (parseSfc(source, { filename: asset.filePath }) as ParsedSfc)
      : undefined;
    for (const style of result.styles) {
      if (ssr) continue;
      const language = style.lang ?? "css";
      if (!["css", "scss", "sass", "less", "styl", "stylus"].includes(language)) {
        fail(`Unsupported style language: ${language}.`, asset.filePath);
      }
      if (style.src) {
        styleImports.push(
          `import ${JSON.stringify(styleRequest(style.src, scope, style.scoped, isProd ? filename : undefined))};`,
        );
      } else {
        const key = `${asset.id}-vize-style-${style.index}`;
        const loc = descriptor?.styles[style.index]?.loc;
        children.push({
          type: language === "styl" ? "stylus" : language,
          content: style.content,
          uniqueKey: key,
          sideEffects: true,
          meta: {
            vizeStyle: {
              scope: style.scoped ? scope : undefined,
              variables: scope,
              productionFilename: isProd ? filename : undefined,
            },
          },
          ...(loc
            ? { map: blockMap(source, loc.start, loc.end, asset.filePath, options.projectRoot) }
            : {}),
        });
        styleImports.push(`import ${JSON.stringify(key)};`);
      }
    }

    let code = result.code;
    let map = nativeMap(result.map, options.projectRoot);
    const assets = config.transformAssetUrls === false ? [] : collectSfcTemplateAssetUrls(source);
    if (assets.length) {
      const rewritten = rewriteSfcTemplateAssetReferences(code, assets);
      map = editMap(code, rewritten, map, asset.filePath, options.projectRoot);
      code = rewritten;
      const imports = assets
        .map(({ url, varName }) => `import ${varName} from ${JSON.stringify(`url:${url}`)};\n`)
        .join("");
      if (map) map.offsetLines(1, imports.split("\n").length - 1);
      code = imports + code;
    }
    if (needsHmr) {
      const instrumented = await instrumentHmr(
        code,
        result.moduleShape,
        runtimeKey,
        JSON.stringify({ scope, scoped: result.hasScoped, compiler: config.compiler }),
      );
      map = editMap(code, instrumented, map, asset.filePath, options.projectRoot);
      code = instrumented;
      children.push({ type: "js", content: hmrRuntime, uniqueKey: runtimeKey });
    }
    const inputMap = await asset.getMap();
    if (map && inputMap) map.extends(inputMap.toBuffer());
    children.unshift({
      type: "js",
      content: code,
      uniqueKey: componentKey,
      map,
      sideEffects: true,
    });

    // The glue module has no authored bindings, so its private names cannot collide
    // with user declarations. The native module and all its mappings stay separate.
    const shape = result.moduleShape;
    const component = shape?.hasDefaultExport
      ? "compiled.default"
      : shape?.hasNamedRenderExport
        ? "{ render: compiled.render }"
        : shape?.hasNamedSsrRenderExport
          ? "{ ssrRender: compiled.ssrRender }"
          : "{}";
    const hot = needsHmr
      ? `
if (module.hot) {
  _sfc_main.__hmrId = ${JSON.stringify(scope)};
  const api = globalThis.__VUE_HMR_RUNTIME__;
  const previous = module.hot.data && module.hot.data.vize;
  const current = { component: _sfc_main, metadata: readMetadata(${shape?.hasDefaultExport ? "_sfc_main" : "compiled.render"}) };
  module.hot.dispose(data => { data.vize = current; });
  // Sass invalidation may reevaluate every virtual block of the SFC. Reuse
  // only an identical component, or identical module bytes AND import values.
  const preserve = previous && (previous.component === _sfc_main || sameMetadata(previous.metadata, current.metadata));
  if (api && !api.createRecord(${JSON.stringify(scope)}, _sfc_main) && !preserve) api.reload(${JSON.stringify(scope)}, _sfc_main);
  module.hot.accept();
}
`
      : "";
    asset.type = "js";
    asset.setMap(null);
    asset.setCode(`${styleImports.join("\n")}
${needsHmr ? `import { read as readMetadata, same as sameMetadata } from ${JSON.stringify(runtimeKey)};` : ""}
import * as compiled from ${JSON.stringify(componentKey)};
export * from ${JSON.stringify(componentKey)};
const _sfc_main = ${component};
${result.hasScoped ? `_sfc_main.__scopeId = ${JSON.stringify(scope)};` : ""}
export default _sfc_main;
${hot}`);
    return [asset, ...children];
  },
});
