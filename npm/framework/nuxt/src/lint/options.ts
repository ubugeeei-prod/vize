import type { NuxtLintFeatures } from "@vizejs/nuxt-lint-config";
export type { VizeNuxtLintCheckerOptions } from "./checker/options.ts";

/** Options for the generated Nuxt-aware oxlint configuration. */
export interface VizeNuxtLintOptions extends NuxtLintFeatures {
  /**
   * File path for the generated oxlint config.
   *
   * Relative paths are resolved from the Nuxt project root.
   * @default ".nuxt/oxlint.config.json"
   */
  configFile?: string;

  /**
   * Create a root `oxlint.config.mts` loading the generated config when no
   * supported oxlint config exists in the project or an ancestor.
   * @default true
   */
  autoInit?: boolean;

  /**
   * Override the directory the generated Nuxt globs are relative to.
   * @default nuxt.options.rootDir
   */
  rootDir?: string;
}
