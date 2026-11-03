import fs from "node:fs";
import { test } from "node:test";
import { compileJsx } from "@vizejs/native";
import { build as esbuild } from "esbuild";
import { rollup } from "rollup";
import ts from "typescript";
import vizeEsbuild from "./esbuild.ts";
import vizeRollup from "./rollup.ts";
import { cssPipeline } from "./test/css-pipeline.ts";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot, resolveFixturePath } from "./test/helpers.ts";

interface TransformResult {
  code: string;
  map?: unknown;
}

function createPlugin(
  vapor: boolean,
  jsxMode?: "vdom" | "vapor",
  jsxCompat?: "native" | "babel",
  sourceMap?: boolean,
) {
  return vizeUnplugin.raw(
    {
      isProduction: true,
      root: packageRoot,
      vapor,
      jsxMode,
      jsxCompat,
      sourceMap,
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
  jsxCompat?: "native" | "babel",
  sourceMap?: boolean,
): Promise<{ result: TransformResult | null; warnings: string[] }> {
  const plugin = createPlugin(vapor, jsxMode, jsxCompat, sourceMap);

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
const STANDALONE_VAPOR_JSX = 'const App = () => <div class="greeting">standalone</div>;';
const SCOPED_APP_TSX = fs.readFileSync(resolveFixturePath("jsx", "ScopedApp.tsx"), "utf8");

void test("an .jsx fixture flows through the transform and emits vdom render code", async (t) => {
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(false, APP_JSX, id);

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.equal(typeof result.code, "string");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  // Opaque JSX values stay raw children of a normal VNode so Vue normalizes every update.
  t.assert.match(
    result.code,
    /_createElementVNode\("div"/,
    "vdom JSX output uses a normal VNode for opaque child values",
  );
  t.assert.match(
    result.code,
    /\[\s*message\s*\]/,
    "the authored message remains one raw child array entry",
  );
  t.assert.doesNotMatch(
    result.code,
    /\b(?:_toDisplayString|_openBlock|_createElementBlock)\b/,
    "raw children cannot retain text or block-only update semantics",
  );
});

void test("vapor:true compiles a standalone .jsx renderer to vapor template output", async (t) => {
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(true, STANDALONE_VAPOR_JSX, id);

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
  const { result, warnings } = await runTransform(false, STANDALONE_VAPOR_JSX, id, "vapor");

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.ok(result.code.length > 0, "emitted code is non-empty");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");
  t.assert.match(result.code, /template\(/, "jsxMode:'vapor' produces vapor template output");
});

void test("Vapor authored exports retain the complete native module and requested map", async (t) => {
  // Keep the original exported fixture that formerly required an explicit
  // refusal. This is the adapter contract; actual renderer execution and
  // backend-origin validation have separate native and consumer tests.
  t.assert.equal(
    APP_JSX,
    'const App = () => <div class="greeting">{message}</div>;\n\nexport default App;\n',
  );
  const id = resolveFixturePath("jsx", "App.jsx");
  const outputs = [];
  for (const sourceMap of [false, true]) {
    const native = compileJsx(APP_JSX, { filename: id, lang: "jsx", vapor: true, sourceMap });
    t.assert.deepEqual(native.errors, []);
    t.assert.deepEqual(native.warnings, []);
    t.assert.deepEqual(native.scopedStyles, []);
    const actual = await runTransform(true, APP_JSX, id, undefined, undefined, sourceMap);
    t.assert.ok(actual.result);
    t.assert.deepEqual(actual, {
      result: { code: native.code, map: native.map ?? null },
      warnings: [],
    });
    outputs.push(actual.result.code);

    const module = ts.createSourceFile(
      id,
      actual.result.code,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.JS,
    );
    t.assert.deepEqual(
      ts.transpileModule(actual.result.code, {
        fileName: id,
        reportDiagnostics: true,
        compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
      }).diagnostics ?? [],
      [],
    );
    const exports = module.statements.filter(ts.isExportAssignment);
    t.assert.equal(exports.length, 1);
    t.assert.equal(exports[0].isExportEquals, undefined);
    t.assert.ok(ts.isIdentifier(exports[0].expression));
    t.assert.equal(exports[0].expression.text, "App");
    const variables = module.statements
      .filter(ts.isVariableStatement)
      .flatMap((statement) => [...statement.declarationList.declarations]);
    const app = variables.filter(
      (declaration) => ts.isIdentifier(declaration.name) && declaration.name.text === "App",
    );
    t.assert.equal(app.length, 1);
    let component = app[0].initializer;
    t.assert.ok(component);
    while (ts.isParenthesizedExpression(component)) component = component.expression;
    t.assert.ok(ts.isObjectLiteralExpression(component));
    t.assert.deepEqual(
      component.properties.map((property) =>
        "name" in property ? property.name?.getText(module) : undefined,
      ),
      ["__vapor", "name", "setup"],
    );
    const marker = component.properties[0];
    t.assert.ok(ts.isPropertyAssignment(marker));
    t.assert.equal(marker.initializer.kind, ts.SyntaxKind.TrueKeyword);
    const name = component.properties[1];
    t.assert.ok(ts.isPropertyAssignment(name));
    t.assert.ok(ts.isStringLiteral(name.initializer));
    t.assert.equal(name.initializer.text, "App");
    const setup = component.properties[2];
    t.assert.ok(ts.isMethodDeclaration(setup));
    t.assert.equal(setup.parameters.length, 0);
    t.assert.ok(setup.body);
    t.assert.ok(setup.body.statements.some(ts.isReturnStatement));
    t.assert.ok(actual.result.code.endsWith(";\n\nexport default App;\n"));
    const remainingJsx: ts.Node[] = [];
    const visit = (node: ts.Node) => {
      if (ts.isJsxElement(node) || ts.isJsxSelfClosingElement(node) || ts.isJsxFragment(node))
        remainingJsx.push(node);
      ts.forEachChild(node, visit);
    };
    visit(module);
    t.assert.deepEqual(remainingJsx, []);

    if (sourceMap) {
      t.assert.equal(typeof native.map, "string");
      const map = JSON.parse(native.map!);
      t.assert.equal(map.version, 3);
      t.assert.deepEqual(map.sourcesContent, [APP_JSX]);
      t.assert.equal(map.sources.length, 1);
      t.assert.ok(map.mappings.length > 0);
    } else t.assert.equal(actual.result.map, null);
  }
  t.assert.equal(outputs[0], outputs[1], "requesting maps preserves every generated byte");
});

void test("jsxMode:'vdom' keeps the vdom default even when vapor:true", async (t) => {
  // Conversely, `jsxMode:'vdom'` overrides a `vapor:true` legacy toggle.
  const id = resolveFixturePath("jsx", "App.jsx");
  const { result, warnings } = await runTransform(true, APP_JSX, id, "vdom");

  t.assert.ok(result && typeof result === "object", "transform returns a result object");
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");
  t.assert.match(
    result.code,
    /_createElementVNode\("div"/,
    "jsxMode:'vdom' preserves raw values in normal VDOM children",
  );
  t.assert.match(
    result.code,
    /\[\s*message\s*\]/,
    "the authored message remains one raw child array entry",
  );
  t.assert.doesNotMatch(
    result.code,
    /\b(?:_toDisplayString|_openBlock|_createElementBlock)\b/,
    "raw children cannot retain text or block-only update semantics",
  );
});

void test("jsxCompat:'babel' reaches the unplugin compiler with complete output", async (t) => {
  const source = "const A = () => <input disabled/>;";
  const id = resolveFixturePath("jsx", "Compat.jsx");
  const native = await runTransform(false, source, id);
  const babel = await runTransform(false, source, id, undefined, "babel");

  t.assert.equal(
    native.result?.code,
    'import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"\n' +
      "\nconst A = () => (() => {\nreturn ((_ctx, _cache) => {\n" +
      '  return (_openBlock(), _createElementBlock("input", { disabled: "" }))\n' +
      "})(undefined, [])\n})();",
  );
  t.assert.equal(
    babel.result?.code,
    'import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"\n' +
      "\nconst A = () => (() => {\nreturn ((_ctx, _cache) => {\n" +
      '  return (_openBlock(), _createElementBlock("input", { disabled: true }))\n' +
      "})(undefined, [])\n})();",
  );
  t.assert.deepEqual(native.warnings, []);
  t.assert.deepEqual(babel.warnings, []);
});

void test("a .tsx transform carries the runtime-helper preamble and a source map", async (t) => {
  // Opaque VDOM children use the normal element factory, so its runtime helper
  // must stay in the complete module preamble;
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
    /import \{[^}]*createElementVNode[^}]*\} from "vue"/,
    "the emitted module imports its runtime helpers",
  );
  t.assert.match(
    result.code,
    /\[\s*message\s*\]/,
    "the authored message remains one raw child array entry",
  );
  t.assert.doesNotMatch(
    result.code,
    /\b(?:_toDisplayString|_openBlock|_createElementBlock)\b/,
    "raw children cannot retain text or block-only update semantics",
  );
  t.assert.equal(typeof result.map, "string", "a source map is surfaced when requested");
  t.assert.match(String(result.map), /"version":\s*3/, "the surfaced source map is v3");
});

void test("a .tsx <style scoped> block emits scope-rewritten CSS through the plugin", async (t) => {
  // Mirrors the SFC path: a JSX/TSX `<style scoped>` block is a style module
  // whose CSS already carries the `data-v-<hash>` scope id (#1495, #1533, #11520).
  const id = resolveFixturePath("jsx", "ScopedApp.tsx");
  const plugin = createPlugin(false);
  const warnings: string[] = [];
  const result = (await (plugin.transform as (...args: unknown[]) => Promise<TransformResult>).call(
    { warn: (message: string) => warnings.push(message) },
    SCOPED_APP_TSX,
    id,
  )) as TransformResult;
  t.assert.deepStrictEqual(warnings, [], "no warnings are emitted");

  const styleId = /^import "([^"]+)";/.exec(result.code)?.[1] ?? "";
  t.assert.equal(styleId, `${id}?vue&type=style&index=0&lang.css`);
  t.assert.doesNotMatch(result.code, /__vize_css__/);
  const loaded = (await (plugin.load as (id: string) => Promise<{ code: string }>).call(
    {},
    styleId,
  )) as { code: string };
  const scope = /data-v-[0-9a-f]+/.exec(loaded.code)?.[0];
  t.assert.ok(scope, loaded.code);
  t.assert.ok(loaded.code.includes(`.box[${scope}]`), "the scope id is on the .box selector");
  // The same scope id is applied to the rendered element.
  t.assert.ok(result.code.includes(scope), "the render output and CSS share one scope id");
});

// #11520: a JSX `<style scoped>` block is a style module for the host's CSS
// pipeline, like an SFC block, and is injected only where nothing bundles CSS.
void test("a .tsx <style scoped> block reaches the host CSS pipeline", async (t) => {
  const css = cssPipeline();
  const bundle = await rollup({
    input: resolveFixturePath("jsx", "ScopedApp.tsx"),
    external: ["vue"],
    plugins: [vizeRollup({ isProduction: true, root: packageRoot }), css],
  });
  const { output } = await bundle.generate({ format: "esm" });
  const js = output.find((item) => item.type === "chunk")?.code ?? "";
  t.assert.doesNotMatch(js, /__vize_css__|document\.createElement\("style"\)/);
  const scope = /data-v-[0-9a-f]+/.exec(js)?.[0];
  t.assert.ok(scope, js);
  const asset = output.find((item) => item.fileName === "style.css");
  t.assert.ok(asset && asset.type === "asset");
  t.assert.match(
    String(asset.source),
    new RegExp(`^\\.box\\[${scope}\\]\\s*\\{\\s*color: red;?\\s*\\}`),
  );

  // Negative controls: esbuild without `bundle` still injects the CSS, and
  // Rollup without a CSS plugin fails on the style module.
  const unbundled = await esbuild({
    entryPoints: [resolveFixturePath("jsx", "ScopedApp.tsx")],
    bundle: false,
    write: false,
    format: "esm",
    outdir: "out",
    plugins: [vizeEsbuild({ isProduction: true, root: packageRoot })],
  });
  t.assert.match(unbundled.outputFiles[0].text, /__vize_css__ = "\.box\[data-v-[0-9a-f]+\]/);
  await t.assert.rejects(
    rollup({
      input: resolveFixturePath("jsx", "ScopedApp.tsx"),
      external: ["vue"],
      plugins: [vizeRollup({ isProduction: true, root: packageRoot })],
      onwarn() {},
    }),
    /ScopedApp\.tsx\?vue&type=style&index=0&lang\.css \(\d+:\d+\): Expression expected/,
  );
});
