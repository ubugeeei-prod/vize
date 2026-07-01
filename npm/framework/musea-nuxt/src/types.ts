/**
 * NuxtMusea plugin options.
 */
export type NuxtMuseaRouteQueryValue = string | string[] | null | undefined;

export interface NuxtMuseaRouteLocation {
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

export type NuxtMuseaNavigationTarget =
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

export interface NuxtMuseaOptions {
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
