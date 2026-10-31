import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { SourceMap } from "node:module";
import { compileJsx } from "@vizejs/native";
import ts from "typescript";
import { fileURLToPath } from "node:url";
import type { VizePluginState } from "./state.ts";
import { transformHook } from "./load.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const baseState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: false,
  root: "/src",
  clientViteBase: "/",
  serverViteBase: "/",
  server: {} as never,
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

// A real .jsx file Vize fixture flows through the transform hook and compiles
// to render code through Vize (the headline acceptance of #1500).
const jsxFixturePath = path.resolve(__dirname, "..", "test", "fixtures", "jsx", "App.jsx");
const jsxFixtureSource = fs.readFileSync(jsxFixturePath, "utf8");

const jsxState: VizePluginState = {
  ...baseState,
  cache: new Map(),
  ssrCache: new Map(),
  mergedOptions: { vapor: false, include: /\.[jt]sx$/ },
};

const jsxTransform = await transformHook(jsxState, jsxFixtureSource, jsxFixturePath, {
  ssr: false,
});
assert.ok(
  jsxTransform && typeof jsxTransform === "object",
  "Vite should hand .jsx files to Vize and receive a transform result",
);
assert.match(
  jsxTransform.code,
  /_createElementVNode\("div", \{ class: "greeting" \}, \[\s+message\s+\]\)/,
  "VDOM .jsx transforms retain the complete authored expression as a raw child value",
);
assert.doesNotMatch(jsxTransform.code, /_toDisplayString|_openBlock|_createElementBlock/);
// The render code's runtime helpers must be imported (the preamble is no longer
// dropped, #1533).
assert.match(
  jsxTransform.code,
  /import \{[^}]*createElementVNode[^}]*\} from "vue"/,
  "VDOM .jsx output should carry the runtime-helper import preamble",
);
// Source maps are on in dev (isProduction === false), so a single-component
// .jsx transform surfaces a v3 map (parsed to the object form Vite expects) for
// the bundler to consume (#1533).
assert.ok(
  jsxTransform.map && typeof jsxTransform.map === "object",
  "JSX transform should surface a source map object in dev",
);
assert.equal(
  (jsxTransform.map as { version?: number }).version,
  3,
  "the surfaced JSX source map should be v3",
);

const jsxVaporState: VizePluginState = {
  ...jsxState,
  mergedOptions: { ...jsxState.mergedOptions, vapor: true },
};

const standaloneVaporSource = 'const App = () => <div class="greeting">standalone</div>;';
for (const input of [
  { source: standaloneVaporSource, exports: [] },
  { source: jsxFixtureSource, exports: ["App"] },
]) {
  const outputs = [];
  for (const isProduction of [false, true]) {
    const sourceMap = !isProduction;
    const native = compileJsx(input.source, {
      filename: jsxFixturePath,
      lang: "jsx",
      vapor: true,
      sourceMap,
    });
    const actual = await transformHook(
      { ...jsxVaporState, isProduction },
      input.source,
      jsxFixturePath,
      { ssr: false },
    );
    assert.ok(actual && typeof actual === "object");
    assert.deepEqual(actual, {
      code: native.code,
      map: sourceMap ? JSON.parse(native.map!) : null,
    });
    assert.deepEqual(native, {
      code: actual.code,
      ...(sourceMap ? { map: native.map } : {}),
      errors: [],
      warnings: [],
      scopedStyles: [],
    });
    outputs.push(actual.code);

    const module = ts.createSourceFile(
      jsxFixturePath,
      actual.code,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.JS,
    );
    assert.deepEqual(
      ts.transpileModule(actual.code, {
        fileName: jsxFixturePath,
        reportDiagnostics: true,
        compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
      }).diagnostics ?? [],
      [],
    );
    assert.deepEqual(
      module.statements
        .filter(ts.isExportAssignment)
        .map((statement) => statement.expression.getText(module)),
      input.exports,
      "the hook retains the authored export role of each original input",
    );
    assert.deepEqual(
      module.statements.filter(
        (statement) =>
          ts.isExportDeclaration(statement) ||
          (ts.canHaveModifiers(statement) &&
            ts
              .getModifiers(statement)
              ?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword)),
      ),
      [],
      "the hook does not invent a standalone renderer export",
    );
    const bindings = module.statements
      .filter(ts.isVariableStatement)
      .flatMap((statement) => [...statement.declarationList.declarations])
      .filter(
        (declaration) => ts.isIdentifier(declaration.name) && declaration.name.text === "App",
      );
    assert.equal(bindings.length, 1);
    let component = bindings[0].initializer;
    assert(component);
    while (ts.isParenthesizedExpression(component)) component = component.expression;
    assert(ts.isObjectLiteralExpression(component));
    assert.deepEqual(
      component.properties.map((property) =>
        "name" in property ? property.name?.getText(module) : undefined,
      ),
      ["__vapor", "name", "setup"],
    );
    const marker = component.properties[0];
    assert(ts.isPropertyAssignment(marker));
    assert.equal(marker.initializer.kind, ts.SyntaxKind.TrueKeyword);
    const setup = component.properties[2];
    assert(ts.isMethodDeclaration(setup));
    assert(setup.body?.statements.some(ts.isReturnStatement));

    if (sourceMap) {
      const map = actual.map;
      assert(map && typeof map === "object");
      const parsed = JSON.parse(native.map!);
      assert.equal(parsed.version, 3);
      assert.equal(parsed.file, "module.jsx");
      assert.deepEqual(parsed.sources, ["module.jsx"]);
      assert.deepEqual(parsed.sourcesContent, [input.source]);
      const offset = actual.code.indexOf("const App =");
      assert(offset >= 0);
      const prefix = actual.code.slice(0, offset);
      assert.deepEqual(
        new SourceMap(parsed).findEntry(
          prefix.split("\n").length - 1,
          prefix.length - prefix.lastIndexOf("\n") - 1,
        ),
        {
          generatedLine: prefix.split("\n").length - 1,
          generatedColumn: prefix.length - prefix.lastIndexOf("\n") - 1,
          originalSource: "module.jsx",
          originalLine: 0,
          originalColumn: 0,
          name: undefined,
        },
      );
    } else assert.equal(actual.map, null);
  }
  assert.equal(outputs[0], outputs[1], "production map policy keeps every compiled byte");
}

const callableAlias = "const App = () => <div/>; const alias = App; export default alias;";
const aliasRefusal =
  "Vapor/SSR component-object bindings may be used only as JSX component tags or direct exports; local calls, callable aliases, introspection and value escapes require an explicit callable representation";
assert.deepEqual(compileJsx(callableAlias, { lang: "jsx", vapor: true, sourceMap: true }), {
  code: "",
  errors: [aliasRefusal],
  warnings: [],
  scopedStyles: [],
});
await assert.rejects(
  () => transformHook(jsxVaporState, callableAlias, jsxFixturePath, { ssr: false }),
  {
    name: "VizeSfcCompileError",
    message: `[vize] Compilation failed in ${jsxFixturePath}:\n  - ${aliasRefusal}`,
  },
);

const tsxFsTransform = await transformHook(
  jsxState,
  `const T = () => <span class="t">x</span>;\nexport default T;\n`,
  "/@fs/abs/src/Widget.tsx",
  { ssr: false },
);
assert.ok(
  tsxFsTransform && typeof tsxFsTransform === "object",
  "/@fs-prefixed .tsx requests should be claimed and compiled through Vize",
);
assert.match(
  tsxFsTransform.code,
  /render|_createElementBlock/,
  "/@fs .tsx requests should emit Vize render code",
);

const nonJsxTransform = await transformHook(jsxState, "export default 1;", "/src/plain.ts", {
  ssr: false,
});
assert.equal(
  nonJsxTransform,
  null,
  "Plain non-JSX modules should bypass the Vize JSX transform path",
);

const rawJsxTransform = await transformHook(jsxState, jsxFixtureSource, `${jsxFixturePath}?raw`, {
  ssr: false,
});
assert.equal(
  rawJsxTransform,
  null,
  "?raw imports of a .jsx file should keep Vite's default asset handling, not Vize compilation",
);

// A .tsx component's `<style scoped>` becomes emitted CSS through the transform
// hook, mirroring the SFC plain-CSS inline-injection path (#1495, #1533).
const scopedTsxTransform = await transformHook(
  jsxState,
  `const Scoped = () => (\n  <div class="box">\n    <style scoped>{\`.box { color: red }\`}</style>\n  </div>\n);\nexport default Scoped;\n`,
  "/src/Scoped.tsx",
  { ssr: false },
);
assert.ok(
  scopedTsxTransform && typeof scopedTsxTransform === "object",
  "a .tsx with <style scoped> should produce a transform result",
);
assert.match(
  scopedTsxTransform.code,
  /__vize_css__/,
  "the transform hook should emit JSX <style scoped> CSS through the inline-style injection path",
);
assert.match(
  scopedTsxTransform.code,
  /\.box\[data-v-[0-9a-f]+\]/,
  "the emitted JSX CSS should apply the scope id to the .box selector",
);

console.log("✅ vite-plugin-vize load boundary tests passed!");
