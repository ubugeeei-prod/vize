// What the host bundler offers the plugin: its CSS pipeline and its resolver.
import path from "node:path";
import { parseSfc } from "@vizejs/native";
import {
  listSfcSrcRequests,
  resolveSfcSrcRequests,
  type HostResolution,
  type HostResolve,
  type SfcSrcResolution,
} from "../../shared/sfc-src.ts";
import { isVueStyleRequest } from "./request.ts";
import type { HostCss } from "./types.ts";

/**
 * What the host does with an imported stylesheet. esbuild without `bundle`
 * never follows an import, and its CSS loaders compile no preprocessor.
 */
export function hostCss(context: unknown, framework: string): HostCss {
  if (framework === "bun") {
    return "css";
  }
  if (framework !== "esbuild") {
    return "preprocessors";
  }
  const native = (
    context as { getNativeBuildContext?: () => unknown }
  ).getNativeBuildContext?.() as { build?: { initialOptions?: { bundle?: boolean } } } | undefined;
  return native?.build?.initialOptions?.bundle === true ? "css" : "none";
}

export const CSS_PIPELINE_HINT =
  "[vize] <style> blocks are stylesheet modules for the bundler's CSS pipeline, and this build did not bundle one. " +
  "tsdown needs @tsdown/css; Rollup needs a CSS plugin such as rollup-plugin-postcss; " +
  "Rolldown alone does not bundle CSS, so build through tsdown or Vite.";

export interface HostBuildError {
  id?: unknown;
  code?: unknown;
  errors?: HostBuildError[];
}

/** A Rollup or Rolldown build error raised on a style module with no CSS pipeline. */
export function isStylePipelineError(error: HostBuildError): boolean {
  return [error, ...(error.errors ?? [])].some(
    (item) =>
      (typeof item.id === "string" && isVueStyleRequest(item.id)) ||
      // Rolldown names no module for a CSS module it cannot bundle.
      item.code === "UNSUPPORTED_FEATURE",
  );
}

type RollupLikeResolve = (
  source: string,
  importer: string,
  options: { skipSelf: boolean },
) => Promise<HostResolution>;
type LoaderResolve = (context: string, request: string) => Promise<string>;
type EsbuildResolve = (
  path: string,
  options: { kind: "import-statement"; resolveDir: string; importer: string },
) => Promise<{ path: string; external: boolean; errors: unknown[] }>;

/** The bundler's own module resolver, or null for a host that offers none. */
function hostResolver(context: unknown): HostResolve | null {
  const ctx = context as { resolve?: RollupLikeResolve; getNativeBuildContext?: () => unknown };
  if (typeof ctx.resolve === "function") {
    const resolve = ctx.resolve;
    return (request, importer) => resolve.call(context, request, importer, { skipSelf: true });
  }
  const native = ctx.getNativeBuildContext?.() as
    | { framework: string; loaderContext?: { getResolve: () => LoaderResolve } }
    | { framework: "esbuild"; build: { resolve: EsbuildResolve } }
    | undefined;
  if (native && "loaderContext" in native && native.loaderContext) {
    const resolve = native.loaderContext.getResolve();
    return (request, importer) =>
      resolve(path.dirname(importer), request).then(
        (id) => ({ id }),
        () => null,
      );
  }
  if (native && "build" in native) {
    const { build } = native;
    return async (request, importer) => {
      const result = await build.resolve(request, {
        kind: "import-statement",
        resolveDir: path.dirname(importer),
        importer,
      });
      return result.errors.length > 0 ? null : { id: result.path, external: result.external };
    };
  }
  return null;
}

/** Resolve the SFC's `src` module requests (packages, aliases) with the bundler. */
export async function resolveSrcRequests(
  context: unknown,
  importer: string,
  source: string,
): Promise<Map<string, SfcSrcResolution> | undefined> {
  const requests = listSfcSrcRequests(parseSfc, importer, source);
  const resolve = requests.length > 0 ? hostResolver(context) : null;
  return resolve ? resolveSfcSrcRequests(importer, requests, resolve) : undefined;
}
