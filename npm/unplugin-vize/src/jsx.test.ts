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
const SCOPED_APP_TSX = fs.readFileSync(resolveFixturePath("jsx", "ScopedApp.tsx"), "utf8");

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

void test("a .tsx transform carries the runtime-helper preamble and a source map", async (t) => {
  // The VDOM render code references `_createElementBlock` / `_toDisplayString`,
  // so the emitted module must import them (the preamble is no longer dropped);
  // with `sourceMap` on, a single-component module also surfaces a v3 map
  // (#1533).
  const id = resolveFixturePath("jsx", "App.tsx");
  const plugin = vizeUnplugin.raw(
    { isProduction: false, sourceMap: true, root: packageRoot },
    { framework: "rollup" },
  );
  const transform = plugin.transform;
  if (typeof transform !== "function") {
    throw new Error("plugin.transform is not a function");
  }
  const result = (await transform.call(
    { warn() {} } as never,
    "const App = () => <div>{message}</div>;\nexport default App;\n",
    id,
  )) as TransformResult | null;

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.match(
    result.code,
    /import \{[^}]*createElementBlock[^}]*\} from "vue"/,
    "the emitted module imports its runtime helpers",
  );
  t.assert.equal(typeof result.map, "string", "a source map is surfaced when requested");
  t.assert.match(String(result.map), /"version":\s*3/, "the surfaced source map is v3");
});

void test("a .tsx <style scoped> block emits scope-rewritten CSS through the plugin", async (t) => {
  // Mirrors the SFC plain-CSS path: a JSX/TSX `<style scoped>` block becomes
  // emitted CSS the bundler picks up, with the `data-v-<hash>` scope id already
  // applied to the selectors (#1495, #1533).
  const id = resolveFixturePath("jsx", "ScopedApp.tsx");
  const { result, warnings } = await runTransform(false, SCOPED_APP_TSX, id);

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // The plugin emits the CSS through the shared inline-injection path, so the
  // compiled module carries the rewritten CSS as a string constant.
  t.assert.match(result.code, /__vize_css__/, "emits CSS through the inline-style injection path");
  const scopeMatch = result.code.match(/data-v-[0-9a-f]+/);
  t.assert.ok(scopeMatch, "the emitted CSS carries a data-v- scope id");
  t.assert.ok(
    result.code.includes(`.box[${scopeMatch[0]}]`),
    "the emitted CSS applies the scope id to the .box selector",
  );
  // The same scope id is applied to the rendered element (createElement path).
  t.assert.ok(result.code.includes(scopeMatch[0]), "the render output and CSS share one scope id");
});
