/**
 * Makes the SSR target of every multi-target native call fail for files named
 * `*Broken.vue`, as a `vue.plugins` directive without SSR support makes it
 * fail, while each environment's own compile still succeeds. A suite imports
 * this before anything else, so the plugin takes the failing function.
 */

import { createRequire } from "node:module";

type Result = { targets: { results: { path: string; code: string; errors: string[] }[] }[] };
type ForTargets = (files: unknown, options: unknown, targets: { ssr?: boolean }[]) => Result;

const native = createRequire(import.meta.url)("@vizejs/native") as {
  compileSfcBatchForTargets: ForTargets;
};
const compile = native.compileSfcBatchForTargets;

/** The message the failing SSR target reports. */
export const INJECTED_SSR_FAILURE = "the SSR output of this file is refused";

native.compileSfcBatchForTargets = function failing(files, options, targets) {
  const result = compile.call(this, files, options, targets);
  result.targets.forEach((target, index) => {
    if (targets[index]?.ssr !== true) return;
    for (const file of target.results) {
      if (!file.path.endsWith("Broken.vue")) continue;
      file.code = "";
      file.errors = [INJECTED_SSR_FAILURE];
    }
  });
  return result;
};
