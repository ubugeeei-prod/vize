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
}
