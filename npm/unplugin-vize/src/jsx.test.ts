import fs from "node:fs";
import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot, resolveFixturePath } from "./test/helpers.ts";

interface TransformResult {
  code: string;
  map?: unknown;
}

function createPlugin(vapor: boolean, jsxMode?: "vdom" | "vapor") {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
      vapor,
      jsxMode,
    },
    {
      framework: "rollup",
    },
  );
}

async function runTransform(
  vapor: boolean,
  source: string,
  id: string,
  jsxMode?: "vdom" | "vapor",
): Promise<{ result: TransformResult | null; warnings: string[] }> {
  const plugin = createPlugin(vapor, jsxMode);

  const transformInclude = plugin.transformInclude;
  if (typeof transformInclude === "function") {
    const included = transformInclude.call({} as never, id);
    if (!included) {
      throw new Error(`transformInclude rejected ${id}`);
    }
  }

  const transform = plugin.transform;
  if (typeof transform !== "function") {
    throw new Error("plugin.transform is not a function");
  }
  const warnings: string[] = [];
  const result = (await transform.call(
    {
      warn(message: string) {
        warnings.push(message);
      },
    } as never,
    source,
    id,
  )) as TransformResult | null;
  return { result, warnings };
}

const APP_JSX = fs.readFileSync(resolveFixturePath("jsx", "App.jsx"), "utf8");

void test("an .jsx fixture flows through the transform and emits vdom render code", async (t) => {
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(false, APP_JSX, id);

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.equal(typeof result.code, "string");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // The default (vdom) JSX backend emits the block-based element factory.
  t.assert.match(
    result.code,
    /_createElementBlock\("div"/,
    "vdom JSX output uses the element block factory",
  );
});

void test("vapor:true compiles the .jsx fixture to vapor template output", async (t) => {
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(true, APP_JSX, id);

  t.assert.ok(result && typeof result === "object", "vapor transform returns a result object");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // Vapor emits a hoisted static template rather than a vdom block call.
  t.assert.match(result.code, /template\(/, "vapor JSX output uses the static template helper");
  t.assert.doesNotMatch(
    result.code,
    /_createElementBlock/,
    "vapor JSX output does not use the vdom element block factory",
  );
});

void test("jsxMode:'vapor' selects the vapor default and wins over vapor:false", async (t) => {
  // The explicit `jsxMode` option mirrors `compiler.jsxMode` and takes
  // precedence over the legacy `vapor` boolean (here left false).
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(false, APP_JSX, id, "vapor");

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");
  t.assert.match(result.code, /template\(/, "jsxMode:'vapor' produces vapor template output");
});

void test("jsxMode:'vdom' keeps the vdom default even when vapor:true", async (t) => {
  // Conversely, `jsxMode:'vdom'` overrides a `vapor:true` legacy toggle.
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(true, APP_JSX, id, "vdom");

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");
  t.assert.match(
    result.code,
    /_createElementBlock\("div"/,
    "jsxMode:'vdom' produces vdom element-block output",
  );
});
