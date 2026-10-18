// #9493: one pinned official compiler/runtime owner for whole-SFC key scopes.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Script } from "node:vm";

type Probe = {
  code: string;
  production: boolean;
  unicodeKeyScope: {
    source: string;
    official: boolean;
    samples: { args: unknown[]; value: unknown }[];
  };
};

export async function prepareUnicodeKeyScope(input: Probe) {
  const fromTests = createRequire(new URL("../../package.json", import.meta.url));
  const fromVue = createRequire(fromTests.resolve("vue-ssr-css-vars-oracle/package.json"));
  const compiler = fromVue("@vue/compiler-sfc");
  const fromCompiler = createRequire(fromVue.resolve("@vue/compiler-sfc"));
  const vaporVersion = fromCompiler("@vue/compiler-vapor/package.json").version;
  const version = "3.6.0-rc.10";
  assert.equal(compiler.version, version);
  assert.equal(vaporVersion, version);
  const runtimePath = fromVue.resolve(
    `./dist/vue.runtime-with-vapor.esm-browser${input.production ? ".prod" : ""}.js`,
  );
  // Preserve Vue's genuine development banner outside the JSON result stream.
  const info = console.info;
  console.info = (...values) => process.stderr.write(values.join(" ") + "\n");
  let vue;
  try {
    vue = await import(pathToFileURL(runtimePath).href);
  } finally {
    console.info = info;
  }
  assert.equal(vue.version, version);
  const { source, official, samples } = input.unicodeKeyScope;
  let code = input.code;
  if (official) {
    const parsed = compiler.parse(source, { filename: "App.vue" });
    assert.deepEqual(parsed.errors, []);
    code = compiler.compileScript(parsed.descriptor, {
      id: "probe",
      inlineTemplate: true,
      isProd: input.production,
    }).content;
  }
  const { parse } = fromTests("@babel/parser") as typeof import("@babel/parser");
  const aliases = new Set<string>();
  const keys: { code: string; params: unknown; body: unknown }[] = [];
  const record = (value: unknown): value is Record<string, unknown> =>
    value !== null && typeof value === "object" && !Array.isArray(value);
  // The official parser checks the complete, unmodified module. Runtime guards
  // retain AST ownership without an untyped Babel plugin or a types-only shim.
  const ast = parse(code, { sourceType: "module" });
  for (const statement of ast.program.body) {
    if (statement.type !== "ImportDeclaration" || statement.source.value !== "vue") continue;
    for (const specifier of statement.specifiers)
      if (
        specifier.type === "ImportSpecifier" &&
        specifier.imported.type === "Identifier" &&
        specifier.imported.name === "createFor"
      )
        aliases.add(specifier.local.name);
  }
  const visit = (value: unknown): void => {
    if (Array.isArray(value)) {
      for (const child of value) visit(child);
      return;
    }
    if (!record(value)) return;
    if (
      value.type === "CallExpression" &&
      record(value.callee) &&
      value.callee.type === "Identifier" &&
      typeof value.callee.name === "string" &&
      aliases.has(value.callee.name)
    ) {
      assert.ok(Array.isArray(value.arguments));
      const key: unknown = value.arguments[2];
      assert.ok(
        record(key) && key.type === "ArrowFunctionExpression",
        "every fixture has an actual key function",
      );
      assert.ok(typeof key.start === "number" && typeof key.end === "number");
      keys.push({ code: code.slice(key.start, key.end), params: key.params, body: key.body });
    }
    for (const child of Object.values(value)) visit(child);
  };
  visit(ast);
  process.stderr.write(
    JSON.stringify({
      unicodeKeyScope: "module",
      authority: official ? "official-stock" : "fresh-native",
      source,
      code,
      compilerVersion: compiler.version,
      vaporVersion,
      runtimePath,
      runtimeVersion: vue.version,
      production: input.production,
      keyFunctions: keys,
    }) + "\n",
  );
  assert.equal(keys.length, samples.length, "complete key-function denominator");
  const values = keys.map((key, index) => {
    // Standalone invocation makes a key's dependency on a sibling render
    // callback fail with its genuine ReferenceError, before any DOM assertion.
    const keyFunction: unknown = new Script(`(${key.code})`).runInNewContext();
    assert.ok(typeof keyFunction === "function");
    const value = keyFunction(...samples[index].args);
    assert.deepEqual(value, samples[index].value);
    return value;
  });
  process.stderr.write(JSON.stringify({ unicodeKeyScope: "key-values", values }) + "\n");
  return { vue, code };
}
