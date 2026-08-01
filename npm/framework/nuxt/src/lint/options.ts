import type { NuxtLintFeatures } from "@vizejs/nuxt-lint-config";
export type { VizeNuxtLintCheckerOptions } from "./checker/options.ts";

export interface VizeNuxtLintDevtoolsOptions {
  /** Enable the inspector eagerly, lazily from its tab, or not at all. @default "lazy" */
  enabled?: boolean | "lazy";
  /** Localhost port for the in-process inspector UI. An available port is chosen by default. */
  port?: number;
}

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

  /** Expose the resolved plan in a Nuxt DevTools tab. @default { enabled: "lazy" } */
  devtools?: VizeNuxtLintDevtoolsOptions;
}
