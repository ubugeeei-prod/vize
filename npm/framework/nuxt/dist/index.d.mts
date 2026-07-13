import { MuseaOptions, MuseaOptions as MuseaOptions$1 } from "@vizejs/vite-plugin-musea";

//#region src/compiler-options.d.ts
type VizeNuxtPattern = string | RegExp;
type VizeNuxtVueVersion = 0.11 | 1 | 2 | "2.7" | 3 | "legacy";
interface VizeNuxtCompilerCompatibilityOptions {
  vueVersion?: VizeNuxtVueVersion;
  hostCompiler?: boolean;
  scriptSetupInStandalone?: boolean;
  optionsApiVapor?: boolean;
  nuxtVersion?: 2 | 3 | 4;
  webpackVersion?: 4 | 5;
}
/**
 * Nuxt-facing mirror of the public `@vizejs/vite-plugin` options.
 *
 * Keeping this shape local lets the Nuxt module expose compiler configuration
 * without requiring the sibling package's generated declaration files to exist
 * during monorepo lint runs.
 */
interface VizeNuxtCompilerOptions {
  /**
   * Vue major version for the host project.
   *
   * Legacy Vue projects keep the host Vue compiler in charge. When set to
   * `0.11`, `1`, `2`, `"2.7"`, or `"legacy"`, the underlying Vite plugin runs in
   * compatibility mode and does not intercept `.vue` files.
   */
  vueVersion?: VizeNuxtVueVersion;
  /**
   * Opt-in compatibility features shared with `@vizejs/vite-plugin`.
   */
  compatibility?: VizeNuxtCompilerCompatibilityOptions;
  /** Emit function-body output for CDN/global Vue evaluation. */
  mode?: "module" | "function";
  /** Module name for runtime imports. */
  runtimeModuleName?: string;
  /** Global variable name for standalone/function mode. */
  runtimeGlobalName?: string;
  /** Override the public base used for dev-time asset URLs. */
  devUrlBase?: string;
  /** Files to include in compilation. */
  include?: VizeNuxtPattern | VizeNuxtPattern[];
  /** Files to exclude from compilation. */
  exclude?: VizeNuxtPattern | VizeNuxtPattern[];
  /** Force production mode. */
  isProduction?: boolean;
  /** Enable SSR mode. */
  ssr?: boolean;
  /** Enable source map generation. */
  sourceMap?: boolean;
  /** Enable Vapor mode compilation. */
  vapor?: boolean;
  /**
   * Default output mode for `.jsx`/`.tsx` components without a `"use vue:*"`
   * directive (forwarded to the underlying Vize plugin). @default "vdom"
   */
  jsxMode?: "vdom" | "vapor";
  /** Treat lowercase non-HTML tags as custom renderer elements. */
  customRenderer?: boolean;
  /** Template syntax compatibility mode. */
  templateSyntax?: "standard" | "strict" | "quirks";
  /** Root directory to scan for .vue files. */
  root?: string;
  /** Glob patterns to scan for .vue files during pre-compilation. */
  scanPatterns?: string[];
  /** Maximum number of Vue files to compile in a single native batch. */
  precompileBatchSize?: number;
  /** Glob patterns to ignore during pre-compilation. */
  ignorePatterns?: string[];
  /** Config file search mode. */
  configMode?: "root" | "auto" | false;
  /** Custom config file path. */
  configFile?: string;
  /** Handle .vue files in node_modules during on-demand compilation. */
  handleNodeModulesVue?: boolean;
  /** Enable debug logging. */
  debug?: boolean;
}
//#endregion
//#region src/options.d.ts
type VizeNuxtMajorVersion = 2 | 3 | 4;
interface VizeNuxtCompatibilityOptions {
  /**
   * Override the detected Nuxt major version.
   *
   * This exists for projects with unusual module wrappers. Most projects should
   * leave it on automatic detection.
   */
  nuxtVersion?: VizeNuxtMajorVersion;
  /**
   * Override the detected Vue major version.
   *
   * Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
   * Vue 0.11, Vue 1, and Vue 2 all use host-compiler compatibility mode.
   */
  vueVersion?: VizeNuxtVueVersion;
  /**
   * Keep legacy Vue projects on the host Vue compiler while still allowing the
   * Vize Nuxt module to provide bridges, linting, type checking, and Musea.
   *
   * @default true for Vue 0.11, Vue 1, Vue 2, and Nuxt 2
   */
  hostCompiler?: boolean;
  /**
   * Allow registering the Vite compiler bridge even when Nuxt's builder
   * detection cannot prove Vite support.
   *
   * @default false
   */
  forceViteCompiler?: boolean;
  /**
   * Enable function-body output for CDN/global Vue evaluation.
   */
  scriptSetupInStandalone?: boolean;
  /**
   * Allow Vapor output for Options API SFCs when the compiler is active.
   */
  optionsApiVapor?: boolean;
  /**
   * Preserve shared compatibility objects that also configure
   * `@vizejs/unplugin/webpack`.
   */
  webpackVersion?: 4 | 5;
}
interface VizeNuxtBridgeOptions {
  /**
   * Re-apply Nuxt auto-import injection to Vize virtual Vue modules.
   * @default true
   */
  autoImports?: boolean;
  /**
   * Re-apply Nuxt component auto-import resolution to Vize virtual Vue modules.
   * @default true
   */
  components?: boolean;
  /**
   * Re-apply @nuxtjs/i18n helper injection to Vize virtual Vue modules.
   * @default true
   */
  i18n?: boolean;
  /**
   * Stabilize Nuxt generated async-data keys between client and SSR transforms.
   * @default true
   */
  stableInjectedKeys?: boolean;
}
interface VizeNuxtUnoCssOptions {
  /**
   * Feed the original .vue source to UnoCSS extraction-only plugins so
   * attributify syntax survives Vize's render-function output.
   *
   * Set to `false` to skip reading SFC source files. Use an object to tune the
   * maximum source size read into Node.
   *
   * @default true
   */
  originalSource?:
    | boolean
    | {
        /**
         * Maximum original .vue source size to append for UnoCSS extraction.
         * @default 2097152
         */
        maxBytes?: number;
      };
}
interface VizeNuxtDevOptions {
  /**
   * Remove broken duplicate/unsafe stylesheet links from Nuxt dev SSR HTML
   * when Vize is the Vue compiler.
   *
   * @default true
   */
  stylesheetLinks?: boolean;
}
interface NuxtMuseaOptions {
  route?: {
    path?: string;
    name?: string;
    params?: Record<string, string>;
    query?: Record<string, string>;
    hash?: string;
    fullPath?: string;
    meta?: Record<string, unknown>;
  };
  runtimeConfig?: {
    public?: Record<string, unknown>;
    [key: string]: unknown;
  };
  fetchMocks?: Record<string, unknown>;
  stateMocks?: Record<string, unknown>;
}
interface VizeNuxtOptions {
  /**
   * Host framework compatibility overrides.
   *
   * These are usually inferred from Nuxt itself. Set `vueVersion: 0.11`, `1`,
   * `2`, or `"legacy"` for setups that must keep the host compiler.
   */
  compatibility?: VizeNuxtCompatibilityOptions;
  /**
   * Enable/disable the Vize compiler (Vue SFC compilation via Vite plugin).
   * Pass an object to configure the underlying `@vizejs/vite-plugin`.
   *
   * @default true
   */
  compiler?: boolean | VizeNuxtCompilerOptions;
  /**
   * Nuxt compatibility bridges for transforms that normally skip Rollup
   * virtual module ids.
   *
   * @default true
   */
  bridge?: boolean | VizeNuxtBridgeOptions;
  /**
   * UnoCSS bridge options for Vize virtual Vue modules.
   *
   * @default true
   */
  unocss?: boolean | VizeNuxtUnoCssOptions;
  /**
   * Dev-server integration options.
   */
  dev?: VizeNuxtDevOptions;
  /**
   * Musea gallery options.
   * Set to `true` to enable Musea with default options.
   *
   * @default false
   */
  musea?: boolean | MuseaOptions$1;
  /**
   * Nuxt mock options for musea gallery.
   * NOTE: In Nuxt context, nuxtMusea mocks are NOT added as a global Vite plugin
   * because they would intercept `#imports` resolution and break Nuxt's internals.
   * Real Nuxt composables are available via Nuxt's own plugin pipeline.
   */
  nuxtMusea?: NuxtMuseaOptions;
}
//#endregion
//#region src/schema.d.ts
declare module "nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }
  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }
}
declare module "@nuxt/schema" {
  interface NuxtConfig {
    vize?: Partial<VizeNuxtOptions>;
  }
  interface NuxtOptions {
    vize?: Partial<VizeNuxtOptions>;
  }
}
//#endregion
//#region src/index.d.ts
type NuxtWithBuilderOptions = {
  _version?: string;
  version?: string;
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
  options: {
    app?: {
      baseURL?: string;
      buildAssetsDir?: string;
    };
    builder?: string;
    build?: {
      publicPath?: string;
    };
    buildDir: string;
    dev?: boolean;
    modules: unknown[];
    rootDir: string;
    router?: {
      base?: string;
    };
    vite?: {
      plugins?: unknown[];
      resolve?: {
        dedupe?: string[];
      };
    };
    nitro?: {
      virtual?: Record<string, string>;
      publicAssets?: unknown[];
    };
    vize?: Partial<VizeNuxtOptions>;
    _requiredModules?: Record<string, boolean>;
    _nuxtVersion?: string;
    [key: string]: unknown;
  };
};
type VizeNuxtModuleContext = {
  nuxt?: NuxtWithBuilderOptions;
};
declare const vizeNuxtModule: ((
  this: VizeNuxtModuleContext | void,
  inlineOptions?: Partial<VizeNuxtOptions>,
  nuxtArg?: NuxtWithBuilderOptions,
) => Promise<void>) & {
  getMeta: () => {
    readonly name: "@vizejs/nuxt";
    readonly configKey: "vize";
  };
  getOptions: (
    inlineOptions?: Partial<VizeNuxtOptions>,
    nuxt?: NuxtWithBuilderOptions,
  ) => VizeNuxtOptions;
  meta: {
    readonly name: "@vizejs/nuxt";
    readonly configKey: "vize";
  };
  defaults: {
    musea: false;
    nuxtMusea: {
      route: {
        path: string;
      };
    };
  };
};
//#endregion
export {
  type MuseaOptions,
  type NuxtMuseaOptions,
  type VizeNuxtBridgeOptions,
  type VizeNuxtCompatibilityOptions,
  type VizeNuxtCompilerCompatibilityOptions,
  type VizeNuxtCompilerOptions,
  type VizeNuxtDevOptions,
  type VizeNuxtMajorVersion,
  type VizeNuxtOptions,
  type VizeNuxtUnoCssOptions,
  type VizeNuxtVueVersion,
  vizeNuxtModule as default,
};
