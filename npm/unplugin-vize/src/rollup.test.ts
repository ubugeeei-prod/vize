import { test } from "node:test";
import { rollup } from "rollup";
import "./test/setup.ts";
import vize from "./rollup.ts";
import { normalizeSnapshot, resolveFixturePath } from "./test/helpers.ts";

void test("rollup bundles a basic SFC", async (t) => {
  const bundle = await rollup({
    input: resolveFixturePath("basic", "entry.ts"),
    external: ["vue"],
    plugins: [vize({ isProduction: true })],
  });

  const output = await bundle.generate({
    format: "esm",
  });

  const chunk = output.output.find((item) => item.type === "chunk");
  t.assert.ok(chunk && chunk.type === "chunk");
  t.assert.match(chunk.code, /Hello from Vize unplugin/);
  t.assert.snapshot(normalizeSnapshot(chunk.code));
});
