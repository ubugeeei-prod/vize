/**
 * NuxtMusea plugin options.
 */
export interface NuxtMuseaOptions {
  /**
   * Mock route data.
   */
  route?: {
    path?: string;
    name?: string;
    params?: Record<string, string>;
    query?: Record<string, string>;
    hash?: string;
    fullPath?: string;
    meta?: Record<string, unknown>;
  };

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
}
