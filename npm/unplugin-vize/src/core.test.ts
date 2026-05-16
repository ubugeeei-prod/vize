import fs from "node:fs";
import path from "node:path";
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

void test("raw transform compiles a basic SFC", async (t) => {
  const plugin = createPlugin();
  const filePath = resolveFixturePath("basic", "App.vue");
  const source = fs.readFileSync(filePath, "utf8");
  const warnings: string[] = [];
  const result = await plugin.transform?.call(
    {
      warn(message: string) {
        warnings.push(message);
      },
    } as never,
    source,
    filePath,
  );

  t.assert.ok(result && typeof result === "object");
  t.assert.deepStrictEqual(warnings, []);
  t.assert.snapshot(
    normalizeSnapshot(
      JSON.stringify(
        {
          filePath: path.relative(packageRoot, filePath),
          code: result.code,
        },
        null,
        2,
      ),
    ),
  );
});
