import fs from "node:fs";
import { test } from "node:test";
import "./test/setup.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { normalizeSnapshot, packageRoot, resolveFixturePath } from "./test/helpers.ts";

function createPlugin() {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
    },
    {
      framework: "rollup",
    },
  );
}

void test("raw hooks expose a dedicated CSS module path", async (t) => {
  const plugin = createPlugin();
  const filePath = resolveFixturePath("css-module", "App.vue");
  const source = fs.readFileSync(filePath, "utf8");
  const transformed = await plugin.transform?.call(
    {
      warn() {},
    } as never,
    source,
    filePath,
  );

  t.assert.ok(transformed && typeof transformed === "object");
  const match = transformed.code.match(/import \$style from "([^"]+)";/);
  t.assert.ok(match, "expected CSS module import in transformed output");

  const requestId = match[1];
  const resolvedId = await plugin.resolveId?.call({} as never, requestId, filePath, {
    isEntry: false,
  });
  t.assert.equal(typeof resolvedId, "string");

  const loaded = await plugin.load?.call({} as never, resolvedId as string);
  t.assert.ok(loaded && typeof loaded === "object");

  t.assert.snapshot(
    normalizeSnapshot(
      JSON.stringify(
        {
          requestId,
          resolvedId,
          loadedCss: loaded.code,
        },
        null,
        2,
      ),
    ),
  );
});
