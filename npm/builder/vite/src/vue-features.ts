import type {
  VitePluginVueComponentIdGenerator,
  VitePluginVueCustomElementOption,
} from "./plugin-vue-types.ts";

/** Vue runtime feature flags shared with `@vitejs/plugin-vue`. */
export interface VizeVueFeatures {
  /**
   * Set to `false` to allow Vue's Options API code to be tree-shaken from
   * production bundles.
   * @default true
   */
  optionsAPI?: boolean;

  /**
   * Enable Vue devtools support in production bundles.
   * @default false
   */
  prodDevtools?: boolean;

  /**
   * Enable detailed hydration mismatch errors in production bundles.
   * @default false
   */
  prodHydrationMismatchDetails?: boolean;

  /** Custom-element matcher from `@vitejs/plugin-vue`'s `features` bag. */
  customElement?: VitePluginVueCustomElementOption;

  /** Vue 3.5 reactive props destructure feature flag. */
  propsDestructure?: boolean | "error";

  /** Scope-id strategy hook accepted for plugin-vue config compatibility. */
  componentIdGenerator?: VitePluginVueComponentIdGenerator;
}
