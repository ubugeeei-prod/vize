import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components.ts";

type Fixture = {
  rootDir: string;
  buildDir: string;
};

const TEST_TMP_ROOT = path.resolve(process.cwd(), "target", "vize-tests", "nuxt-components-tests");

function writeFile(filePath: string, content = ""): string {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
  return filePath;
}

function writePackage(rootDir: string, name: string): string {
  const packageDir = path.join(rootDir, "node_modules", ...name.split("/"));
  writeFile(path.join(packageDir, "package.json"), JSON.stringify({ name, version: "0.0.0" }));
  return packageDir;
}

function createFixture(): Fixture {
  fs.mkdirSync(TEST_TMP_ROOT, { recursive: true });
  const rootDir = fs.realpathSync(fs.mkdtempSync(path.join(TEST_TMP_ROOT, "fixture-")));
  const buildDir = path.join(rootDir, ".nuxt");
  fs.mkdirSync(buildDir, { recursive: true });
  writeFile(path.join(rootDir, "package.json"), JSON.stringify({ private: true }));

  const nuxtDir = writePackage(rootDir, "nuxt");
  const i18nDir = writePackage(rootDir, "@nuxtjs/i18n");
  const pwaDir = writePackage(rootDir, "@vite-pwa/nuxt");

  const scrollToTopPath = writeFile(path.join(rootDir, "app/components/ScrollToTop.client.vue"));
  const commonPreviewPath = writeFile(
    path.join(rootDir, "app/components/common/CommonPreviewPrompt.vue"),
  );
  const nuxtLinkLocalePath = writeFile(
    path.join(i18nDir, "dist/runtime/components/NuxtLinkLocale.js"),
  );
  const nuxtPagePath = writeFile(path.join(nuxtDir, "dist/pages/runtime/page.js"));
  const nuxtStubsPath = writeFile(path.join(nuxtDir, "dist/app/components/nuxt-stubs.js"));
  writeFile(path.join(pwaDir, "dist/runtime/components/NuxtPwaAssets.js"));

  writeFile(
    path.join(buildDir, "components.d.ts"),
    [
      `export const NuxtLinkLocale: typeof import(${JSON.stringify(
        path.relative(buildDir, nuxtLinkLocalePath),
      )}).default`,
      `export const ScrollToTop: typeof import(${JSON.stringify(
        path.relative(buildDir, scrollToTopPath),
      )}).default`,
      `export const NuxtPage: typeof import(${JSON.stringify(
        path.relative(buildDir, nuxtPagePath),
      )})['default']`,
      `export const NuxtImg: typeof import(${JSON.stringify(
        path.relative(buildDir, nuxtStubsPath),
      )})['NuxtImg']`,
      `export const LazyCommonPreviewPrompt: LazyComponent<typeof import(${JSON.stringify(
        path.relative(buildDir, commonPreviewPath),
      )}).default>`,
      "",
    ].join("\n"),
  );

  return { rootDir, buildDir };
}

void test("Nuxt component resolver reads generated d.ts, runtime fallbacks, and component modes", () => {
  const fixture = createFixture();
  try {
    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      moduleNames: ["@vite-pwa/nuxt"],
      rootDir: fixture.rootDir,
    });

    assert.deepEqual(
      resolver.resolve("NuxtLinkLocale"),
      {
        exportName: "default",
        filePath: path.join(
          fixture.rootDir,
          "node_modules/@nuxtjs/i18n/dist/runtime/components/NuxtLinkLocale.js",
        ),
      },
      "Nuxt-generated d.ts should resolve module components",
    );

    const nuxtPwaAssets = resolver.resolve("NuxtPwaAssets");
    assert.deepEqual(
      nuxtPwaAssets,
      {
        exportName: "default",
        filePath: path.join(
          fixture.rootDir,
          "node_modules/@vite-pwa/nuxt/dist/runtime/components/NuxtPwaAssets.js",
        ),
      },
      "runtime component fallback should resolve module-added components missing from Nuxt d.ts",
    );

    assert.deepEqual(
      resolver.resolve("ScrollToTop"),
      {
        exportName: "default",
        filePath: path.join(fixture.rootDir, "app/components/ScrollToTop.client.vue"),
        mode: "client",
      },
      "Nuxt-generated d.ts should preserve client-only component mode",
    );

    assert.deepEqual(
      resolver.resolve("NuxtPage"),
      {
        exportName: "default",
        filePath: path.join(fixture.rootDir, "node_modules/nuxt/dist/pages/runtime/page.js"),
      },
      "Nuxt bracket-notation exports should resolve built-in components",
    );

    assert.deepEqual(
      resolver.resolve("LazyCommonPreviewPrompt"),
      {
        exportName: "default",
        filePath: path.join(fixture.rootDir, "app/components/common/CommonPreviewPrompt.vue"),
        lazy: true,
      },
      "Lazy-prefixed component aliases should preserve async component intent",
    );

    assert.deepEqual(
      resolver.resolve("NuxtImg"),
      {
        exportName: "NuxtImg",
        filePath: path.join(fixture.rootDir, "node_modules/nuxt/dist/app/components/nuxt-stubs.js"),
      },
      "named exports in bracket notation should resolve correctly",
    );
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt component resolver reads GlobalComponents declarations", () => {
  const fixture = createFixture();
  try {
    const helloCardPath = writeFile(path.join(fixture.rootDir, "app/components/HelloCard.vue"));
    const quotedWidgetPath = writeFile(
      path.join(fixture.rootDir, "app/components/widgets/QuotedWidget.vue"),
    );
    writeFile(
      path.join(fixture.buildDir, "types/components.d.ts"),
      [
        `declare module "vue" {`,
        `  export interface GlobalComponents {`,
        `    HelloCard: typeof import(${JSON.stringify(
          path.relative(path.join(fixture.buildDir, "types"), helloCardPath),
        )})["default"]`,
        `    "QuotedWidget": typeof import(${JSON.stringify(
          path.relative(path.join(fixture.buildDir, "types"), quotedWidgetPath),
        )})["default"]`,
        `  }`,
        `}`,
        `export {}`,
        "",
      ].join("\n"),
    );

    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      rootDir: fixture.rootDir,
    });

    assert.deepEqual(
      resolver.resolve("HelloCard"),
      {
        exportName: "default",
        filePath: path.join(fixture.rootDir, "app/components/HelloCard.vue"),
      },
      "Nuxt 4 GlobalComponents d.ts entries should resolve app components",
    );
    assert.deepEqual(
      resolver.resolve("quoted-widget"),
      {
        exportName: "default",
        filePath: path.join(fixture.rootDir, "app/components/widgets/QuotedWidget.vue"),
      },
      "quoted GlobalComponents entries should get kebab aliases",
    );
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt component import injection rewrites resolved runtime components", () => {
  const fixture = createFixture();
  try {
    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      moduleNames: ["@vite-pwa/nuxt"],
      rootDir: fixture.rootDir,
    });

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
      (name) => resolver.resolve(name),
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
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt component import injection wraps client-only components", () => {
  const fixture = createFixture();
  try {
    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      rootDir: fixture.rootDir,
    });

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
      (name) => resolver.resolve(name),
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
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt component resolver keeps NuxtRouteAnnouncer unwrapped", () => {
  const fixture = createFixture();
  try {
    const routeAnnouncerPath = writeFile(
      path.join(fixture.rootDir, "node_modules/nuxt/dist/app/components/nuxt-route-announcer.js"),
    );
    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      rootDir: fixture.rootDir,
    });
    resolver.register([
      {
        pascalName: "NuxtRouteAnnouncer",
        kebabName: "nuxt-route-announcer",
        name: "NuxtRouteAnnouncer",
        filePath: routeAnnouncerPath,
        export: "default",
        mode: "client",
      },
    ]);

    assert.deepEqual(
      resolver.resolve("NuxtRouteAnnouncer"),
      {
        exportName: "default",
        filePath: routeAnnouncerPath,
        mode: "client",
      },
      "components:extend entries should preserve explicit client-only mode",
    );

    const transformed = injectNuxtComponentImports(
      `
export default {
  setup(__props) {
    return (_ctx, _cache) => {
      const _component_NuxtRouteAnnouncer = resolveComponent("NuxtRouteAnnouncer");
      return _component_NuxtRouteAnnouncer;
    };
  }
}
`,
      (name) => resolver.resolve(name),
    );

    assert.match(
      transformed,
      /import __nuxt_component_0 from ".*nuxt-route-announcer\.js";/,
      "NuxtRouteAnnouncer should be imported directly before use",
    );
    assert.equal(
      transformed.includes("__nuxt_create_client_only"),
      false,
      "NuxtRouteAnnouncer should not use createClientOnly",
    );

    const componentImportTransformed = injectNuxtComponentImports(
      `
import { NuxtPage, NuxtRouteAnnouncer } from "#components";

export default {
  setup(__props) {
    return (_ctx, _cache) => {
      return [NuxtPage, NuxtRouteAnnouncer];
    };
  }
}
`,
      (name) => resolver.resolve(name),
    );

    assert.equal(
      componentImportTransformed.includes('from "#components"'),
      false,
      "#components imports should be rewritten when every imported component resolves",
    );
    assert.match(
      componentImportTransformed,
      /import NuxtPage from ".*page\.js";/,
      "#components imports should preserve imported local component names",
    );
    assert.match(
      componentImportTransformed,
      /import NuxtRouteAnnouncer from ".*nuxt-route-announcer\.js";/,
      "#components imports should keep NuxtRouteAnnouncer direct",
    );
    assert.equal(
      componentImportTransformed.includes("__nuxt_create_client_only"),
      false,
      "#components imports should not wrap NuxtRouteAnnouncer",
    );
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt component import injection dedupes repeated component resolutions", () => {
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
});

void test("Nuxt component import injection preserves lazy and lazy client-only semantics", () => {
  const fixture = createFixture();
  try {
    const resolver = createNuxtComponentResolver({
      buildDir: fixture.buildDir,
      rootDir: fixture.rootDir,
    });

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
      (name) => resolver.resolve(name),
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
      (name) => resolver.resolve(name),
    );

    assert.match(
      lazyClientOnlyTransformed,
      /const __nuxt_component_0 = __nuxt_define_async_component\(\(\) => import\(".*ScrollToTop\.client\.vue"\)\.then\(\(module\) => __nuxt_create_client_only\(module\.default\)\)\);/,
      "lazy client-only components should wrap their async payload with createClientOnly",
    );
  } finally {
    fs.rmSync(fixture.rootDir, { recursive: true, force: true });
  }
});
