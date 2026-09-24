/**
 * In-process module hooks for the Vapor runtime conformance lanes.
 *
 * `.vue` imports are compiled on the fly with `@vizejs/native` for the
 * requested lane (VDOM SSR or Vapor), `.css` side-effect imports become
 * empty modules, and — for the Vapor lane — the bare `vue` specifier is
 * redirected to the Vue 3.6 runtime-with-vapor build so components, their
 * composables, and the fixtures all share one Vapor-capable runtime.
 */
import { readFileSync } from "node:fs";
import { createRequire, registerHooks } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

import { compileSfc } from "@vizejs/native";

/** Renderer lane compiled by {@link registerSfcHooks}. */
export type SfcLane = "ssr" | "vapor";

/** Compilation failures collected while modules load. */
export const compileFailures: string[] = [];

const require = createRequire(import.meta.url);

/** Absolute URL of the Vue 3.6 runtime-with-vapor browser build. */
export function vaporVueUrl(): string {
  return pathToFileURL(
    require.resolve("vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js"),
  ).href;
}

/** Install the `.vue` / `.css` (and, for Vapor, `vue`) module hooks. */
export function registerSfcHooks(lane: SfcLane): void {
  const vueUrl = lane === "vapor" ? vaporVueUrl() : undefined;
  registerHooks({
    resolve(specifier, context, nextResolve) {
      if (vueUrl !== undefined && specifier === "vue") return { url: vueUrl, shortCircuit: true };
      return nextResolve(specifier, context);
    },
    load(url, context, nextLoad) {
      if (url.endsWith(".css")) return { format: "module", source: "", shortCircuit: true };
      if (!url.endsWith(".vue")) return nextLoad(url, context);
      const filename = fileURLToPath(url);
      const result = compileSfc(readFileSync(filename, "utf8"), {
        filename,
        isTs: true,
        mode: "module",
        ssr: lane === "ssr",
        vapor: lane === "vapor",
      });
      if (result.errors.length > 0) {
        compileFailures.push(`${filename} [${lane}]\n${result.errors.join("\n")}`);
      }
      return {
        format: "module-typescript",
        source: result.code.replaceAll('"@vue/server-renderer"', '"vue/server-renderer"'),
        shortCircuit: true,
      };
    },
  });
}
