// #7972: whole source-built SFC modules versus independently pinned Vue 3.5.38.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { transformSync } from "@babel/core";
import { parse } from "@babel/parser";

const fixtureRoot = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(readFileSync(new URL("setup-scope-await.manifest.json", fixtureRoot)));
const pin = manifest.cases[0];
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
for (const artifact of [
  ...pin.inputs.files.map((file) => ({ ...file, path: pin.inputs.root + "/" + file.path })),
  pin.reference,
])
  assert.equal(sha(readFileSync(new URL(artifact.path, fixtureRoot))), artifact.sha256);
const reported = readFileSync(new URL("setup-scope-await/Probe.vue.txt", fixtureRoot), "utf8");
const controls = JSON.parse(readFileSync(new URL("setup-scope-await/cases.json", fixtureRoot)));
const expected = JSON.parse(readFileSync(new URL(pin.reference.path, fixtureRoot)));
const sources = new Map([["reported", reported], ...controls.map((row) => [row.name, row.source])]);
assert.equal(sources.size, 16);
assert.deepEqual(
  expected.map((row) => row.name),
  [...sources.keys()],
);

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(input.cases.length, 32);
const fromTests = createRequire(new URL("../../package.json", import.meta.url));
const vuePackagePath = fromTests.resolve("vue-setup-await-oracle/package.json");
const fromVue = createRequire(vuePackagePath);
const vue = fromVue("vue");
const compiler = fromVue("vue/compiler-sfc");
const server = fromVue("vue/server-renderer");
const ts = fromTests("typescript");
const fromCompiler = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
const { SourceMapConsumer } = fromCompiler("source-map-js");
const identities = [
  vuePackagePath,
  fromVue.resolve("@vue/compiler-sfc/package.json"),
  fromVue.resolve("@vue/server-renderer/package.json"),
].map((path) => {
  const bytes = readFileSync(path);
  const value = JSON.parse(bytes);
  assert.equal(value.version, "3.5.38");
  return { path, sha256: sha(bytes), value };
});
assert.equal(vue.version, "3.5.38");
assert.equal(compiler.version, vue.version);
globalThis.__setupAwaitModules = { vue, server };
const receipt = {
  schema: "vize.setup-scope-await-runtime-v1",
  complete: false,
  identities,
  typescriptVersion: ts.version,
  observations: [],
};
function save() {
  writeFileSync(join(input.capture, "runtime.json"), JSON.stringify(receipt, null, 2) + "\n");
}
save();
let sequence = 0;

function astFacts(code) {
  const ast = parse(code, { sourceType: "module", plugins: ["typescript"] });
  const aliases = new Set();
  for (const node of ast.program.body) {
    if (node.type !== "ImportDeclaration" || node.source.value !== "vue") continue;
    for (const specifier of node.specifiers)
      if (specifier.type === "ImportSpecifier" && specifier.imported.name === "withAsyncContext")
        aliases.add(specifier.local.name);
  }
  let wraps = 0;
  let awaits = 0;
  let nestedAwaits = 0;
  let blockCallbacks = 0;
  function walk(value, authoredFunction = false, parent = null) {
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value)) {
      for (const item of value) walk(item, authoredFunction, parent);
      return;
    }
    let own = authoredFunction;
    if (value.type === "FunctionDeclaration" && value.id?.name === "declared") own = true;
    if (
      value.type === "ArrowFunctionExpression" &&
      parent?.type === "VariableDeclarator" &&
      parent.id?.name === "arrow"
    )
      own = true;
    if (["ObjectMethod", "ClassMethod"].includes(value.type) && value.key?.name === "method")
      own = true;
    if (value.type === "AwaitExpression") {
      awaits++;
      if (own) nestedAwaits++;
    }
    if (
      value.type === "CallExpression" &&
      value.callee.type === "Identifier" &&
      aliases.has(value.callee.name)
    ) {
      assert.equal(own, false, "no transformation inside the authored nested async functions");
      wraps++;
      if (
        value.arguments[0]?.type === "ArrowFunctionExpression" &&
        value.arguments[0].body.type === "BlockStatement"
      )
        blockCallbacks++;
    }
    for (const [key, child] of Object.entries(value))
      if (!["loc", "start", "end", "comments", "tokens"].includes(key)) walk(child, own, value);
  }
  walk(ast);
  return { wraps, awaits, nestedAwaits, blockCallbacks };
}

function mappingFacts(code, map, source, name) {
  assert.equal(map.version, 3);
  assert.deepEqual(map.sourcesContent, [source]);
  assert.equal(map.sources.length, 1);
  const generatedLines = code.split("\n");
  const originalLines = source.split("\n");
  const consumer = new SourceMapConsumer(map);
  const mappings = [];
  consumer.eachMapping((entry) => {
    assert.ok(entry.generatedLine >= 1 && entry.generatedLine <= generatedLines.length);
    assert.ok(
      entry.generatedColumn >= 0 &&
        entry.generatedColumn <= generatedLines[entry.generatedLine - 1].length,
    );
    if (entry.originalLine !== null) {
      assert.ok(entry.originalLine >= 1 && entry.originalLine <= originalLines.length);
      assert.ok(
        entry.originalColumn >= 0 &&
          entry.originalColumn <= originalLines[entry.originalLine - 1].length,
      );
      assert.equal(entry.source, map.sources[0]);
    }
    mappings.push(entry);
  });
  assert.ok(mappings.length > 0);
  const token = name === "reported" ? "getCurrentInstance() !== null" : 'mark("end")';
  function position(text) {
    const index = text.indexOf(token);
    assert.ok(index >= 0 && text.indexOf(token, index + 1) < 0, token);
    const lines = text.slice(0, index).split("\n");
    return { line: lines.length, column: lines.at(-1).length };
  }
  const authored = position(source);
  const generated = position(code);
  const mapped = consumer.originalPositionFor(generated);
  assert.deepEqual(
    { line: mapped.line, column: mapped.column },
    authored,
    "the whole module maps the unchanged post-await observation to its exact authored UTF16 position",
  );
  return { mappings, postAwait: { token, authored, generated, mapped } };
}

async function component(code) {
  // Erase TypeScript only for execution; retain the original complete code/maps above.
  const erased = ts.transpileModule(code, {
    fileName: "Probe.ts",
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
    reportDiagnostics: true,
  });
  assert.deepEqual(erased.diagnostics, []);
  const transformed = transformSync(erased.outputText, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const moduleName =
              path.node.source.value === "vue"
                ? "vue"
                : path.node.source.value === "@vue/server-renderer"
                  ? "server"
                  : null;
            assert.ok(moduleName, path.node.source.value);
            const declarations = path.node.specifiers.map((specifier) => {
              assert.ok(t.isImportSpecifier(specifier));
              const name = specifier.imported.name;
              assert.ok(Object.hasOwn(globalThis.__setupAwaitModules[moduleName], name), name);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(
                    t.memberExpression(
                      t.identifier("globalThis"),
                      t.identifier("__setupAwaitModules"),
                    ),
                    t.identifier(moduleName),
                  ),
                  t.stringLiteral(name),
                  true,
                ),
              );
            });
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  });
  const body = transformed.code + "\n// isolated setup-await module " + sequence++;
  const loaded = await import(
    "data:text/javascript;base64," + Buffer.from(body).toString("base64")
  );
  assert.ok(loaded.default && typeof loaded.default.setup === "function");
  return loaded.default;
}

async function observe(compiled, props) {
  const warnings = [];
  const consoleWarnings = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...args) => consoleWarnings.push({ method: "warn", args: args.map(String) });
  console.error = (...args) => consoleWarnings.push({ method: "error", args: args.map(String) });
  const app = vue.createSSRApp({ render: () => vue.h(compiled, props) });
  app.provide("token", "provided");
  app.config.warnHandler = (message) => warnings.push(message);
  try {
    const html = await server.renderToString(app);
    const result = { html, warnings, consoleWarnings };
    assert.equal(vue.getCurrentInstance(), null, "SSR leaves no active setup instance");
    return result;
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
  }
}

const seen = new Set();
for (const row of input.cases) {
  const key = row.name + "/" + row.target;
  assert.ok(!seen.has(key));
  seen.add(key);
  assert.ok(["dom", "ssr"].includes(row.target));
  assert.equal(row.source, sources.get(row.name));
  assert.equal(row.filename, row.name + ".vue");
  const oracle = expected.find((value) => value.name === row.name);
  const current = row.current;
  assert.deepEqual(
    Object.keys(current).sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
    ["bindings", "code", "css", "errors", "macroArtifacts", "map", "warnings"],
  );
  assert.deepEqual(
    {
      css: current.css,
      errors: current.errors,
      warnings: current.warnings,
      macroArtifacts: current.macroArtifacts,
    },
    { css: null, errors: [], warnings: [], macroArtifacts: [] },
  );
  assert.ok(current.bindings?.isScriptSetup);
  const referenceSource =
    controls.find((value) => value.name === row.name)?.referenceSource ?? row.source;
  const parsed = compiler.parse(referenceSource, { filename: row.filename });
  assert.deepEqual(parsed.errors, []);
  const official = compiler.compileScript(parsed.descriptor, {
    id: "probe",
    inlineTemplate: true,
    sourceMap: true,
    templateOptions: { ssr: row.target === "ssr" },
  });
  const officialBare =
    referenceSource === row.source
      ? null
      : compiler.compileScript(compiler.parse(row.source, { filename: row.filename }).descriptor, {
          id: "probe",
          inlineTemplate: true,
          sourceMap: true,
          templateOptions: { ssr: row.target === "ssr" },
        });
  const observation = {
    name: row.name,
    target: row.target,
    source: row.source,
    current,
    referenceSource,
    official: JSON.parse(JSON.stringify(official)),
    sourceSha256: sha(row.source),
    referenceSourceSha256: sha(referenceSource),
    stockBareEvidence: officialBare
      ? {
          complete: JSON.parse(JSON.stringify(officialBare)),
          facts: astFacts(officialBare.content),
          mappings: mappingFacts(officialBare.content, officialBare.map, row.source, row.name),
          runtimeAcceptance: false,
        }
      : null,
    currentFacts: astFacts(current.code),
    officialFacts: astFacts(official.content),
    currentMappings: mappingFacts(current.code, current.map, row.source, row.name),
    officialMappings: mappingFacts(official.content, official.map, referenceSource, row.name),
    runtime: [],
  };
  receipt.observations.push(observation);
  save();
  const facts = {
    wraps: oracle.wraps,
    awaits: oracle.wraps + oracle.nestedAwaits,
    nestedAwaits: oracle.nestedAwaits,
    blockCallbacks: 0,
  };
  assert.deepEqual(observation.currentFacts, facts, key);
  assert.deepEqual(observation.officialFacts, facts, key);
  if (officialBare)
    assert.deepEqual(observation.stockBareEvidence.facts, { ...facts, blockCallbacks: 2 });
  const [actualComponent, officialComponent] = await Promise.all([
    component(current.code),
    component(official.content),
  ]);
  for (const [state, props] of [
    ["default", {}],
    ["true", { wait: true }],
    ["false", { wait: false }],
  ]) {
    const actual = await observe(actualComponent, props);
    const reference = await observe(officialComponent, props);
    observation.runtime.push({ state, props, actual, reference });
    save();
    assert.deepEqual(
      reference,
      { html: oracle.html[state], warnings: [], consoleWarnings: [] },
      "authored complete Vue oracle " + key,
    );
    assert.deepEqual(actual, reference, "whole actual SSR result " + key);
  }
}
assert.equal(seen.size, 32);
receipt.complete = true;
save();
process.stdout.write(JSON.stringify(receipt) + "\n");
