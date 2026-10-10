import assert from "node:assert/strict";
import crypto from "node:crypto";
import { readFile } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import test from "node:test";
import type { Connect, ViteDevServer } from "vite";
import { createLoad, createResolveId, type VirtualModuleState } from "./plugin/virtual.ts";
import { previewModuleId } from "./preview-module-id.ts";
import { registerMiddleware } from "./server-middleware.ts";
import { staticPreviewId } from "./static-data.ts";
import { loadStaticRuntimeModule } from "./static-export.ts";
import type { ArtFileInfo } from "./types/art.ts";
import { toPascalCase } from "./utils.ts";

function context(artPath: string, names: string[]) {
  const art: ArtFileInfo = {
    path: artPath,
    metadata: { title: "Native static variants", tags: [], status: "ready" },
    variants: names.map((name, index) => ({ name, template: "<Host />", isDefault: index === 0 })),
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  const state: VirtualModuleState = {
    basePath: "/gallery",
    inlineArt: false,
    artFiles: new Map([[artPath, art]]),
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    getConfigRoot: () => "/repo",
    getScanRoots: () => ["/repo"],
    getVueVersion: () => 3,
    getServer: () => null,
    processArtFile: async () => {},
  };
  return state;
}

const names = [
  "Default",
  "State enabled",
  "State: enabled",
  "State%3A enabled",
  "100% ready",
  "日本語: enabled",
  "Malformed %QQ",
  'quoted "value": enabled',
];
for (const artPath of ["/repo/components/Host.art.vue", "C:/project:demo/Host.art.vue"]) {
  void test(`static preview IDs preserve authored names and art path ${artPath}`, () => {
    const state = context(artPath, names);
    const runtime = loadStaticRuntimeModule("\0musea-static-runtime", state.artFiles)!;
    const ids = [...runtime.matchAll(/import\(("(?:\\.|[^"\\])*")\)/g)].map(
      (match) => JSON.parse(match[1]) as string,
    );
    assert.equal(new Set(ids).size, names.length);
    assert.deepEqual(
      ids.slice(0, 2),
      names.slice(0, 2).map((name) => `virtual:musea-preview:${artPath}:${name}`),
    );
    for (const [index, name] of names.entries()) {
      assert.equal(ids[index], previewModuleId(artPath, name));
      assert.ok(runtime.includes(JSON.stringify(staticPreviewId(artPath, name))));
      const resolved = createResolveId(state)(ids[index]);
      assert.ok(resolved);
      const code = createLoad(state)(resolved);
      assert.ok(code, `missing preview for ${name}`);
      assert.ok(code.includes(`artModule[${JSON.stringify(toPascalCase(name))}]`));
      assert.ok(code.includes(`Mounted variant:', ${JSON.stringify(name)}`));
    }
  });
}

void test("dev preview middleware uses the same escaping and resolves literal percent names once", async () => {
  const state = context("/repo/Host.art.vue", names);
  const handlers = new Map<string, Connect.NextHandleFunction>();
  const requested: string[] = [];
  const devServer = {
    middlewares: {
      use: (route: string, handler: Connect.NextHandleFunction) => handlers.set(route, handler),
    },
    transformRequest: async (id: string) => {
      requested.push(id);
      const resolved = createResolveId(state)(id)!;
      return { code: createLoad(state)(resolved)! };
    },
  } as unknown as ViteDevServer;
  registerMiddleware(devServer, {
    ...state,
    scanRoots: [],
    devSessionToken: "unit",
    themeConfig: undefined,
  });
  const handler = handlers.get("/gallery/preview-module")!;
  assert.ok(handler);
  for (const name of names) {
    let body = "";
    const response = {
      setHeader() {},
      end: (value: string) => {
        body = value;
      },
    } as unknown as ServerResponse;
    const query = new URLSearchParams({ art: "/repo/Host.art.vue", variant: name });
    await Promise.resolve(
      handler({ url: `/?${query.toString()}` } as IncomingMessage, response, () =>
        assert.fail("unexpected next"),
      ),
    );
    assert.equal(requested.at(-1), previewModuleId("/repo/Host.art.vue", name));
    assert.ok(body.includes(`Mounted variant:', ${JSON.stringify(name)}`));
  }
});

void test("the public macOS red and ordinary controls retain their exact authored fixture bytes", async () => {
  const root = new URL(
    "../../../../tests/tooling/fixtures/musea/static-variant-name/",
    import.meta.url,
  );
  const files = {
    "Host.vue": "3087f1c9c435eccf08fedd48050fd124ab5574ac5c85f508b6b561ea5a0e599c",
    "Host.art.vue": "b13eb1f8d2379b0f91a9f2435f6c4305ee53945560d2bcb607ec90a201845dc0",
    "ordinary/Host.art.vue": "29b14c1956a20cf9dd22b74161d97b304bbb4cb9944ed6edc494c6b4a68d85fc",
  };
  for (const [file, expected] of Object.entries(files)) {
    const bytes = await readFile(new URL(file, root));
    assert.equal(crypto.createHash("sha256").update(bytes).digest("hex"), expected);
  }
  const ordinary = await readFile(new URL("ordinary/Host.art.vue", root), "utf8");
  const colon = await readFile(new URL("Host.art.vue", root), "utf8");
  assert.equal(ordinary.replace('name="State enabled"', 'name="State: enabled"'), colon);
});
