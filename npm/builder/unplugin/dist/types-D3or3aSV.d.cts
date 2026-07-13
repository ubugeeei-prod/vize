//#region src/types.d.ts
interface MacroArtifact {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}
interface VizeUnpluginOptions {
  include?: string | RegExp | Array<string | RegExp>;
  exclude?: string | RegExp | Array<string | RegExp>;
  compatibility?: VizeCompatibilityOptions;
  isProduction?: boolean;
  ssr?: boolean;
  sourceMap?: boolean;
  mode?: "module" | "function";
  vapor?: boolean;
  experimentalInTagComments?: boolean;
  experimentalPatternedTemplate?: boolean;
  experimentalServerScript?: boolean;
  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive. Distinct from `vapor` (which targets `.vue` SFCs). A
   * per-component directive overrides it.
   * @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";
  customRenderer?: boolean;
  templateSyntax?: VizeTemplateSyntax;
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: VizeVueVersion;
  root?: string;
  debug?: boolean;
}
type VizeVueVersion = 0.11 | 1 | 2 | "2.7" | 3 | "legacy";
type VizeTemplateSyntax = "standard" | "strict" | "quirks";
interface VizeCompatibilityOptions {
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
   * Force Webpack compatibility behavior.
   *
   * Webpack 4 does not expose `compiler.webpack`, so the plugin resolves
   * `DefinePlugin` from the host `webpack` package when this is `4` or when
   * auto-detection sees a Webpack 4 compiler shape.
   */
  webpackVersion?: 4 | 5;
}
//#endregion
export {
  VizeVueVersion as i,
  VizeCompatibilityOptions as n,
  VizeUnpluginOptions as r,
  MacroArtifact as t,
};
