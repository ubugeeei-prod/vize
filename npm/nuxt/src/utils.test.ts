import assert from "node:assert";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
  buildNuxtCompilerOptions,
  buildNuxtDevAssetBase,
  isVizeGeneratedVueModuleId,
  isVizeVirtualVueModuleId,
  normalizeNuxtInjectedKeysForVizeVirtualModule,
  normalizeVizeVirtualVueModuleId,
  preserveExplicitVueImportsFromNuxtAutoImports,
  preserveExplicitVueImportsFromVizeModuleSource,
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
    exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
    root: "/repo/app",
    scanPatterns: [],
  },
  "Nuxt compiler options should use on-demand compilation to avoid retaining every SFC in large Nuxt apps",
);

assert.deepStrictEqual(
  buildNuxtCompilerOptions("/repo/app", "/2026/", "/_nuxt/", {
    configFile: "vize.nuxt.config.ts",
    debug: true,
    ignorePatterns: ["node_modules/**", ".nuxt/**", "fixtures/**"],
    scanPatterns: ["app/**/*.vue", "layers/**/*.vue"],
    sourceMap: false,
    vapor: true,
  }),
  {
    configFile: "vize.nuxt.config.ts",
    debug: true,
    devUrlBase: "/2026/_nuxt/",
    exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
    ignorePatterns: ["node_modules/**", ".nuxt/**", "fixtures/**"],
    root: "/repo/app",
    scanPatterns: ["app/**/*.vue", "layers/**/*.vue"],
    sourceMap: false,
    vapor: true,
  },
  "Nuxt compiler options should forward Vite plugin overrides while keeping Nuxt defaults",
);

assert.deepStrictEqual(
  buildNuxtCompilerOptions("/repo/app", "/2026/", "/_nuxt/", {
    customRenderer: true,
    exclude: /\.custom-renderer-only\.vue$/,
  }),
  {
    customRenderer: true,
    devUrlBase: "/2026/_nuxt/",
    exclude: /\.custom-renderer-only\.vue$/,
    root: "/repo/app",
    scanPatterns: [],
  },
  "custom renderer Nuxt compiler options should preserve explicit excludes without adding Takumi defaults",
);

assert.equal(
  isVizeVirtualVueModuleId("\0vize-ssr:/repo/app/components/Foo.vue.ts"),
  true,
  "SSR virtual Vue modules should stay eligible for Nuxt bridge transforms",
);

assert.equal(isVizeGeneratedVueModuleId("\0/repo/app/components/Foo.vue.ts"), true);
assert.equal(isVizeGeneratedVueModuleId("/repo/app/components/Foo.vue.ts"), true);
assert.equal(isVizeGeneratedVueModuleId("/@id/__x00__/repo/app/components/Foo.vue.ts"), true);
assert.equal(isVizeGeneratedVueModuleId("/repo/app/components/Foo.vue"), false);

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

{
  const originalCode = `import { resolveComponent, computed as _computed } from "vue";
const resolved = resolveComponent(name);
const doubled = _computed(() => value * 2);`;
  const injectedCode = `import { resolveComponent, computed as _computed, useRuntimeConfig } from "#imports";
const resolved = resolveComponent(name);
const doubled = _computed(() => value * 2);
const config = useRuntimeConfig();`;

  assert.equal(
    preserveExplicitVueImportsFromNuxtAutoImports(originalCode, injectedCode),
    `import { resolveComponent, computed as _computed } from "vue";
import { useRuntimeConfig } from "#imports";
const resolved = resolveComponent(name);
const doubled = _computed(() => value * 2);
const config = useRuntimeConfig();`,
    "Nuxt auto-imports should not move explicit Vue runtime imports from vize virtual modules into #imports",
  );
}

{
  const originalCode = `import { defineAsyncComponent } from "vue";
const component = defineAsyncComponent(loader);`;
  const injectedCode = `import { defineAsyncComponent, useRoute } from "#entry";
const component = defineAsyncComponent(loader);
const route = useRoute();`;

  assert.equal(
    preserveExplicitVueImportsFromNuxtAutoImports(originalCode, injectedCode),
    `import { defineAsyncComponent } from "vue";
import { useRoute } from "#entry";
const component = defineAsyncComponent(loader);
const route = useRoute();`,
    "Already-normalized #entry imports should also give explicit Vue helpers back to vue",
  );
}

{
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-utils-"));
  const sourcePath = path.join(tmpDir, "ContentRenderer.vue");
  fs.writeFileSync(
    sourcePath,
    `<script setup>
import { resolveComponent, computed as _computed } from "vue";
import { useRuntimeConfig } from "#imports";
</script>
`,
  );

  assert.equal(
    preserveExplicitVueImportsFromVizeModuleSource(
      `\0${sourcePath}.ts`,
      `import { resolveComponent, computed as _computed, useRuntimeConfig } from "#entry";
const resolved = resolveComponent(name);
const doubled = _computed(() => value * 2);
const config = useRuntimeConfig();`,
    ),
    `import { resolveComponent, computed as _computed } from "vue";
import { useRuntimeConfig } from "#entry";
const resolved = resolveComponent(name);
const doubled = _computed(() => value * 2);
const config = useRuntimeConfig();`,
    "Nuxt bridge should restore explicit Vue helpers from the original Vize module source",
  );
}

console.log("✅ nuxt utils tests passed!");
