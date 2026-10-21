import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import process from "node:process";
import { compileFunction } from "node:vm";
import { fileURLToPath, pathToFileURL } from "node:url";
import { transformSync } from "@babel/core";
import { Window } from "happy-dom";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..", "..");
const fixtureDir = join(
  repoRoot,
  "crates",
  "vize_atelier_jsx",
  "tests",
  "vue_jsx_vapor_compat",
  "fixtures",
);
const recordedCorpus = JSON.parse(readFileSync(join(fixtureDir, "corpus.json"), "utf8"));
const verdicts = JSON.parse(readFileSync(join(fixtureDir, "verdicts.json"), "utf8"));
const inventory = readFileSync(
  join(repoRoot, "crates", "vize_atelier_jsx", "tests", "VUE_JSX_VAPOR_COMPAT_INVENTORY.md"),
  "utf8",
);

const window = new Window();
for (const key of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "HTMLInputElement",
  "SVGElement",
  "Event",
  "ShadowRoot",
]) {
  globalThis[key] = key === "window" ? window : window[key];
}

const require = createRequire(import.meta.url);
// The development browser build prints a banner to stdout during module
// evaluation, which would corrupt the machine-readable receipt consumed by
// the Rust oracle. Production and reference output must share this exact
// runtime entry so stdout remains a single JSON document.
const vueVaporEntry = require.resolve("vue/dist/vue.runtime-with-vapor.esm-browser.prod.js");
const Vue = await import(pathToFileURL(vueVaporEntry));
const { transformVueJsxVapor } = await import("vue-jsx-vapor/api");

// Execute the published vue-jsx-vapor runtime source against the same Vue
// module as Vize output. Node's `vue` condition omits Vapor helpers, while the
// package is normally linked by a bundler to Vue's with-vapor entry.
const fromJsxVapor = createRequire(require.resolve("vue-jsx-vapor"));
const runtimeEntry = fromJsxVapor.resolve("@vue-jsx-vapor/runtime");
const runtimeSource = readFileSync(join(dirname(runtimeEntry), "vapor.js"), "utf8");
const linkedRuntime = transformSync(runtimeSource, {
  configFile: false,
  babelrc: false,
  plugins: [
    ({ types: t }) => ({
      visitor: {
        ImportDeclaration(path) {
          assert.equal(path.node.source.value, "vue");
          const declarations = path.node.specifiers.map((specifier) => {
            if (specifier.type === "ImportNamespaceSpecifier") {
              return t.variableDeclaration("const", [
                t.variableDeclarator(specifier.local, t.identifier("__Vue")),
              ]);
            }
            assert.equal(specifier.type, "ImportSpecifier");
            return t.variableDeclaration("const", [
              t.variableDeclarator(
                specifier.local,
                t.memberExpression(t.identifier("__Vue"), specifier.imported),
              ),
            ]);
          });
          path.replaceWithMultiple(declarations);
        },
        ExportNamedDeclaration(path) {
          if (path.node.declaration) path.replaceWith(path.node.declaration);
          else path.remove();
        },
      },
    }),
  ],
}).code;
const JsxVapor = compileFunction(`${linkedRuntime}\nreturn { setNodes };`, ["__Vue"], {
  filename: "vue-jsx-vapor.runtime.mjs",
})(Vue);
const installed = {
  packageVersion: require("vue-jsx-vapor/package.json").version,
  vueVersion: require("vue/package.json").version,
};

function linkedFunction(code, functionName, state) {
  const linked = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            const runtime =
              source === "vue" ? "Vue" : source.includes("vue-jsx-vapor") ? "JsxVapor" : null;
            assert.notEqual(runtime, null, `unexpected runtime import: ${source}`);
            const properties = path.node.specifiers.map((specifier) => {
              assert.equal(specifier.type, "ImportSpecifier", source);
              return t.objectProperty(specifier.imported, specifier.local);
            });
            path.replaceWith(
              t.variableDeclaration("const", [
                t.variableDeclarator(t.objectPattern(properties), t.identifier(runtime)),
              ]),
            );
          },
          ExportNamedDeclaration(path) {
            assert.notEqual(
              path.node.declaration,
              null,
              "only declaration exports are executable fixtures",
            );
            path.replaceWith(path.node.declaration);
          },
        },
      }),
    ],
  }).code;
  return compileFunction(`${linked}\nreturn ${functionName};`, ["Vue", "JsxVapor", "state"], {
    filename: `${functionName}.generated.mjs`,
  })(Vue, JsxVapor, state);
}

function observeChildren(parent) {
  const children = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      if (!node.data) continue;
      if (typeof children.at(-1) === "string") children[children.length - 1] += node.data;
      else children.push(node.data);
      continue;
    }
    if (node.nodeType !== 1) continue;
    const attributes = Object.fromEntries(
      [...node.attributes]
        .map(({ name, value }) => [name, value])
        .sort(([left], [right]) => left.localeCompare(right)),
    );
    const element = { tag: node.localName, attributes, children: observeChildren(node) };
    if (node.localName === "input") {
      element.value = node.value;
      element.checked = node.checked;
    }
    children.push(element);
  }
  return children;
}

async function trace(code, functionName, spec) {
  const state = Vue.reactive(structuredClone(spec.initial));
  const render = linkedFunction(code, functionName, state);
  const component = Vue.defineVaporComponent({ setup: () => render({}) });
  const app = Vue.createVaporApp(component);
  const warnings = [];
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap();
  let nextIdentity = 0;

  function snapshot() {
    assert.deepEqual(warnings, [], `${spec.id}: runtime diagnostics`);
    const elements = [...host.querySelectorAll("*")];
    for (const element of elements) {
      if (!identities.has(element)) identities.set(element, nextIdentity++);
    }
    return {
      dom: observeChildren(host),
      text: host.textContent,
      elementIdentities: elements.map((element) => identities.get(element)),
      warnings: [...warnings],
    };
  }

  try {
    app.mount(host);
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const step of spec.steps) {
      assert.deepEqual(Object.keys(step), ["patch"], `${spec.id}: update shape`);
      Object.assign(state, structuredClone(step.patch));
      await Vue.nextTick();
      phases.push(snapshot());
    }
    app.unmount();
    await Vue.nextTick();
    assert.equal(host.childNodes.length, 0, `${spec.id}: complete unmount`);
    return phases;
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}

function referenceCode(spec) {
  const result = transformVueJsxVapor(spec.source, `/semantic-oracle/${spec.id}.jsx?vapor=true`);
  assert.equal(typeof result.code, "string", `${spec.id}: oracle output`);
  return result.code;
}

function assertInventory(corpus) {
  assert.deepEqual(installed, {
    packageVersion: corpus.oracle.packageVersion,
    vueVersion: corpus.oracle.vueVersion,
  });
  assert.equal(verdicts.oraclePackageVersion, installed.packageVersion);
  assert.equal(verdicts.vueVersion, installed.vueVersion);
  const ids = corpus.cases.map(({ id }) => id).sort();
  assert.deepEqual(Object.keys(verdicts.cases).sort(), ids);
  for (const id of ids) {
    assert.equal(verdicts.cases[id].verdict, "equivalent", id);
    assert.equal(typeof verdicts.cases[id].basis, "string", id);
  }
  const documented = [
    ...inventory.matchAll(/^\| `([^`]+)`\s+\| (?:equivalent|divergent|deferred)\s+\|/gmu),
  ]
    .map((match) => match[1])
    .sort();
  assert.deepEqual(documented, ids);
}

function assertBehavior(spec, phases) {
  assert.equal(phases.length, spec.steps.length + 1, `${spec.id}: initial render plus updates`);
  assert.deepEqual(
    phases.map(({ text }) => text),
    spec.expectedText,
    `${spec.id}: visible update trace`,
  );
  assert.deepEqual(
    phases.map(({ elementIdentities }) => elementIdentities),
    spec.expectedElementIdentities,
    `${spec.id}: element update identity trace`,
  );
}

assertInventory(recordedCorpus);

if (process.argv.includes("--reference")) {
  const reference = {};
  for (const spec of recordedCorpus.cases) {
    reference[spec.id] = await trace(referenceCode(spec), "App", spec);
  }
  console.log(JSON.stringify(reference, null, 2));
} else {
  let payload = "";
  for await (const chunk of process.stdin) payload += chunk;
  const input = JSON.parse(payload);
  assert.deepEqual(input.corpus, recordedCorpus);
  assert.deepEqual(
    input.compiled.map(({ id, source }) => ({ id, source })),
    recordedCorpus.cases.map(({ id, source }) => ({ id, source })),
  );

  const receipt = { cases: recordedCorpus.cases.length, equivalent: 0, failures: 0 };
  for (const [index, spec] of recordedCorpus.cases.entries()) {
    const compiled = input.compiled[index];
    const reference = await trace(referenceCode(spec), "App", spec);
    const actual = await trace(compiled.code, compiled.functionName, spec);
    assert.deepEqual(actual, reference, spec.id);
    assertBehavior(spec, actual);
    receipt.equivalent += 1;
  }
  await window.happyDOM.close();
  console.log(JSON.stringify(receipt));
}
