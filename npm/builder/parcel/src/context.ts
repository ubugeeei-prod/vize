import path from "node:path";
import type { MutableAsset, PluginOptions, PluginTracer } from "@parcel/types";
import { generateSfcScopeId } from "@vizejs/native";
import { nativeOperation } from "./trace.ts";

export function compilationContext(
  asset: MutableAsset,
  options: PluginOptions,
  source: string,
  tracer: PluginTracer,
) {
  const isProd = options.mode === "production";
  const ssr = asset.env.isNode();
  return {
    isProd,
    ssr,
    needsHmr: !ssr && !isProd && Boolean(options.hmrOptions),
    filename: path.relative(options.projectRoot, asset.filePath).replaceAll("\\", "/"),
    scope: `data-v-${nativeOperation(tracer, "generateSfcScopeId", asset.filePath, () => generateSfcScopeId(asset.filePath, options.projectRoot, isProd, source))}`,
  };
}
