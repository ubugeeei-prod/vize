// What the host bundler offers the plugin: its CSS pipeline and its resolver.
import path from "node:path";
import { parseSfc } from "@vizejs/native";
import { listSfcSrcRequests, resolveSfcSrcRequests } from "../../shared/sfc-src.ts";
import { isVueStyleRequest } from "./request.ts";
import type { HostCss } from "./types.ts";

/**
 * What the host does with an imported stylesheet. esbuild without `bundle`
 * never follows an import, and its CSS loaders compile no preprocessor.
 */
export function hostCss(context: unknown, framework: string): HostCss {
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

type HostResolve = (request: string, importer: string) => Promise<string | null | undefined>;
type RollupLikeResolve = (
  source: string,
  importer: string,
  options: { skipSelf: boolean },
) => Promise<{ id: string; external?: boolean | string } | null | undefined>;
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
    return async (request, importer) => {
      const result = await resolve.call(context, request, importer, { skipSelf: true });
      // An external module is not bundled, so there is no file to inline.
      return result && !result.external ? result.id : null;
    };
  }
  const native = ctx.getNativeBuildContext?.() as
    | { framework: string; loaderContext?: { getResolve: () => LoaderResolve } }
    | { framework: "esbuild"; build: { resolve: EsbuildResolve } }
    | undefined;
  if (native && "loaderContext" in native && native.loaderContext) {
    const resolve = native.loaderContext.getResolve();
    return (request, importer) => resolve(path.dirname(importer), request).catch(() => null);
  }
  if (native && "build" in native) {
    const { build } = native;
    return async (request, importer) => {
      const result = await build.resolve(request, {
        kind: "import-statement",
        resolveDir: path.dirname(importer),
        importer,
      });
      return result.errors.length > 0 || result.external ? null : result.path;
    };
  }
  return null;
}

/** Resolve the SFC's `src` module requests (packages, aliases) with the bundler. */
export async function resolveSrcRequests(
  context: unknown,
  importer: string,
  source: string,
): Promise<Map<string, string> | undefined> {
  const requests = listSfcSrcRequests(parseSfc, importer, source);
  const resolve = requests.length > 0 ? hostResolver(context) : null;
  return resolve ? resolveSfcSrcRequests(importer, requests, resolve) : undefined;
}
