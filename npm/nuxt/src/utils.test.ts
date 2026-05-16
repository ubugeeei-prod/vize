import assert from "node:assert";
import {
  buildNuxtCompilerOptions,
  buildNuxtDevAssetBase,
  isVizeVirtualVueModuleId,
  normalizeNuxtInjectedKeysForVizeVirtualModule,
  normalizeVizeVirtualVueModuleId,
} from "./utils.ts";

assert.strictEqual(
  buildNuxtDevAssetBase("/", "/_nuxt/"),
  "/_nuxt/",
  "default Nuxt dev assets should stay under /_nuxt/",
);

assert.strictEqual(
  buildNuxtDevAssetBase("/2025/", "/_nuxt/"),
  "/2025/_nuxt/",
  "Nuxt baseURL should prefix buildAssetsDir",
);

assert.strictEqual(
  buildNuxtDevAssetBase("/preview", "_assets"),
  "/preview/_assets/",
  "missing slashes should be normalized",
);

assert.deepStrictEqual(
  buildNuxtCompilerOptions("/repo/app", "/2026/", "/_nuxt/"),
  {
    devUrlBase: "/2026/_nuxt/",
    root: "/repo/app",
  },
  "Nuxt compiler options should pin Vize root to the app root so vize.config.ts is discovered",
);

assert.equal(
  isVizeVirtualVueModuleId("\0vize-ssr:/repo/app/components/Foo.vue.ts"),
  true,
  "SSR virtual Vue modules should stay eligible for Nuxt bridge transforms",
);

assert.equal(
  normalizeVizeVirtualVueModuleId("\0vize-ssr:/repo/app/components/Foo.vue.ts"),
  "/repo/app/components/Foo.vue",
  "Nuxt bridge normalization should strip only the virtual .ts suffix",
);

assert.equal(
  normalizeVizeVirtualVueModuleId("\0/repo/app/components/Foo.vue.ts?macro=true"),
  "/repo/app/components/Foo.vue?macro=true",
  "Nuxt bridge normalization should preserve query strings on client virtual ids",
);

assert.equal(
  normalizeVizeVirtualVueModuleId("\0vize-ssr:/repo/app/components/Foo.vue.ts?vue&type=template"),
  "/repo/app/components/Foo.vue?vue&type=template",
  "Nuxt bridge normalization should preserve query strings on SSR virtual ids",
);

{
  const clientCode =
    "useFetch('/api/a', {}, '$client-a' /* nuxt-injected */); useFetch('/api/b', {}, '$client-b' /* nuxt-injected */)";
  const ssrCode =
    "useFetch('/api/a', {}, '$ssr-a' /* nuxt-injected */); useFetch('/api/b', {}, '$ssr-b' /* nuxt-injected */)";

  assert.equal(
    normalizeNuxtInjectedKeysForVizeVirtualModule(clientCode, "\0/repo/app/components/Foo.vue.ts"),
    normalizeNuxtInjectedKeysForVizeVirtualModule(
      ssrCode,
      "\0vize-ssr:/repo/app/components/Foo.vue.ts",
    ),
    "Nuxt injected keys should match between client and SSR virtual modules",
  );
}

console.log("✅ nuxt utils tests passed!");
