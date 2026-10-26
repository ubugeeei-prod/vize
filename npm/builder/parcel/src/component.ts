import type {
  MutableAsset,
  PluginOptions,
  PluginLogger,
  PluginTracer,
  TransformerResult,
} from "@parcel/types";
import {
  collectSfcTemplateAssetUrls,
  compileSfc,
  extractSfcSrcInfo,
  rewriteSfcTemplateAssetReferences,
} from "@vizejs/native";
import { fail, type VizeParcelOptions } from "./config.ts";
import { compilationContext } from "./context.ts";
import { editMap, nativeMap } from "./maps.ts";
import { nativeOperation } from "./trace.ts";

/** Native code and its stable HMR asset belong to a request without CSS dependencies. */
export async function compileComponent({
  asset,
  config,
  options,
  logger,
  tracer,
}: {
  asset: MutableAsset;
  config: VizeParcelOptions;
  options: PluginOptions;
  logger: PluginLogger;
  tracer: PluginTracer;
}): Promise<TransformerResult[]> {
  const source = await asset.getCode();
  const src = nativeOperation(tracer, "extractSfcSrcInfo", asset.filePath, () =>
    extractSfcSrcInfo(source, asset.filePath),
  );
  if (src.scriptSrc || src.templateSrc)
    fail(
      "External script/template blocks are not supported by the Parcel adapter yet. Inline the block or import its module explicitly.",
      asset.filePath,
    );
  const { filename, scope, isProd, ssr, needsHmr } = compilationContext(
    asset,
    options,
    source,
    tracer,
  );
  const result = nativeOperation(tracer, "compileSfc", asset.filePath, () =>
    compileSfc(source, {
      ...config.compiler,
      filename,
      mode: "module",
      sourceMap: Boolean(asset.env.sourceMap),
      ssr,
      isProd,
      isTs: false,
      scopeId: scope,
    }),
  );
  if (result.errors.length) fail(result.errors.join("\n"), asset.filePath);
  if (result.customBlocks.length || result.macroArtifacts.length)
    fail(
      "Custom blocks and emitted macro artifacts require an explicit Parcel adapter and are not supported yet.",
      asset.filePath,
    );
  if (result.styles.some((style) => style.module))
    fail(
      "SFC CSS modules are not supported by the Parcel adapter yet. Use a separate .module.css import.",
      asset.filePath,
    );
  for (const message of result.warnings) logger.warn({ message });
  let code = result.code;
  let map = nativeMap(result.map, options.projectRoot);
  const assets =
    config.transformAssetUrls === false
      ? []
      : nativeOperation(tracer, "collectSfcTemplateAssetUrls", asset.filePath, () =>
          collectSfcTemplateAssetUrls(source),
        );
  if (assets.length) {
    const rewritten = nativeOperation(
      tracer,
      "rewriteSfcTemplateAssetReferences",
      asset.filePath,
      () => rewriteSfcTemplateAssetReferences(code, assets),
    );
    map = editMap(code, rewritten, map, asset.filePath, options.projectRoot);
    code = rewritten;
    const imports = assets
      .map(({ url, varName }) => `import ${varName} from ${JSON.stringify(`url:${url}`)};\n`)
      .join("");
    if (map) map.offsetLines(1, imports.split("\n").length - 1);
    code = imports + code;
  }
  const inputMap = await asset.getMap();
  if (map && inputMap) map.extends(inputMap.toBuffer());
  const componentKey = `${asset.id}-vize-component`;
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
  if (api && !api.createRecord(${JSON.stringify(scope)}, _sfc_main)) api.reload(${JSON.stringify(scope)}, _sfc_main);
  module.hot.accept();
}
`
    : "";
  return [
    {
      type: "js",
      uniqueKey: `${asset.id}-vize-glue`,
      sideEffects: true,
      content: `import * as compiled from ${JSON.stringify(componentKey)};
export * from ${JSON.stringify(componentKey)};
const _sfc_main = ${component};
${result.hasScoped ? `_sfc_main.__scopeId = ${JSON.stringify(scope)};` : ""}
export default _sfc_main;
${hot}`,
    },
    { type: "js", content: code, uniqueKey: componentKey, map, sideEffects: true },
  ];
}
