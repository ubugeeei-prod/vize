import assert from "node:assert/strict";
import { SourceMap } from "node:module";
import { compileJsx } from "@vizejs/native";
import ts from "typescript";
import { compileJsxModule } from "./compiler.ts";

const jsxSource = `const App = () => <div class="greeting">{message}</div>;\nexport default App;\n`;

const jsxVdomCompiled = compileJsxModule("/src/App.jsx", jsxSource, { vapor: false });
assert.match(
  jsxVdomCompiled.code,
  /_createElementBlock\("div"/,
  "VDOM JSX compilation should emit the element block factory",
);
assert.deepEqual(
  jsxVdomCompiled.warnings,
  [],
  "VDOM JSX compilation of a valid component should emit no warnings",
);

const standaloneVaporSource = 'const App = () => <div class="greeting">standalone</div>;';
const jsxVaporCompiled = compileJsxModule("/src/App.jsx", standaloneVaporSource, { vapor: true });
assert.match(
  jsxVaporCompiled.code,
  /_template\(/,
  "Vapor JSX compilation should hoist a static template",
);
assert.doesNotMatch(
  jsxVaporCompiled.code,
  /_createElementBlock/,
  "Vapor JSX compilation should not emit the VDOM element block factory",
);
// Preserve the original authored export as the input. Native module output
// retains that binding rather than replacing it with a standalone renderer.
const authoredVaporOutputs = [];
for (const sourceMap of [false, true]) {
  const native = compileJsx(jsxSource, {
    filename: "/src/App.jsx",
    lang: "jsx",
    vapor: true,
    sourceMap,
  });
  const actual = compileJsxModule("/src/App.jsx", jsxSource, { vapor: true, sourceMap });
  assert.deepEqual(actual, {
    code: native.code,
    map: native.map ?? null,
    warnings: [],
  });
  assert.deepEqual(native, {
    code: actual.code,
    ...(sourceMap ? { map: actual.map } : {}),
    errors: [],
    warnings: [],
    scopedStyles: [],
  });
  authoredVaporOutputs.push(actual.code);

  const module = ts.createSourceFile(
    "/src/App.jsx",
    actual.code,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.JS,
  );
  assert.deepEqual(
    ts.transpileModule(actual.code, {
      fileName: "/src/App.jsx",
      reportDiagnostics: true,
      compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
    }).diagnostics ?? [],
    [],
  );
  assert.deepEqual(
    module.statements
      .filter(ts.isExportAssignment)
      .map((statement) => statement.expression.getText(module)),
    ["App"],
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
    "no standalone renderer export is invented",
  );
  const bindings = module.statements
    .filter(ts.isVariableStatement)
    .flatMap((statement) => [...statement.declarationList.declarations])
    .filter((declaration) => ts.isIdentifier(declaration.name) && declaration.name.text === "App");
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
  const name = component.properties[1];
  assert(ts.isPropertyAssignment(name) && ts.isStringLiteral(name.initializer));
  assert.equal(name.initializer.text, "App");
  const setup = component.properties[2];
  assert(ts.isMethodDeclaration(setup));
  assert(setup.body?.statements.some(ts.isReturnStatement));

  if (sourceMap) {
    assert.equal(typeof actual.map, "string");
    const map = JSON.parse(actual.map!);
    assert.equal(map.version, 3);
    // The native filename option selects the language; JSX maps use this
    // documented virtual source, with the complete untouched authored content.
    assert.equal(map.file, "module.jsx");
    assert.deepEqual(map.sources, ["module.jsx"]);
    assert.deepEqual(map.sourcesContent, [jsxSource]);
    const offset = actual.code.indexOf("const App =");
    assert(offset >= 0);
    const prefix = actual.code.slice(0, offset);
    const position = new SourceMap(map).findEntry(
      prefix.split("\n").length - 1,
      prefix.length - prefix.lastIndexOf("\n") - 1,
    );
    assert.deepEqual(position, {
      generatedLine: prefix.split("\n").length - 1,
      generatedColumn: prefix.length - prefix.lastIndexOf("\n") - 1,
      originalSource: "module.jsx",
      originalLine: 0,
      originalColumn: 0,
      name: undefined,
    });
  } else assert.equal(actual.map, null);
}
assert.equal(
  authoredVaporOutputs[0],
  authoredVaporOutputs[1],
  "map requests retain every generated module byte",
);

const callableAlias = "const App = () => <div/>; const alias = App; export default alias;";
const aliasRefusal =
  "Vapor/SSR component-object bindings may be used only as JSX component tags or direct exports; local calls, callable aliases, introspection and value escapes require an explicit callable representation";
assert.deepEqual(compileJsx(callableAlias, { lang: "jsx", vapor: true }), {
  code: "",
  errors: [aliasRefusal],
  warnings: [],
  scopedStyles: [],
});
assert.throws(() => compileJsxModule("/src/Alias.jsx", callableAlias, { vapor: true }), {
  name: "VizeSfcCompileError",
  message: `[vize] Compilation failed in /src/Alias.jsx:\n  - ${aliasRefusal}`,
});

const tsxCompiled = compileJsxModule(
  "/src/Typed.tsx",
  `const Typed = (props: { label: string }) => <span>{props.label}</span>;\nexport default Typed;\n`,
  { vapor: false },
);
assert.match(
  tsxCompiled.code,
  /_createElementBlock\("span"|_createBlock|render/,
  "TSX compilation should infer the tsx language from the .tsx filename and emit render code",
);

assert.throws(
  () => compileJsxModule("/src/Broken.jsx", `const Broken = () => <div>{;`, { vapor: false }),
  /Compilation failed in \/src\/Broken\.jsx/,
  "JSX compilation errors should fail the Vite plugin instead of emitting broken code",
);

// A `.tsx` component with `<style scoped>` emits its (scope-rewritten) CSS
// through the same inline-injection path plain SFC `<style>` blocks use, with
// the `data-v-<hash>` scope id already applied (#1495, #1533).
const scopedJsxSource = `const Scoped = () => (\n  <div class="box">\n    <style scoped>{\`.box { color: red }\`}</style>\n  </div>\n);\nexport default Scoped;\n`;
const scopedJsxCompiled = compileJsxModule("/src/Scoped.tsx", scopedJsxSource, { vapor: false });
assert.match(
  scopedJsxCompiled.code,
  /__vize_css__/,
  "JSX <style scoped> CSS should be emitted through the inline-style injection path",
);
const jsxScopeMatch = scopedJsxCompiled.code.match(/data-v-[0-9a-f]+/);
assert.ok(jsxScopeMatch, "the emitted JSX CSS should carry a data-v- scope id");
assert.ok(
  scopedJsxCompiled.code.includes(`.box[${jsxScopeMatch[0]}]`),
  "the emitted JSX CSS should apply the scope id to the .box selector",
);

// Under SSR the inline `document`-based injection is skipped, mirroring the SFC
// inline-CSS path (the scope id still rides the render output).
const scopedJsxSsr = compileJsxModule("/src/Scoped.tsx", scopedJsxSource, {
  vapor: false,
  ssr: true,
});
assert.doesNotMatch(
  scopedJsxSsr.code,
  /__vize_css__/,
  "SSR JSX compilation should not emit the client-only inline-style injection",
);

console.log("✅ vite-plugin-vize JSX compiler tests passed!");
