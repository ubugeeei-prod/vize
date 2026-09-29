import assert from "node:assert/strict";
import test from "node:test";
import { createApp, h } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

type MuseaNuxtRuntime = typeof import("./index.ts");

async function loadRuntime(): Promise<MuseaNuxtRuntime> {
  return (await import(new URL("../dist/index.mjs", import.meta.url).href)) as MuseaNuxtRuntime;
}

void test("Nuxt Musea mocks configure route, runtime, app, state, cookie, and fetch data", async () => {
  const {
    configureNuxtMuseaMocks,
    resetNuxtMuseaMocks,
    useAppConfig,
    useCookie,
    useFetch,
    useRequestHeaders,
    useRequestURL,
    useRoute,
    useRuntimeConfig,
    useState,
  } = await loadRuntime();
  resetNuxtMuseaMocks();
  configureNuxtMuseaMocks({
    route: {
      path: "/users/42",
      params: { id: "42" },
      query: { tab: "profile" },
      meta: { auth: true },
    },
    runtimeConfig: {
      public: { apiBase: "/api" },
      secret: "server-only",
    },
    appConfig: {
      ui: { primary: "green" },
    },
    stateMocks: {
      count: 3,
    },
    cookieMocks: {
      session: "abc",
    },
    fetchMocks: {
      "/api/users/42": { id: 42, name: "Ada" },
    },
    request: {
      url: "https://example.test/stories",
      headers: { "x-preview": "1" },
    },
  });

  assert.equal(useRoute().fullPath, "/users/42?tab=profile");
  assert.deepEqual(useRoute().params, { id: "42" });
  assert.deepEqual(useRoute().meta, { auth: true });
  assert.deepEqual(useRuntimeConfig(), {
    public: { apiBase: "/api" },
    secret: "server-only",
  });
  assert.deepEqual(useAppConfig(), { ui: { primary: "green" } });
  assert.equal(useState<number>("count").value, 3);
  assert.equal(useState<number>("count"), useState<number>("count"));
  assert.equal(useCookie<string>("session").value, "abc");
  assert.equal(useCookie<string>("session"), useCookie<string>("session"));
  assert.deepEqual(useFetch<{ id: number; name: string }>("/api/users/42").data.value, {
    id: 42,
    name: "Ada",
  });
  assert.deepEqual(useRequestHeaders(), { "x-preview": "1" });
  assert.equal(useRequestURL().href, "https://example.test/stories");
});

void test("navigation mocks mutate the shared route state", async () => {
  const { navigateTo, resetNuxtMuseaMocks, useRoute, useRouter } = await loadRuntime();
  resetNuxtMuseaMocks();

  await navigateTo({ path: "/settings", query: { panel: "profile" }, hash: "top" });

  assert.equal(useRoute().fullPath, "/settings?panel=profile#top");

  await useRouter().push("/dashboard");

  assert.equal(useRoute().path, "/dashboard");
  assert.equal(
    useRouter().resolve({ path: "/dashboard", query: { page: "1" } }).href,
    "/dashboard?page=1",
  );
});

void test("NuxtLink uses href as its navigation target when provided", async () => {
  const { NuxtLink, resetNuxtMuseaMocks, useRoute } = await loadRuntime();
  resetNuxtMuseaMocks();

  const render = (
    NuxtLink as unknown as {
      setup: (
        props: Record<string, unknown>,
        context: { slots: Record<string, () => string> },
      ) => () => { props: { onClick: (event: MouseEvent) => void; href: string } };
    }
  ).setup(
    {
      href: "/from-href",
      to: "/from-to",
      external: false,
      replace: false,
      custom: false,
    },
    { slots: { default: () => "link" } },
  );
  const vnode = render();
  let prevented = false;

  vnode.props.onClick({
    preventDefault: () => {
      prevented = true;
    },
  } as MouseEvent);

  assert.equal(vnode.props.href, "/from-href");
  assert.equal(prevented, true);
  assert.equal(useRoute().path, "/from-href");
});

void test("runtime helpers keep app config and error state reactive", async () => {
  const { clearError, resetNuxtMuseaMocks, showError, updateAppConfig, useAppConfig, useError } =
    await loadRuntime();
  resetNuxtMuseaMocks();

  updateAppConfig({ theme: "dark" });
  assert.equal(useAppConfig().theme, "dark");

  const error = showError({ statusCode: 404, statusMessage: "Not found" });
  assert.equal(useError().value, error);
  assert.equal(useError().value?.message, "Not found");

  await clearError();
  assert.equal(useError().value, null);
});

void test("installNuxtMuseaMocks registers Nuxt built-ins and global properties", async () => {
  const { installNuxtMuseaMocks, resetNuxtMuseaMocks } = await loadRuntime();
  resetNuxtMuseaMocks();
  const app = createApp({
    render: () => h("div"),
  });

  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { baseURL: "/mock" } },
  });

  assert.ok(app.component("NuxtLink"));
  assert.ok(app.component("NuxtPage"));
  assert.ok(app.component("ClientOnly"));
  assert.equal(app.config.globalProperties.$route.path, "/preview");
  assert.deepEqual(app.config.globalProperties.$config, { public: { baseURL: "/mock" } });
});

void test("installNuxtMuseaMocks keeps vue-router route, router, and link components", async () => {
  const { installNuxtMuseaMocks, resetNuxtMuseaMocks } = await loadRuntime();
  resetNuxtMuseaMocks();
  const app = createApp({ render: () => h("div") });
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: "/", component: { render: () => null } }],
  });
  app.use(router);
  const link = app.component("RouterLink");
  const view = app.component("RouterView");
  const routeDescriptor = Object.getOwnPropertyDescriptor(app.config.globalProperties, "$route");
  assert.equal(routeDescriptor?.configurable, false);

  installNuxtMuseaMocks(app, { route: { path: "/preview" } });

  assert.equal(app.config.globalProperties.$route.path, "/");
  assert.equal(app.config.globalProperties.$router, router);
  assert.equal(app.component("RouterLink"), link);
  assert.equal(app.component("RouterView"), view);
});

void test("installNuxtMuseaMocks does not overwrite a non-configurable $route", async () => {
  const { installNuxtMuseaMocks, resetNuxtMuseaMocks } = await loadRuntime();
  resetNuxtMuseaMocks();
  const app = createApp({ render: () => h("div") });
  const route = { path: "/from-router" };
  const router = { push() {} };
  Object.defineProperty(app.config.globalProperties, "$route", {
    enumerable: true,
    configurable: false,
    get: () => route,
  });
  app.config.globalProperties.$router = router;

  installNuxtMuseaMocks(app, { route: { path: "/preview" } });

  assert.equal(app.config.globalProperties.$route, route);
  assert.equal(app.config.globalProperties.$router, router);
});

void test("installNuxtMuseaMocks mocks RouterLink and RouterView when no router is installed", async () => {
  const { installNuxtMuseaMocks, resetNuxtMuseaMocks, useRoute } = await loadRuntime();
  resetNuxtMuseaMocks();
  const app = createApp({ render: () => h("div") });
  installNuxtMuseaMocks(app);

  const routerLink = app.component("RouterLink") as unknown as {
    setup: (
      props: Record<string, unknown>,
      context: { slots: Record<string, (...args: never[]) => unknown> },
    ) => () => {
      type?: unknown;
      props?: { href?: string; onClick?: (event: MouseEvent) => void };
    };
  };
  const routerView = app.component("RouterView") as unknown as {
    setup: (
      props: Record<string, unknown>,
      context: { slots: Record<string, () => unknown> },
    ) => () => { type?: unknown; props?: Record<string, unknown> };
  };

  const custom = routerLink.setup(
    { to: "/about", custom: true },
    {
      slots: { default: ({ href }: { href: string }) => href },
    },
  )();
  assert.equal(custom, "/about");

  const link = routerLink.setup(
    { to: "/about", external: false, replace: false, custom: false },
    { slots: { default: () => "About" } },
  )();
  assert.equal(link.type, "a");
  assert.equal(link.props?.href, "/about");
  let prevented = false;
  link.props?.onClick?.({
    preventDefault: () => {
      prevented = true;
    },
  } as MouseEvent);
  assert.equal(prevented, true);
  assert.equal(useRoute().path, "/about");

  const view = routerView.setup({ name: "default" }, { slots: {} })();
  assert.equal(view.type, "div");
  assert.equal(view.props?.["data-router-view"], "default");
});

void test("installNuxtMuseaMocks does not replace a RouterLink the app already registered", async () => {
  const { installNuxtMuseaMocks, resetNuxtMuseaMocks } = await loadRuntime();
  resetNuxtMuseaMocks();
  const app = createApp({ render: () => h("div") });
  const existingLink = { name: "ExistingLink", setup: () => () => h("span", "kept") };
  const existingView = { name: "ExistingView", setup: () => () => h("section") };
  app.component("RouterLink", existingLink);
  app.component("RouterView", existingView);

  installNuxtMuseaMocks(app);

  assert.equal(app.component("RouterLink"), existingLink);
  assert.equal(app.component("RouterView"), existingView);
});
