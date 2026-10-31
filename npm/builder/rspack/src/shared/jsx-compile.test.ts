import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import { prepareOutputDir } from "../test/helpers.ts";
import { nativeVdomConsumers } from "../test/jsx-vdom-child-values.ts";
import assert from "node:assert/strict";
import { compileJsx } from "@vizejs/native";
import ts from "typescript";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { compileJsxModule } from "./compiler.ts";

const SOURCE = "const App = () => <div>{message}</div>;\nexport default App;\n";

void test("compileJsxModule includes the runtime-helper preamble in the emitted module", (t) => {
  // The VDOM render code references `_createElementBlock` / `_toDisplayString`,
  // so the emitted module must import them — the preamble is no longer dropped
  // (#1533).
  const { code } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom" });
  const source = "const A = () => <input disabled/>;";
  const native = compileJsxModule("/src/App.jsx", source, { jsxMode: "vdom" });
  const babel = compileJsxModule("/src/App.jsx", source, {
    jsxMode: "vdom",
    jsxCompat: "babel",
  });
  // Preserve complete native/Babel compatibility output alongside the authored
  // module, including their different boolean attribute semantics.
  const observed = { module: code, nativeCompat: native.code, babelCompat: babel.code };
  const directory = process.env.VIZE_RSPACK_JSX_CAPTURE ?? prepareOutputDir("jsx-vdom-module");
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    path.join(directory, "native-vdom-module.json"),
    JSON.stringify(
      {
        source: SOURCE,
        compatSource: source,
        observed,
        nativeDto: compileJsx(SOURCE, {
          filename: "/src/App.tsx",
          lang: "tsx",
          jsxMode: "vdom",
          sourceMap: false,
        }),
        sourceRevision: process.env.GITHUB_SHA ?? null,
      },
      null,
      2,
    ) + "\n",
  );
  const expected = nativeVdomConsumers.module;
  t.assert.deepEqual(
    { source: SOURCE, compatSource: source, observed },
    { source: expected.source, compatSource: expected.compatSource, observed: expected.observed },
  );
});

void test("compileJsxModule surfaces a v3 source map when sourceMap is requested", (t) => {
  const { map } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom", sourceMap: true });
  assert.equal(typeof map, "string", "a source map is surfaced when requested");
  const directory = process.env.VIZE_RSPACK_JSX_CAPTURE ?? prepareOutputDir("jsx-vdom-map");
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    path.join(directory, "native-vdom-map.json"),
    JSON.stringify(
      {
        source: SOURCE,
        observed: String(map),
        nativeDto: compileJsx(SOURCE, {
          filename: "/src/App.tsx",
          lang: "tsx",
          jsxMode: "vdom",
          sourceMap: true,
        }),
        sourceRevision: process.env.GITHUB_SHA ?? null,
      },
      null,
      2,
    ) + "\n",
  );
  t.assert.equal(SOURCE, nativeVdomConsumers.map.source);
  t.assert.equal(String(map), nativeVdomConsumers.map.observed);
});

void test("compileJsxModule omits the source map when sourceMap is off", () => {
  const { map } = compileJsxModule("/src/App.tsx", SOURCE, { jsxMode: "vdom", sourceMap: false });
  assert.equal(map, null, "no source map unless requested");
});

function assertAuthoredVapor(source: string, exports: string[]) {
  const filename = "/src/App.tsx";
  const outputs = [];
  for (const sourceMap of [false, true]) {
    const native = compileJsx(source, { filename, lang: "tsx", vapor: true, sourceMap });
    assert.deepEqual(native.errors, []);
    assert.deepEqual(native.warnings, []);
    assert.deepEqual(native.scopedStyles, []);
    const actual = compileJsxModule(filename, source, { vapor: true, sourceMap });
    assert.deepEqual(actual, { code: native.code, map: native.map ?? null, warnings: [] });
    outputs.push(actual.code);

    const module = ts.createSourceFile(
      filename,
      actual.code,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.JS,
    );
    assert.deepEqual(
      ts.transpileModule(actual.code, {
        fileName: filename,
        reportDiagnostics: true,
        compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
      }).diagnostics ?? [],
      [],
    );
    const defaultExports = module.statements.filter(ts.isExportAssignment);
    assert.deepEqual(
      defaultExports.map((entry) => entry.expression.getText(module)),
      exports,
    );
    assert.deepEqual(
      module.statements.filter(
        (statement) =>
          !ts.isExportAssignment(statement) &&
          (ts.isExportDeclaration(statement) ||
            (ts.canHaveModifiers(statement) &&
              ts
                .getModifiers(statement)
                ?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword))),
      ),
      [],
      "no standalone renderer export is invented",
    );
    const app = module.statements
      .filter(ts.isVariableStatement)
      .flatMap((statement) => [...statement.declarationList.declarations])
      .filter(
        (declaration) => ts.isIdentifier(declaration.name) && declaration.name.text === "App",
      );
    assert.equal(app.length, 1);
    let value = app[0].initializer;
    assert(value);
    while (ts.isParenthesizedExpression(value)) value = value.expression;
    assert(ts.isObjectLiteralExpression(value));
    assert.deepEqual(
      value.properties.map((property) =>
        "name" in property ? property.name?.getText(module) : undefined,
      ),
      ["__vapor", "name", "setup"],
    );
    const marker = value.properties[0];
    assert(ts.isPropertyAssignment(marker));
    assert.equal(marker.initializer.kind, ts.SyntaxKind.TrueKeyword);
    const setup = value.properties[2];
    assert(ts.isMethodDeclaration(setup));
    assert(setup.body?.statements.some(ts.isReturnStatement));
    if (sourceMap) {
      assert.equal(typeof actual.map, "string");
      const map = JSON.parse(actual.map!);
      assert.equal(map.version, 3);
      assert.deepEqual(map.sourcesContent, [source]);
      // The current native filename option selects the language. Its map
      // source identity is the documented virtual module for that language.
      assert.deepEqual(map.sources, ["module.tsx"]);
      const token = "const App =";
      const offset = actual.code.indexOf(token);
      assert(offset >= 0);
      const prefix = actual.code.slice(0, offset);
      const position = originalPositionFor(new TraceMap(map), {
        line: prefix.split("\n").length,
        column: prefix.length - prefix.lastIndexOf("\n") - 1,
      });
      assert.deepEqual(position, { source: "module.tsx", line: 1, column: 0, name: null });
    } else assert.equal(actual.map, null);
  }
  assert.equal(outputs[0], outputs[1], "map requests preserve every generated byte");
}

void test("compileJsxModule preserves private Vapor bindings and their original map", () => {
  // The original standalone input has no authored export. The module keeps
  // that private binding rather than inventing a standalone renderer export.
  assertAuthoredVapor("const App = () => <div>static</div>;", []);
});

void test("compileJsxModule retains the complete authored Vapor default export and map", () => {
  assert.equal(SOURCE, "const App = () => <div>{message}</div>;\nexport default App;\n");
  assertAuthoredVapor(SOURCE, ["App"]);
});

void test("compileJsxModule retains the exact guarded callable-alias refusal", () => {
  const source = "const App = () => <div/>; const alias = App; export default alias;";
  const refusal =
    "Vapor/SSR component-object bindings may be used only as JSX component tags or direct exports; local calls, callable aliases, introspection and value escapes require an explicit callable representation";
  const native = compileJsx(source, { filename: "/src/App.tsx", lang: "tsx", vapor: true });
  assert.deepEqual(native.errors, [refusal]);
  assert.deepEqual(native.warnings, []);
  assert.equal(native.code, "");
  assert.throws(() => compileJsxModule("/src/App.tsx", source, { vapor: true }), {
    name: "Error",
    message: `[vize] Compilation failed for /src/App.tsx:\n${refusal}`,
  });
});
