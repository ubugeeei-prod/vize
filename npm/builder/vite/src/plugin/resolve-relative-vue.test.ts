import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { loadHook } from "./load.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";

const testRoot = fs.mkdtempSync(
  path.join(fs.realpathSync(os.tmpdir()), "vize-vite-plugin-relative-vue-"),
);

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createState(root: string): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: {
      log() {},
      info() {},
      warn() {},
      error() {},
    } as never,
  };
}

function expectResolvedId(resolved: Awaited<ReturnType<typeof resolveIdHook>>): string {
  assert.notEqual(resolved, null);
  assert.notEqual(resolved, undefined);
  return typeof resolved === "string" ? resolved : resolved.id;
}

{
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "nuxt-ui-override-"));
  const runtimeDir = path.join(projectRoot, "node_modules", "@nuxt", "ui", "dist", "runtime");
  const source = path.join(runtimeDir, "components", "Button.vue");
  const override = path.join(runtimeDir, "vue", "components", "Icon.vue");
  writeFixtureFile(source, '<script setup>import UIcon from "./Icon.vue"</script>');
  writeFixtureFile(override, "<template><span /></template>");

  let resolverImporter: string | undefined;
  const resolved = await resolveIdHook(
    {
      resolve: async (id, importer) => {
        resolverImporter = importer;
        return id === "./Icon.vue" && importer === source ? { id: override } : null;
      },
    },
    createState(projectRoot),
    "./Icon.vue",
    toPluginVisibleVirtualId(source),
    undefined,
  );

  assert.equal(
    resolverImporter,
    source,
    "Relative Vue imports from virtual modules should reach Vite resolvers with the real importer path",
  );
  assert.equal(
    expectResolvedId(resolved),
    toPluginVisibleVirtualId(override),
    "Relative Vue imports from virtual modules should preserve upstream component override resolvers",
  );

  const scanResolved = await resolveIdHook(
    {
      resolve: async (id, importer) =>
        id === "./Icon.vue" && importer === source ? { id: override } : null,
    },
    createState(projectRoot),
    "./Icon.vue",
    source,
    { scan: true },
  );
  assert.equal(
    expectResolvedId(scanResolved),
    override,
    "Nested SFC imports should stay scannable after a resolver override",
  );
}

// Vite resolves the complete asset request against the physical SFC importer.
// Preserve every returned field in both graphs and dependency scans.
{
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "glob-assets-"));
  const source = path.join(projectRoot, "src/pages/demo/Plain.vue");
  const target = path.join(projectRoot, "fixtures/Alpha.vue");
  writeFixtureFile(source, "<template><p>original</p></template>");
  writeFixtureFile(target, "<template><p>alpha</p></template>");
  for (const query of ["raw&import=default", "url", "worker&inline", "sharedworker"])
    for (const ssr of [false, true])
      for (const scan of [false, true]) {
        const id = `../../../fixtures/Alpha.vue?${query}`;
        const resolved = Object.freeze({
          id: `${target}?${query}`,
          external: false,
          meta: Object.freeze({ assetQuery: query }),
        });
        const calls: unknown[] = [];
        const state = createState(projectRoot);
        const result = await resolveIdHook(
          {
            resolve: async (request, importer, options) => {
              calls.push({ request, importer, options });
              return resolved;
            },
          },
          state,
          id,
          toPluginVisibleVirtualId(source),
          { ssr, scan },
        );
        assert.equal(result, resolved);
        assert.deepEqual(result, {
          id: `${target}?${query}`,
          external: false,
          meta: { assetQuery: query },
        });
        assert.deepEqual(calls, [{ request: id, importer: source, options: { skipSelf: true } }]);
        assert.equal(state.cache.size, 0);
        assert.equal(state.ssrCache.size, 0);
      }
  const resolvedAsset = Object.freeze({ id: `${target}?raw`, external: false });
  assert.equal(
    await resolveIdHook(
      { resolve: async () => resolvedAsset },
      createState(projectRoot),
      "../../../fixtures/Alpha.vue",
      toPluginVisibleVirtualId(source),
    ),
    resolvedAsset,
    "The effective query from an upstream resolver retains its complete asset result",
  );
}

// Persisted import graphs replay root-relative URLs before any SFC import has
// established the absolute filesystem identity in Vite's module graph (#9363).
{
  const projectRoot = fs.mkdtempSync(path.join(testRoot, "persisted-vue-url-"));
  const source = path.join(projectRoot, "src", "Hello.vue");
  writeFixtureFile(source, "<template><p>Hello</p></template>");
  const state = createState(projectRoot);
  const context = { resolve: async () => null };
  const legacyId = source + "?vue&vize-ssr&t=123";
  const directLegacy = loadHook(createState(projectRoot), legacyId);
  const resolvedLegacy = expectResolvedId(await resolveIdHook(context, state, legacyId));
  const resolvedLegacyLoad = loadHook(createState(projectRoot), resolvedLegacy);
  assert.ok(directLegacy && resolvedLegacyLoad);
  const legacyCode = typeof directLegacy === "string" ? directLegacy : directLegacy.code;
  const resolvedLegacyCode =
    typeof resolvedLegacyLoad === "string" ? resolvedLegacyLoad : resolvedLegacyLoad.code;
  assert.match(
    legacyCode,
    /ssrRender/,
    "the native compiler must produce the real legacy SSR module",
  );
  assert.equal(
    resolvedLegacyCode,
    legacyCode,
    "Resolve then load must retain the complete native SSR output when only the legacy query selects SSR",
  );
  const directClient = loadHook(createState(projectRoot), toPluginVisibleVirtualId(source));
  assert.ok(directClient);
  const clientCode = typeof directClient === "string" ? directClient : directClient.code;
  assert.notEqual(
    clientCode,
    legacyCode,
    "the genuine client and SSR compilations must be distinguishable",
  );
  assert.doesNotMatch(clientCode, /ssrRender/);
  for (const ssr of [false, true]) {
    for (const suffix of ["?vue&vize", "?vue&vize&used=true", "?vue&vize-ssr&t=123"]) {
      const expected = ssr ? toPluginVisibleVirtualId(source, true, suffix) : source + suffix;
      for (const id of ["/src/Hello.vue" + suffix, source + suffix, "/@fs" + source + suffix]) {
        assert.equal(
          expectResolvedId(await resolveIdHook(context, state, id, undefined, { ssr })),
          expected,
          "Fresh module graphs must resolve persisted URLs to the same absolute SFC identity",
        );
        assert.equal(
          expectResolvedId(await resolveIdHook(context, state, id, undefined, { ssr, scan: true })),
          source,
          "Dependency scans must receive the real source for persisted and absolute SFC URLs",
        );
      }
    }
  }
  const externalRoot = fs.mkdtempSync(path.join(testRoot, "external-persisted-vue-"));
  const externalSource = path.join(externalRoot, "Outside.vue");
  writeFixtureFile(externalSource, "<template><span>Outside</span></template>");
  assert.equal(
    expectResolvedId(await resolveIdHook(context, state, toPluginVisibleVirtualId(externalSource))),
    toPluginVisibleVirtualId(externalSource),
    "Real absolute SFCs outside Vite root must retain their filesystem identity",
  );
}

console.log("vite-plugin-vize relative Vue resolve tests passed!");
