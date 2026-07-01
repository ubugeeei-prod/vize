import assert from "node:assert/strict";
import test from "node:test";
import { createApp, h } from "vue";

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
