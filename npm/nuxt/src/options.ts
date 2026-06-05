import type { MuseaOptions } from "@vizejs/vite-plugin-musea";
import type { NuxtMuseaOptions } from "@vizejs/musea-nuxt";
import type {
  VizeNuxtCompilerCompatibilityOptions,
  VizeNuxtCompilerOptions,
  VizeNuxtVueVersion,
} from "./compiler-options.ts";
import { NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE, buildNuxtCompilerOptions } from "./utils.ts";

export type {
  VizeNuxtCompilerCompatibilityOptions,
  VizeNuxtCompilerOptions,
  VizeNuxtVueVersion,
} from "./compiler-options.ts";
export { NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE };

export type VizeNuxtMajorVersion = 2 | 3 | 4;

export interface VizeNuxtCompatibilityOptions {
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

export interface VizeNuxtBridgeOptions {
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

export interface VizeNuxtUnoCssOptions {
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

export interface VizeNuxtDevOptions {
  /**
   * Remove broken duplicate/unsafe stylesheet links from Nuxt dev SSR HTML
   * when Vize is the Vue compiler.
   *
   * @default true
   */
  stylesheetLinks?: boolean;
}

export interface VizeNuxtOptions {
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
  musea?: boolean | MuseaOptions;

  /**
   * Nuxt mock options for musea gallery.
   * NOTE: In Nuxt context, nuxtMusea mocks are NOT added as a global Vite plugin
   * because they would intercept `#imports` resolution and break Nuxt's internals.
   * Real Nuxt composables are available via Nuxt's own plugin pipeline.
   */
  nuxtMusea?: NuxtMuseaOptions;
}

export interface ResolvedVizeNuxtUnoCssOptions {
  originalSource: false | { maxBytes?: number };
}

export const DEFAULT_NUXT_BRIDGE_OPTIONS: Required<VizeNuxtBridgeOptions> = {
  autoImports: true,
  components: true,
  i18n: true,
  stableInjectedKeys: true,
};

export const DEFAULT_NUXT_UNOCSS_OPTIONS: ResolvedVizeNuxtUnoCssOptions = {
  originalSource: {},
};

export const DEFAULT_NUXT_DEV_OPTIONS: Required<VizeNuxtDevOptions> = {
  stylesheetLinks: true,
};

function isLegacyVueVersion(version: VizeNuxtVueVersion | undefined): boolean {
  return version === 0.11 || version === 1 || version === 2 || version === "legacy";
}

function normalizeNuxtCompilerCompatibilityOptions(
  compatibility: VizeNuxtCompatibilityOptions,
): VizeNuxtCompilerCompatibilityOptions {
  const normalized: VizeNuxtCompilerCompatibilityOptions = {};
  const legacyHost =
    isLegacyVueVersion(compatibility.vueVersion) || compatibility.nuxtVersion === 2;

  if (compatibility.vueVersion !== undefined) {
    normalized.vueVersion = compatibility.vueVersion;
  }
  if (compatibility.hostCompiler !== undefined || legacyHost) {
    normalized.hostCompiler = compatibility.hostCompiler ?? true;
  }
  if (compatibility.scriptSetupInStandalone !== undefined) {
    normalized.scriptSetupInStandalone = compatibility.scriptSetupInStandalone;
  }
  if (compatibility.optionsApiVapor !== undefined) {
    normalized.optionsApiVapor = compatibility.optionsApiVapor;
  }
  if (compatibility.nuxtVersion !== undefined) {
    normalized.nuxtVersion = compatibility.nuxtVersion;
  }
  if (compatibility.webpackVersion !== undefined) {
    normalized.webpackVersion = compatibility.webpackVersion;
  }

  return normalized;
}

export function resolveNuxtCompilerOptions(
  rootDir: string,
  baseURL: string | undefined,
  buildAssetsDir: string | undefined,
  compiler: VizeNuxtOptions["compiler"],
  compatibility: VizeNuxtCompatibilityOptions & { supportsViteCompiler?: boolean } = {},
): VizeNuxtCompilerOptions | false {
  if (compiler === false) {
    return false;
  }

  if (compatibility.supportsViteCompiler === false && compatibility.forceViteCompiler !== true) {
    return false;
  }

  const compatibilityOptions = normalizeNuxtCompilerCompatibilityOptions(compatibility);
  const hasCompatibilityOptions = Object.keys(compatibilityOptions).length > 0;
  const overrides = typeof compiler === "object" && compiler != null ? compiler : {};
  return buildNuxtCompilerOptions(rootDir, baseURL, buildAssetsDir, {
    vueVersion: compatibility.vueVersion,
    ...(hasCompatibilityOptions ? { compatibility: compatibilityOptions } : {}),
    mode: compatibility.scriptSetupInStandalone === true ? "function" : undefined,
    ...overrides,
  });
}

export function resolveNuxtBridgeOptions(
  bridge: VizeNuxtOptions["bridge"],
): Required<VizeNuxtBridgeOptions> {
  if (bridge === false) {
    return {
      autoImports: false,
      components: false,
      i18n: false,
      stableInjectedKeys: false,
    };
  }

  if (bridge === true || bridge == null) {
    return { ...DEFAULT_NUXT_BRIDGE_OPTIONS };
  }

  return {
    autoImports: bridge.autoImports ?? DEFAULT_NUXT_BRIDGE_OPTIONS.autoImports,
    components: bridge.components ?? DEFAULT_NUXT_BRIDGE_OPTIONS.components,
    i18n: bridge.i18n ?? DEFAULT_NUXT_BRIDGE_OPTIONS.i18n,
    stableInjectedKeys: bridge.stableInjectedKeys ?? DEFAULT_NUXT_BRIDGE_OPTIONS.stableInjectedKeys,
  };
}

export function resolveNuxtUnoCssOptions(
  unocss: VizeNuxtOptions["unocss"],
): ResolvedVizeNuxtUnoCssOptions | false {
  if (unocss === false) {
    return false;
  }

  if (unocss === true || unocss == null) {
    return { ...DEFAULT_NUXT_UNOCSS_OPTIONS };
  }

  const originalSource = unocss.originalSource;
  if (originalSource === false) {
    return { originalSource: false };
  }

  if (originalSource === true || originalSource == null) {
    return { originalSource: {} };
  }

  return { originalSource };
}

export function resolveNuxtDevOptions(dev: VizeNuxtOptions["dev"]): Required<VizeNuxtDevOptions> {
  return {
    ...DEFAULT_NUXT_DEV_OPTIONS,
    ...dev,
  };
}

export function resolveNuxtMuseaOptions(musea: VizeNuxtOptions["musea"]): MuseaOptions | false {
  if (musea === true) {
    return {};
  }
  if (musea === false || musea == null) {
    return false;
  }
  return musea;
}
