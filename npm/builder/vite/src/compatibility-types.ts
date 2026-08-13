export type VizeVueVersion = 0.11 | 1 | 2 | "2.7" | 3 | "legacy";

export interface VizeCompatibilityOptions {
  /**
   * Host Vue version. Vue 0.11/1/2/2.7 opt into host-compiler compatibility.
   */
  vueVersion?: VizeVueVersion;
  /**
   * Keep .vue files on the existing Vue compiler for legacy Vue runtimes.
   * @default true when vueVersion is 0.11, 1, 2, "2.7", or "legacy"
   */
  hostCompiler?: boolean;
  /**
   * Enable function-body output for CDN/global Vue evaluation.
   */
  scriptSetupInStandalone?: boolean;
  /**
   * Allow Vapor output for Options API SFCs when vapor is enabled.
   */
  optionsApiVapor?: boolean;
  /**
   * Override the host Nuxt major when this option object is shared with Nuxt.
   */
  nuxtVersion?: 2 | 3 | 4;
  /**
   * Override the host Webpack major when this option object is shared with unplugin.
   */
  webpackVersion?: 4 | 5;
}
