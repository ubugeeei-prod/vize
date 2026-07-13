import * as _$vue from "vue";
import { App, MaybeRefOrGetter, PropType, Ref } from "vue";
import { Plugin } from "vite";

//#region src/types.d.ts
/**
 * NuxtMusea plugin options.
 */
type NuxtMuseaRouteQueryValue = string | string[] | null | undefined;
interface NuxtMuseaRouteLocation {
  path: string;
  name: string | null;
  params: Record<string, string | string[]>;
  query: Record<string, NuxtMuseaRouteQueryValue>;
  hash: string;
  fullPath: string;
  meta: Record<string, unknown>;
  matched: unknown[];
  redirectedFrom?: unknown;
}
type NuxtMuseaNavigationTarget =
  | string
  | {
      path?: string;
      name?: string;
      params?: Record<string, string | string[]>;
      query?: Record<string, NuxtMuseaRouteQueryValue>;
      hash?: string;
      replace?: boolean;
      [key: string]: unknown;
    };
interface NuxtMuseaOptions {
  /**
   * Mock route data.
   */
  route?: Partial<NuxtMuseaRouteLocation>;
  /**
   * Mock runtime config.
   */
  runtimeConfig?: {
    public?: Record<string, unknown>;
    [key: string]: unknown;
  };
  /**
   * Mock useFetch / useAsyncData default responses.
   * Key is the URL/key pattern, value is the mock response data.
   */
  fetchMocks?: Record<string, unknown>;
  /**
   * Mock useState initial values.
   * Key is the state key, value is the initial state.
   */
  stateMocks?: Record<string, unknown>;
  /**
   * Mock useCookie initial values.
   * Key is the cookie name, value is the initial cookie value.
   */
  cookieMocks?: Record<string, unknown>;
  /**
   * Mock app config exposed through useAppConfig().
   */
  appConfig?: Record<string, unknown>;
  /**
   * Request information exposed by server-oriented composables.
   */
  request?: {
    url?: string;
    headers?: Record<string, string>;
  };
}
//#endregion
//#region src/context.d.ts
declare function configureNuxtMuseaMocks(options?: NuxtMuseaOptions): void;
declare function resetNuxtMuseaMocks(): void;
declare function getRouteState(): NuxtMuseaRouteLocation;
declare function getRuntimeConfigState(): Record<string, unknown>;
declare function getAppConfigState(): Record<string, unknown>;
//#endregion
//#region src/app.d.ts
declare function installNuxtMuseaMocks(app: App, options?: NuxtMuseaOptions): App;
declare function createNuxtMuseaPreviewSetup(options?: NuxtMuseaOptions): (app: App) => void;
//#endregion
//#region src/mocks/composables.d.ts
/**
 * Mock useRoute - returns a reactive route object.
 */
declare function useRoute(): NuxtMuseaRouteLocation;
/**
 * Mock useRouter - returns a router-like object with no-op navigation.
 */
declare function useRouter(): {
  push: (to: NuxtMuseaNavigationTarget) => Promise<void>;
  replace: (to: NuxtMuseaNavigationTarget) => Promise<void>;
  back: () => void;
  forward: () => void;
  go: (_delta: number) => void;
  resolve: (to: NuxtMuseaNavigationTarget) => {
    href: string;
    route: NuxtMuseaRouteLocation;
  };
  currentRoute: _$vue.ComputedRef<NuxtMuseaRouteLocation>;
  addRoute: () => () => void;
  removeRoute: () => void;
  hasRoute: () => boolean;
  getRoutes: () => never[];
  beforeEach: () => () => void;
  afterEach: () => () => void;
  onError: () => () => void;
  isReady: () => Promise<void>;
  options: {};
};
//#endregion
//#region src/mocks/data.d.ts
interface AsyncDataOptions<T> {
  default?: () => T;
  lazy?: boolean;
  immediate?: boolean;
  transform?: (input: unknown) => T;
  pick?: string[];
}
interface AsyncDataResult<T> {
  data: Ref<T | null>;
  pending: Ref<boolean>;
  error: Ref<Error | null>;
  refresh: () => Promise<void>;
  execute: () => Promise<void>;
  clear: () => void;
  status: Ref<"idle" | "pending" | "success" | "error">;
}
/**
 * Mock useFetch - returns reactive data based on mock config.
 */
declare function useFetch<T = unknown>(
  url: MaybeRefOrGetter<string | URL>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T>;
/**
 * Mock useAsyncData - similar to useFetch but with key-based lookup.
 */
declare function useAsyncData<T = unknown>(
  key: string,
  handler?: () => T | Promise<T>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T>;
/**
 * Mock useLazyFetch - lazy variant of useFetch.
 */
declare function useLazyFetch<T = unknown>(
  url: MaybeRefOrGetter<string | URL>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T>;
/**
 * Mock useLazyAsyncData - lazy variant of useAsyncData.
 */
declare function useLazyAsyncData<T = unknown>(
  key: string,
  handler?: () => T | Promise<T>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T>;
declare function refreshNuxtData(_keys?: string | string[]): Promise<void>;
declare function clearNuxtData(_keys?: string | string[]): void;
declare function useNuxtData<T = unknown>(
  key: string,
): {
  data: Ref<T | null>;
};
declare function useRequestFetch(): <T = unknown>(url: string | URL) => Promise<T>;
//#endregion
//#region src/mocks/navigation.d.ts
/**
 * Mock navigateTo - updates the shared route state in gallery context.
 */
declare function navigateTo(
  to: NuxtMuseaNavigationTarget,
  _opts?: {
    replace?: boolean;
    redirectCode?: number;
    external?: boolean;
  },
): Promise<void>;
/**
 * Mock abortNavigation - no-op in gallery context.
 */
declare function abortNavigation(_err?: string | Error): void;
/**
 * Mock defineNuxtRouteMiddleware - returns the middleware function as-is.
 */
declare function defineNuxtRouteMiddleware(middleware: unknown): unknown;
/**
 * Mock definePageMeta - page metadata is handled by Nuxt at build time.
 */
declare function definePageMeta(_meta: Record<string, unknown>): void;
/**
 * Mock setPageLayout - no layout switching in isolated previews.
 */
declare function setPageLayout(_layout: string | false): void;
declare function prefetchComponents(_to: NuxtMuseaNavigationTarget): Promise<void>;
declare function preloadComponents(_to: NuxtMuseaNavigationTarget): Promise<void>;
declare function preloadRouteComponents(_to: NuxtMuseaNavigationTarget): Promise<void>;
//#endregion
//#region src/mocks/head.d.ts
/**
 * Mock Nuxt head management composables.
 * All are no-ops in the gallery context.
 */
interface HeadEntry {
  dispose: () => void;
  patch: (_input: Record<string, unknown>) => void;
  pause: () => void;
  resume: () => void;
}
/**
 * Mock useHead - no-op.
 */
declare function useHead(_input: Record<string, unknown>): HeadEntry;
/**
 * Mock useSeoMeta - no-op.
 */
declare function useSeoMeta(_input: Record<string, unknown>): HeadEntry;
/**
 * Mock useHeadSafe - no-op.
 */
declare function useHeadSafe(_input: Record<string, unknown>): HeadEntry;
/**
 * Mock useServerSeoMeta - no-op.
 */
declare function useServerSeoMeta(_input: Record<string, unknown>): HeadEntry;
//#endregion
//#region src/mocks/error.d.ts
interface NuxtError extends Error {
  data?: unknown;
  fatal?: boolean;
  statusCode?: number;
  statusMessage?: string;
}
declare function createError(input: string | Partial<NuxtError>): NuxtError;
declare function showError(input: string | Partial<NuxtError>): NuxtError;
declare function clearError(_options?: { redirect?: string }): Promise<void>;
declare function useError(): _$vue.Ref<Error | null, Error | null>;
//#endregion
//#region src/mocks/runtime.d.ts
/**
 * Mock useNuxtApp - returns a minimal Nuxt app-like object.
 */
declare function useNuxtApp(): {
  $config: Record<string, unknown>;
  $router: {
    push: (to: NuxtMuseaNavigationTarget) => Promise<void>;
    replace: (to: NuxtMuseaNavigationTarget) => Promise<void>;
    back: () => void;
    forward: () => void;
    go: (_delta: number) => void;
    resolve: (to: NuxtMuseaNavigationTarget) => {
      href: string;
      route: NuxtMuseaRouteLocation;
    };
    currentRoute: _$vue.ComputedRef<NuxtMuseaRouteLocation>;
    addRoute: () => () => void;
    removeRoute: () => void;
    hasRoute: () => boolean;
    getRoutes: () => never[];
    beforeEach: () => () => void;
    afterEach: () => () => void;
    onError: () => () => void;
    isReady: () => Promise<void>;
    options: {};
  };
  $route: NuxtMuseaRouteLocation;
  provide: (_name: string, _value: unknown) => void;
  hook: (_name: string, _fn: (...args: unknown[]) => void) => void;
  callHook: (_name: string, ..._args: unknown[]) => Promise<void>;
  vueApp: null;
  payload: {
    data: {};
    state: {};
  };
  isHydrating: boolean;
  runWithContext: <T>(fn: () => T) => T;
};
/**
 * Mock useRuntimeConfig - returns the configured runtime config.
 */
declare function useRuntimeConfig(): Record<string, unknown>;
declare function useAppConfig(): Record<string, unknown>;
declare function updateAppConfig(config: Record<string, unknown>): void;
/**
 * Mock useState - returns a ref initialized from mock config or init function.
 */
declare function useState<T = unknown>(key: string, init?: () => T): Ref<T | undefined>;
/**
 * Mock useRequestHeaders - returns empty headers in gallery context.
 */
declare function useRequestHeaders(_include?: string[]): Record<string, string>;
/**
 * Mock useRequestEvent - returns undefined in gallery context.
 */
declare function useRequestEvent(): undefined;
/**
 * Mock useRequestURL - returns current window location.
 */
declare function useRequestURL(): URL;
/**
 * Mock useCookie - returns a ref-like cookie mock.
 */
declare function useCookie<T = unknown>(
  name: string,
  opts?: {
    default?: () => T;
  },
): Ref<T | undefined>;
/**
 * Mock clearNuxtState - no-op.
 */
declare function clearNuxtState(keys?: string | string[]): void;
/**
 * Mock defineNuxtPlugin - returns the plugin function as-is.
 */
declare function defineNuxtPlugin(plugin: unknown): unknown;
declare function defineNuxtComponent<T>(component: T): T;
declare function onNuxtReady(callback: () => void): void;
declare function callOnce<T>(
  keyOrFn: string | (() => T | Promise<T>),
  fn?: () => T | Promise<T>,
): Promise<T | undefined>;
declare function useId(): string;
declare function reloadNuxtApp(_options?: Record<string, unknown>): Promise<void>;
//#endregion
//#region src/mocks/components.d.ts
/**
 * Mock NuxtLink - renders as <RouterLink> or <a>.
 */
declare const NuxtLink: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    to: {
      type: PropType<NuxtMuseaNavigationTarget>;
      default: string;
    };
    href: {
      type: StringConstructor;
      default: undefined;
    };
    target: {
      type: StringConstructor;
      default: undefined;
    };
    rel: {
      type: StringConstructor;
      default: undefined;
    };
    external: {
      type: BooleanConstructor;
      default: boolean;
    };
    replace: {
      type: BooleanConstructor;
      default: boolean;
    };
    prefetch: {
      type: BooleanConstructor;
      default: boolean;
    };
    noPrefetch: {
      type: BooleanConstructor;
      default: boolean;
    };
    activeClass: {
      type: StringConstructor;
      default: string;
    };
    exactActiveClass: {
      type: StringConstructor;
      default: string;
    };
    custom: {
      type: BooleanConstructor;
      default: boolean;
    };
  }>,
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | undefined,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      to: {
        type: PropType<NuxtMuseaNavigationTarget>;
        default: string;
      };
      href: {
        type: StringConstructor;
        default: undefined;
      };
      target: {
        type: StringConstructor;
        default: undefined;
      };
      rel: {
        type: StringConstructor;
        default: undefined;
      };
      external: {
        type: BooleanConstructor;
        default: boolean;
      };
      replace: {
        type: BooleanConstructor;
        default: boolean;
      };
      prefetch: {
        type: BooleanConstructor;
        default: boolean;
      };
      noPrefetch: {
        type: BooleanConstructor;
        default: boolean;
      };
      activeClass: {
        type: StringConstructor;
        default: string;
      };
      exactActiveClass: {
        type: StringConstructor;
        default: string;
      };
      custom: {
        type: BooleanConstructor;
        default: boolean;
      };
    }>
  > &
    Readonly<{}>,
  {
    to: NuxtMuseaNavigationTarget;
    href: string;
    target: string;
    rel: string;
    external: boolean;
    replace: boolean;
    prefetch: boolean;
    noPrefetch: boolean;
    activeClass: string;
    exactActiveClass: string;
    custom: boolean;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
/**
 * Mock NuxtPage - renders <RouterView> or slot content.
 */
declare const NuxtPage: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    name: {
      type: StringConstructor;
      default: string;
    };
    transition: {
      type: (ObjectConstructor | BooleanConstructor)[];
      default: undefined;
    };
    keepalive: {
      type: (ObjectConstructor | BooleanConstructor)[];
      default: undefined;
    };
    pageKey: {
      type: (StringConstructor | FunctionConstructor)[];
      default: undefined;
    };
  }>,
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[],
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      name: {
        type: StringConstructor;
        default: string;
      };
      transition: {
        type: (ObjectConstructor | BooleanConstructor)[];
        default: undefined;
      };
      keepalive: {
        type: (ObjectConstructor | BooleanConstructor)[];
        default: undefined;
      };
      pageKey: {
        type: (StringConstructor | FunctionConstructor)[];
        default: undefined;
      };
    }>
  > &
    Readonly<{}>,
  {
    name: string;
    transition: boolean | Record<string, any>;
    keepalive: boolean | Record<string, any>;
    pageKey: string | Function;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
/**
 * Mock ClientOnly - renders default slot on client side (always in browser context).
 */
declare const ClientOnly: _$vue.DefineComponent<
  {},
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | null,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
/**
 * Mock NuxtLayout - renders slot content with optional layout wrapper.
 */
declare const NuxtLayout: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    name: {
      type: StringConstructor;
      default: string;
    };
    fallback: {
      type: StringConstructor;
      default: undefined;
    };
  }>,
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | null,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      name: {
        type: StringConstructor;
        default: string;
      };
      fallback: {
        type: StringConstructor;
        default: undefined;
      };
    }>
  > &
    Readonly<{}>,
  {
    name: string;
    fallback: string;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
/**
 * Mock NuxtLoadingIndicator - renders nothing.
 */
declare const NuxtLoadingIndicator: _$vue.DefineComponent<
  {},
  {},
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
/**
 * Mock NuxtErrorBoundary - renders default slot.
 */
declare const NuxtErrorBoundary: _$vue.DefineComponent<
  {},
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | null,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtRouteAnnouncer: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    politeness: {
      type: StringConstructor;
      default: string;
    };
  }>,
  () => _$vue.VNode<
    _$vue.RendererNode,
    _$vue.RendererElement,
    {
      [key: string]: any;
    }
  >,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      politeness: {
        type: StringConstructor;
        default: string;
      };
    }>
  > &
    Readonly<{}>,
  {
    politeness: string;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtWelcome: _$vue.DefineComponent<
  {},
  () => _$vue.VNode<
    _$vue.RendererNode,
    _$vue.RendererElement,
    {
      [key: string]: any;
    }
  >,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtIsland: _$vue.DefineComponent<
  {},
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | null,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtClientFallback: _$vue.DefineComponent<
  {},
  () =>
    | _$vue.VNode<
        _$vue.RendererNode,
        _$vue.RendererElement,
        {
          [key: string]: any;
        }
      >[]
    | null,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<{}> & Readonly<{}>,
  {},
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtImg: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    src: {
      type: StringConstructor;
      required: true;
    };
    alt: {
      type: StringConstructor;
      default: string;
    };
    width: {
      type: (StringConstructor | NumberConstructor)[];
      default: undefined;
    };
    height: {
      type: (StringConstructor | NumberConstructor)[];
      default: undefined;
    };
  }>,
  () => _$vue.VNode<
    _$vue.RendererNode,
    _$vue.RendererElement,
    {
      [key: string]: any;
    }
  >,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      src: {
        type: StringConstructor;
        required: true;
      };
      alt: {
        type: StringConstructor;
        default: string;
      };
      width: {
        type: (StringConstructor | NumberConstructor)[];
        default: undefined;
      };
      height: {
        type: (StringConstructor | NumberConstructor)[];
        default: undefined;
      };
    }>
  > &
    Readonly<{}>,
  {
    alt: string;
    width: string | number;
    height: string | number;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
declare const NuxtPicture: _$vue.DefineComponent<
  _$vue.ExtractPropTypes<{
    src: {
      type: StringConstructor;
      required: true;
    };
    alt: {
      type: StringConstructor;
      default: string;
    };
  }>,
  () => _$vue.VNode<
    _$vue.RendererNode,
    _$vue.RendererElement,
    {
      [key: string]: any;
    }
  >,
  {},
  {},
  {},
  _$vue.ComponentOptionsMixin,
  _$vue.ComponentOptionsMixin,
  {},
  string,
  _$vue.PublicProps,
  Readonly<
    _$vue.ExtractPropTypes<{
      src: {
        type: StringConstructor;
        required: true;
      };
      alt: {
        type: StringConstructor;
        default: string;
      };
    }>
  > &
    Readonly<{}>,
  {
    alt: string;
  },
  {},
  {},
  {},
  string,
  _$vue.ComponentProvideOptions,
  true,
  {},
  any
>;
//#endregion
//#region src/index.d.ts
/**
 * Create Nuxt mock Vite plugin for Musea.
 */
declare function nuxtMusea(options?: NuxtMuseaOptions): Plugin;
//#endregion
export {
  ClientOnly,
  NuxtClientFallback,
  NuxtErrorBoundary,
  NuxtImg,
  NuxtIsland,
  NuxtLayout,
  NuxtLink,
  NuxtLoadingIndicator,
  type NuxtMuseaOptions,
  NuxtPage,
  NuxtPicture,
  NuxtRouteAnnouncer,
  NuxtWelcome,
  abortNavigation,
  callOnce,
  clearError,
  clearNuxtData,
  clearNuxtState,
  configureNuxtMuseaMocks,
  createError,
  createNuxtMuseaPreviewSetup,
  defineNuxtComponent,
  defineNuxtPlugin,
  defineNuxtRouteMiddleware,
  definePageMeta,
  getAppConfigState,
  getRouteState,
  getRuntimeConfigState,
  installNuxtMuseaMocks,
  navigateTo,
  nuxtMusea,
  onNuxtReady,
  prefetchComponents,
  preloadComponents,
  preloadRouteComponents,
  refreshNuxtData,
  reloadNuxtApp,
  resetNuxtMuseaMocks,
  setPageLayout,
  showError,
  updateAppConfig,
  useAppConfig,
  useAsyncData,
  useCookie,
  useError,
  useFetch,
  useHead,
  useHeadSafe,
  useId,
  useLazyAsyncData,
  useLazyFetch,
  useNuxtApp,
  useNuxtData,
  useRequestEvent,
  useRequestFetch,
  useRequestHeaders,
  useRequestURL,
  useRoute,
  useRouter,
  useRuntimeConfig,
  useSeoMeta,
  useServerSeoMeta,
  useState,
};
