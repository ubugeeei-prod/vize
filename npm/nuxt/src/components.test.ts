import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const npmxFixtureRoot = path.resolve(__dirname, "../../../tests/_fixtures/_git/npmx.dev");
const elkFixtureRoot = path.resolve(__dirname, "../../../tests/_fixtures/_git/elk");

const npmxResolver = createNuxtComponentResolver({
  buildDir: path.join(npmxFixtureRoot, ".nuxt"),
  moduleNames: ["@vite-pwa/nuxt"],
  rootDir: npmxFixtureRoot,
});

const elkResolver = createNuxtComponentResolver({
  buildDir: path.join(elkFixtureRoot, ".nuxt"),
  rootDir: elkFixtureRoot,
});

assert.deepEqual(
  npmxResolver.resolve("NuxtLinkLocale"),
  {
    exportName: "default",
    filePath: path.join(
      npmxFixtureRoot,
      "node_modules/.pnpm/@nuxtjs+i18n@10.2.3_@upstash+redis@1.36.2_@vue+compiler-dom@3.5.29_db0@0.3.4_better-sql_87e0fb07cee3a7f571dc433b5422e7ef/node_modules/@nuxtjs/i18n/dist/runtime/components/NuxtLinkLocale.js",
    ),
  },
  "Nuxt-generated d.ts should resolve module components",
);

assert.deepEqual(
  npmxResolver.resolve("NuxtPwaAssets"),
  {
    exportName: "default",
    filePath: path.join(
      npmxFixtureRoot,
      "node_modules/.pnpm/@vite-pwa+nuxt@1.1.1_@vite-pwa+assets-generator@1.0.2_magicast@0.5.1_vite@8.0.0-beta.10_0578499fc59b04792d0542db4407cb90/node_modules/@vite-pwa/nuxt/dist/runtime/components/NuxtPwaAssets.js",
    ),
  },
  "runtime component fallback should resolve module-added components missing from Nuxt d.ts",
);

assert.deepEqual(
  npmxResolver.resolve("ScrollToTop"),
  {
    exportName: "default",
    filePath: path.join(
      npmxFixtureRoot,
      "app/components/ScrollToTop.client.vue",
    ),
    mode: "client",
  },
  "Nuxt-generated d.ts should preserve client-only component mode",
);

assert.deepEqual(
  elkResolver.resolve("NuxtPage"),
  {
    exportName: "default",
    filePath: path.join(
      elkFixtureRoot,
      "node_modules/.pnpm/nuxt@4.1.2_@parcel+watcher@2.5.1_@types+node@24.10.1_@upstash+redis@1.35.4_@vercel+kv@3_6c5e8b54358a47470a7b7512245bea5a/node_modules/nuxt/dist/pages/runtime/page.js",
    ),
  },
  "Nuxt 4 bracket-notation exports should resolve built-in components",
);

assert.deepEqual(
  elkResolver.resolve("LazyCommonPreviewPrompt"),
  {
    exportName: "default",
    filePath: path.join(
      elkFixtureRoot,
      "app/components/common/CommonPreviewPrompt.vue",
    ),
    lazy: true,
  },
  "Lazy-prefixed component aliases should preserve async component intent",
);

assert.deepEqual(
  elkResolver.resolve("NuxtImg"),
  {
    exportName: "NuxtImg",
    filePath: path.join(
      elkFixtureRoot,
      "node_modules/.pnpm/nuxt@4.1.2_@parcel+watcher@2.5.1_@types+node@24.10.1_@upstash+redis@1.35.4_@vercel+kv@3_6c5e8b54358a47470a7b7512245bea5a/node_modules/nuxt/dist/app/components/nuxt-stubs.js",
    ),
  },
  "named exports in bracket notation should resolve correctly",
);

const transformed = injectNuxtComponentImports(
  `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const _component_NuxtPwaAssets = resolveComponent("NuxtPwaAssets");
      return _component_NuxtPwaAssets;
    };
  }
}
`,
  (name) => npmxResolver.resolve(name),
);

assert.match(
  transformed,
  /import __nuxt_component_0 from .*NuxtPwaAssets\.js";/,
  "resolved components should become direct imports",
);
assert.equal(
  transformed.includes('resolveComponent("NuxtPwaAssets")'),
  false,
  "resolved components should no longer go through resolveComponent()",
);

const clientOnlyTransformed = injectNuxtComponentImports(
  `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const _component_ScrollToTop = resolveComponent("ScrollToTop");
      return _component_ScrollToTop;
    };
  }
}
`,
  (name) => npmxResolver.resolve(name),
);

assert.match(
  clientOnlyTransformed,
  /import \{ createClientOnly as __nuxt_create_client_only \} from "#app\/components\/client-only";/,
  "client-only components should import createClientOnly",
);
assert.match(
  clientOnlyTransformed,
  /import __nuxt_component_0_raw from ".*ScrollToTop\.client\.vue";\s*const __nuxt_component_0 = __nuxt_create_client_only\(__nuxt_component_0_raw\);/s,
  "client-only components should be wrapped before use",
);

const deduped = injectNuxtComponentImports(
  `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const first = resolveComponent("AppHeader");
      const second = resolveComponent("AppHeader");
      return [first, second];
    };
  }
}
`,
  (name) => {
    if (name === "AppHeader") {
      return {
        exportName: "default",
        filePath: "/virtual/AppHeader.vue",
      };
    }
    return null;
  },
);

assert.equal(
  deduped.match(/import __nuxt_component_0 from "\/virtual\/AppHeader\.vue";/g)?.length,
  1,
  "reused components should emit a single import",
);
assert.equal(
  deduped.match(/__nuxt_component_0/g)?.length,
  3,
  "reused components should share the same imported binding",
);

const lazyTransformed = injectNuxtComponentImports(
  `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const lazy = resolveComponent("LazyCommonPreviewPrompt");
      const eager = resolveComponent("NuxtPage");
      return [lazy, eager];
    };
  }
}
`,
  (name) => elkResolver.resolve(name),
);

assert.match(
  lazyTransformed,
  /import \{ defineAsyncComponent as __nuxt_define_async_component \} from "vue";/,
  "lazy components should import defineAsyncComponent once",
);
assert.match(
  lazyTransformed,
  /const __nuxt_component_0 = __nuxt_define_async_component\(\(\) => import\(".*CommonPreviewPrompt\.vue"\)\.then\(\(module\) => module\.default\)\);/,
  "lazy component resolution should preserve async loading",
);
assert.match(
  lazyTransformed,
  /import __nuxt_component_1 from ".*page\.js";/,
  "non-lazy components should remain direct imports",
);

const lazyClientOnlyTransformed = injectNuxtComponentImports(
  `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const lazy = resolveComponent("LazyScrollToTop");
      return lazy;
    };
  }
}
`,
  (name) => npmxResolver.resolve(name),
);

assert.match(
  lazyClientOnlyTransformed,
  /const __nuxt_component_0 = __nuxt_define_async_component\(\(\) => import\(".*ScrollToTop\.client\.vue"\)\.then\(\(module\) => __nuxt_create_client_only\(module\.default\)\)\);/,
  "lazy client-only components should wrap their async payload with createClientOnly",
);

console.log("✅ nuxt component bridge tests passed!");
