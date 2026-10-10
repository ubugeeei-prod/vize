import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type { IncomingMessage, ServerResponse } from "node:http";
import test from "node:test";
import type { Connect, ModuleNode, ViteDevServer } from "vite";
import { generateArtModule } from "./art-module.ts";
import { handlePreviewWithProps } from "./api-routes/post-handlers.ts";
import type { ApiRoutesContext } from "./api-routes/index.ts";
import { createHandleHotUpdate, createLoad, type VirtualModuleState } from "./plugin/virtual.ts";
import { previewModuleId } from "./preview-module-id.ts";
import { registerMiddleware } from "./server-middleware.ts";
import type { ArtFileInfo } from "./types/art.ts";
import { variantComponentNames } from "./variant-bindings.ts";

function context(names = ["State enabled", "State-enabled"]) {
  const art: ArtFileInfo = {
    path: "/repo/Host.art.vue",
    metadata: { title: "Bindings", tags: [], status: "ready" },
    variants: names.map((name, index) => ({
      name,
      template: `<button>${index}</button>`,
      isDefault: index === 0,
      skipVrt: false,
    })),
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  const state: VirtualModuleState = {
    basePath: "/gallery",
    inlineArt: false,
    artFiles: new Map([[art.path, art]]),
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    getConfigRoot: () => "/repo",
    getScanRoots: () => ["/repo"],
    getVueVersion: () => 3,
    getServer: () => null,
    processArtFile: async () => {},
  };
  return { art, state };
}

void test("art imports, Vue 2 declarations, defaults and virtual previews agree", () => {
  const { art, state } = context();
  const bindings = variantComponentNames(art.variants);
  for (const vueVersion of [2, 3] as const) {
    for (const selected of [-1, 0, 1]) {
      art.variants.forEach((variant, index) => (variant.isDefault = index === selected));
      const code = generateArtModule(art, art.path, { vueVersion });
      for (const variant of art.variants) {
        const binding = bindings.get(variant.name)!;
        assert.ok(
          code.includes(vueVersion === 2 ? `export const ${binding}` : `import ${binding} from`),
        );
        const load = createLoad({ ...state, getVueVersion: () => vueVersion });
        const preview = load(previewModuleId(art.path, variant.name).replace("virtual:", "\0"));
        assert.ok(preview?.includes(`artModule[${JSON.stringify(binding)}]`));
      }
      assert.ok(
        code.includes(`export default ${bindings.get(art.variants[Math.max(selected, 0)].name)};`),
      );
    }
  }
  assert.equal(
    createLoad(state)(previewModuleId(art.path, "Missing").replace("virtual:", "\0")),
    null,
  );
});

void test("dev fallback and props override choose each distinct binding", async () => {
  const { art, state } = context();
  const routes = new Map<string, Connect.NextHandleFunction>();
  const server = {
    middlewares: {
      use: (route: string, handler: Connect.NextHandleFunction) => routes.set(route, handler),
    },
    transformRequest: async () => null,
  } as unknown as ViteDevServer;
  registerMiddleware(server, {
    ...state,
    scanRoots: [],
    devSessionToken: "unit",
    themeConfig: undefined,
  });
  const bindings = variantComponentNames(art.variants);
  for (const variant of art.variants) {
    const expected = `artModule[${JSON.stringify(bindings.get(variant.name))}]`;
    let body = "";
    const response = {
      setHeader() {},
      end: (value: string) => (body = value),
    } as unknown as ServerResponse;
    const params = new URLSearchParams({ art: art.path, variant: variant.name });
    await Promise.resolve(
      routes.get("/gallery/preview-module")!(
        { url: `?${params.toString()}` } as IncomingMessage,
        response,
        (error?: unknown) => {
          if (error) throw error;
        },
      ),
    );
    assert.ok(body.includes(expected), body);
    handlePreviewWithProps(
      state as unknown as ApiRoutesContext,
      JSON.stringify({
        artPath: art.path,
        variantName: variant.name,
        props: { label: "Override" },
      }),
      response,
      () => assert.fail("unexpected JSON response"),
      (message) => assert.fail(message),
    );
    assert.ok(body.includes(expected), body);
    assert.ok(body.includes('"label":"Override"'));
  }
});

void test("HMR adding and removing a collision refreshes producer and previews without stale bindings", async () => {
  const { art, state } = context(["State enabled"]);
  const ids = [
    `\0musea-art:${art.path}?musea-virtual`,
    `\0musea-preview:${art.path}:State enabled`,
  ];
  const modules = ids.map((id) => ({ id }) as ModuleNode);
  state.getServer = () => ({
    moduleGraph: { idToModuleMap: new Map(modules.map((node) => [node.id!, node])) },
  });
  const load = createLoad(state);
  assert.ok(load(ids[0])!.includes("import StateEnabled from"));
  state.processArtFile = async () => {
    art.variants.push({ ...art.variants[0], name: "State-enabled", isDefault: false });
  };
  assert.deepEqual(await createHandleHotUpdate(state)({ file: art.path, modules: [] }), modules);
  const hashed = variantComponentNames(art.variants).get("State enabled")!;
  assert.ok(load(ids[0])!.includes(`import ${hashed} from`));
  assert.ok(load(ids[1])!.includes(`artModule[${JSON.stringify(hashed)}]`));
  state.processArtFile = async () => {
    art.variants.pop();
  };
  await createHandleHotUpdate(state)({ file: art.path, modules: [] });
  assert.ok(load(ids[0])!.includes("import StateEnabled from"));
  assert.ok(load(ids[1])!.includes('artModule["StateEnabled"]'));
});

void test("noncolliding generated modules remain byte-exact for Vue 2 and Vue 3", () => {
  const { art } = context(["State enabled", "State disabled"]);
  const expected = {
    2: "e09be98768ab656823be38c618d0bae4dc31288c560d7a587856d50333abbb2f",
    3: "2539ded0191a840bc7fe8bdabd4d2c517a43bb57d5f929ae249976d685d7ee82",
  };
  for (const vueVersion of [2, 3] as const) {
    const code = generateArtModule(art, art.path, { vueVersion });
    assert.equal(createHash("sha256").update(code).digest("hex"), expected[vueVersion]);
  }
});
