/** Lint wiring for the Nuxt module: generated config plus the dev checker. */
import type { VizeNuxtOptions } from "../options.ts";
import { setupNuxtLintChecker } from "./checker/setup.ts";
import { setupNuxtLintConfigGeneration, type NuxtLintConfigGeneration } from "./generation.ts";

type NuxtLintModuleNuxt = Parameters<typeof setupNuxtLintConfigGeneration>[1] &
  Parameters<typeof setupNuxtLintChecker>[1] & { options: { vize?: Partial<VizeNuxtOptions> } };

/** Whether user config asks for lint, as opposed to inheriting the module default. */
function isLintRequested(...configured: Array<Partial<VizeNuxtOptions> | undefined>): boolean {
  return configured.some(
    (options) =>
      (options?.lint !== undefined && options.lint !== false) ||
      (options?.checker !== undefined && options.checker !== false),
  );
}

export async function setupNuxtLint(
  options: Pick<VizeNuxtOptions, "checker" | "lint">,
  nuxt: NuxtLintModuleNuxt,
  inlineOptions: Partial<VizeNuxtOptions> | undefined,
): Promise<NuxtLintConfigGeneration | undefined> {
  const generation = await setupNuxtLintConfigGeneration(options.lint, nuxt, {
    reportMissingPlugin: isLintRequested(inlineOptions, nuxt.options.vize),
  });
  // Without `oxlint-plugin-vize`, generation has already skipped lint.
  if (generation || options.lint === false) {
    await setupNuxtLintChecker(options.checker, nuxt, generation);
  }
  return generation;
}
