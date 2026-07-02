import type { Page } from "@playwright/test";

export async function disableViteHmrClient(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const global = globalThis as typeof globalThis & {
      [key: string]: unknown;
      process?: { env?: Record<string, string>; test?: boolean };
    };
    const defineGlobal = (key: string, value: unknown) => {
      Object.defineProperty(global, key, { configurable: true, value, writable: true });
    };

    global.process ??= {};
    global.process.env ??= {};
    global.process.test = true;
    for (const [key, value] of Object.entries({
      __DEFAULT_COOKIE_KEY__: "i18n_redirected",
      __DEFAULT_DIRECTION__: "ltr",
      __DIFFERENT_DOMAINS__: false,
      __DYNAMIC_PARAMS_KEY__: "__nuxt_i18n_params",
      __I18N_CACHE__: false,
      __I18N_CACHE_LIFETIME__: 3_600,
      __I18N_FULL_STATIC__: false,
      __I18N_HASH__: "dev",
      __I18N_PRELOAD__: false,
      __I18N_ROUTING__: false,
      __I18N_SERVER_REDIRECT__: false,
      __I18N_STRATEGY__: "no_prefix",
      __I18N_STRICT_SEO__: false,
      __I18N_STRIP_UNUSED__: false,
      __IS_SSG__: false,
      __IS_SSR__: false,
      __MULTI_DOMAIN_LOCALES__: false,
      __NUXT_ASYNC_CONTEXT__: false,
      __NUXT_I18N_VERSION__: "10.1.0",
      __NUXT_VERSION__: "4.1.2",
      __PARALLEL_PLUGIN__: false,
      __ROUTE_NAME_DEFAULT_SUFFIX__: "default",
      __ROUTE_NAME_SEPARATOR__: "___",
      __SWITCH_LOCALE_PATH_LINK_IDENTIFIER__: "nuxt-i18n-slp",
      __TRAILING_SLASH__: false,
      __VUE_OPTIONS_API__: true,
      __VUE_PROD_DEVTOOLS__: false,
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false,
    })) {
      defineGlobal(key, value);
    }
  });
  await page.route("**/_nuxt/@vite/client", (route) =>
    route.fulfill({
      body: [
        "const noop = () => {};",
        "const hotContext = {",
        "  accept: noop,",
        "  acceptExports: noop,",
        "  data: {},",
        "  decline: noop,",
        "  dispose: noop,",
        "  invalidate: noop,",
        "  off: noop,",
        "  on: noop,",
        "  prune: noop,",
        "  send: noop,",
        "};",
        "export const createHotContext = () => hotContext;",
        "export const injectQuery = (url) => url;",
        "export const removeStyle = noop;",
        "export const updateStyle = noop;",
        "export default {};",
        "",
      ].join("\n"),
      contentType: "text/javascript",
      status: 200,
    }),
  );
}
