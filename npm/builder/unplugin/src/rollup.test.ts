import { test } from "node:test";
import { rollup } from "rollup";
import "./test/setup.ts";
import vize from "./rollup.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { cssPipeline } from "./test/css-pipeline.ts";
import { normalizeSnapshot, resolveFixturePath } from "./test/helpers.ts";

void test("rollup bundles a basic SFC", async (t) => {
  const css = cssPipeline();
  const bundle = await rollup({
    input: resolveFixturePath("basic", "entry.ts"),
    external: ["vue"],
    plugins: [vize({ isProduction: true }), css],
  });

  const output = await bundle.generate({
    format: "esm",
  });

  const chunk = output.output.find((item) => item.type === "chunk");
  t.assert.ok(chunk && chunk.type === "chunk");
  t.assert.match(chunk.code, /Hello from Vize unplugin/);
  t.assert.doesNotMatch(chunk.code, /document\.createElement\("style"\)/);
  const asset = output.output.find((item) => item.fileName === "style.css");
  t.assert.ok(asset && asset.type === "asset");
  t.assert.match(String(asset.source), /\.root\[data-v-[0-9a-f]{8}\]\s*\{\s*color: tomato;/);
  t.assert.snapshot(normalizeSnapshot(chunk.code));
});

void test("rollup without a CSS plugin fails on the style block and names the fix", async (t) => {
  const warnings: string[] = [];
  await t.assert.rejects(
    rollup({
      input: resolveFixturePath("basic", "entry.ts"),
      external: ["vue"],
      plugins: [vize({ isProduction: true })],
      onwarn: (warning) => warnings.push(warning.message),
    }),
    /App\.vue\?vue&type=style&index=0&scoped=data-v-[0-9a-f]{8}&lang\.css \(\d+:\d+\): Expression expected/,
  );
  t.assert.equal(warnings.length, 1);
  t.assert.match(warnings[0], /stylesheet modules for the bundler's CSS pipeline/);
  t.assert.match(warnings[0], /rollup-plugin-postcss/);
});

void test("the CSS pipeline hint fires only for an error on a style module", async (t) => {
  const plugin = vizeUnplugin.raw({ isProduction: true }, { framework: "rollup" }) as unknown as {
    load(id: string): Promise<unknown>;
    rollup: {
      buildStart(): void;
      buildEnd(this: { warn(message: string): void }, error?: object): void;
    };
  };
  const styleId = "/p/App.vue?vue&type=style&index=0&lang.css";
  const hints = (error?: object) => {
    const warnings: string[] = [];
    plugin.rollup.buildEnd.call({ warn: (message) => warnings.push(message) }, error);
    return warnings.length;
  };
  plugin.rollup.buildStart();
  // No style module loaded yet: even Rolldown's CSS error is not about one.
  t.assert.equal(hints({ errors: [{ code: "UNSUPPORTED_FEATURE" }] }), 0);
  await plugin.load(styleId);
  t.assert.equal(hints(), 0);
  t.assert.equal(hints({ id: "/p/entry.ts", code: "PARSE_ERROR" }), 0);
  t.assert.equal(hints({ errors: [{ id: "/p/entry.ts", code: "PARSE_ERROR" }] }), 0);
  t.assert.equal(hints({ id: styleId, code: "PARSE_ERROR" }), 1);
  t.assert.equal(hints({ errors: [{ id: styleId, code: "PLUGIN_ERROR" }] }), 1);
  t.assert.equal(hints({ errors: [{ id: ".b{color:blue}", code: "UNSUPPORTED_FEATURE" }] }), 1);
});
