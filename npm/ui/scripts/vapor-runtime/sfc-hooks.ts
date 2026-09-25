/**
 * In-process module hooks for the Vapor runtime conformance lanes.
 *
 * `.vue` imports are compiled on the fly with `@vizejs/native` for the
 * requested lane (VDOM SSR or Vapor), `.css` side-effect imports become
 * empty modules, and — for the Vapor lane — the bare `vue` specifier is
 * redirected to the Vue 3.6 runtime-with-vapor build so components, their
 * composables, and the fixtures all share one Vapor-capable runtime.
 */
import { createHash } from "node:crypto";
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

/**
 * Compile one SFC for the Vapor lane with Vue's own `@vue/compiler-sfc` 3.6.
 *
 * Selected with `VIZE_VAPOR_COMPILER=vue`, this turns the lane into an oracle:
 * a fixture that passes with Vue's compiler but fails with Vize's points at a
 * Vize codegen difference, while one that fails with both is a runtime gap.
 */
function compileWithVue(source: string, filename: string): string {
  const compiler = require("vue-vapor-runtime/compiler-sfc") as typeof import("vue/compiler-sfc");
  const { descriptor, errors } = compiler.parse(source, { filename });
  if (errors.length > 0) compileFailures.push(`${filename} [vue]\n${errors.join("\n")}`);
  const scoped = descriptor.styles.some((style) => style.scoped);
  const id = createHash("sha256").update(filename).digest("hex").slice(0, 8);
  const script = compiler.compileScript(descriptor, {
    id,
    inlineTemplate: true,
    isProd: false,
    templateOptions: { scoped },
    vapor: true,
  } as Parameters<typeof compiler.compileScript>[1]);
  return script.content;
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
      if (lane === "vapor" && process.env.VIZE_VAPOR_COMPILER === "vue") {
        return {
          format: "module-typescript",
          source: compileWithVue(readFileSync(filename, "utf8"), filename),
          shortCircuit: true,
        };
      }
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
