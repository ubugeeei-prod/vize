import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";

const fixtureRoot = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(readFileSync(new URL("vapor-css-vars.manifest.json", fixtureRoot)));
const [pinned] = manifest.cases;
assert.equal(manifest.cases.length, 1);
assert.equal(pinned.id, "compiler/sfc/vapor-css-vars");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const originals = new Map();
for (const item of pinned.inputs.files) {
  const bytes = readFileSync(new URL(`${pinned.inputs.root}/${item.path}`, fixtureRoot));
  assert.equal(digest(bytes), item.sha256, item.path);
  originals.set(item.path, bytes.toString("utf8"));
}
const expectation = readFileSync(new URL(pinned.reference.path, fixtureRoot));
assert.equal(digest(expectation), pinned.reference.sha256);
const expected = JSON.parse(expectation);
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(typeof input.production, "boolean");
const sourceNames = {
  reported: "App.vue.txt",
  flag: "flag.vue.txt",
  typed: "typed.vue.txt",
  vdom: "flag.vue.txt",
};
assert.deepEqual(
  input.cases.map(({ name, ssr }) => `${name}:${ssr}`).sort(),
  Object.keys(sourceNames)
    .flatMap((name) => [`${name}:false`, `${name}:true`])
    .sort(),
);

const window = new Window();
for (const name of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Event",
  "ShadowRoot",
  "MutationObserver",
])
  globalThis[name] = name === "window" ? window : window[name];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVapor = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVapor("vue/compiler-sfc");
assert.equal(compiler.version, pinned.provenance.versions.vueRuntime);
const runtime = await loadRuntime({ production: input.production });
assert.equal(runtime.version, compiler.version);
const stable = fromUi("vue");
const stableCompiler = fromUi("vue/compiler-sfc");
const server = fromUi("vue/server-renderer");
assert.equal(stable.version, pinned.provenance.versions.ssrRuntime);
assert.equal(stableCompiler.version, stable.version);
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const { transformSync } = fromUi("@babel/core");
let sequence = 0;

async function evaluate(code, vue) {
  globalThis.__cssVarsModules = {
    vue,
    "vue/vapor": vue,
    "@vue/server-renderer": server,
    "vue/server-renderer": server,
  };
  const plain = await transformWithOxc(code, "App.vue.ts", { lang: "ts" });
  const normalized = transformSync(plain.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            assert.ok(Object.hasOwn(globalThis.__cssVarsModules, source), source);
            const declarations = path.node.specifiers.map((specifier) => {
              const imported = t.isImportDefaultSpecifier(specifier)
                ? "default"
                : specifier.imported.name;
              assert.ok(Object.hasOwn(globalThis.__cssVarsModules[source], imported), imported);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(
                    t.memberExpression(
                      t.identifier("globalThis"),
                      t.identifier("__cssVarsModules"),
                    ),
                    t.stringLiteral(source),
                    true,
                  ),
                  t.stringLiteral(imported),
                  true,
                ),
              );
            });
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  }).code;
  const component = (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${normalized}\n// ${sequence++}`).toString("base64")}`
    )
  ).default;
  // Both stock and Vize raw SFC modules use the adapter's scope attachment.
  // This is component metadata, not an instance or a CSS-helper replacement.
  component.__scopeId = "data-v-abc12345";
  return component;
}

function official(fixture) {
  const selected = fixture.ssr ? stableCompiler : compiler;
  const parsed = selected.parse(fixture.source, { filename: "App.vue" });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.source, fixture.source);
  parsed.descriptor.vapor = fixture.vapor && !fixture.ssr;
  const result = selected.compileScript(parsed.descriptor, {
    id: "abc12345",
    isProd: input.production,
    inlineTemplate: true,
    genDefaultAs: "__reference",
    templateOptions: { ssr: fixture.ssr },
  });
  const style = selected.compileStyle({
    source: parsed.descriptor.styles[0].content,
    filename: "App.vue",
    id: "data-v-abc12345",
    scoped: true,
    isProd: input.production,
  });
  assert.deepEqual(style.errors, []);
  // SFC adapters attach the scope ID to the complete compiled component.
  return {
    code: `${result.content}\n__reference.__scopeId = "data-v-abc12345";\nexport default __reference;`,
    css: style.code,
  };
}

function propertyName(css) {
  const properties = [...css.matchAll(/var\(--([\w-]+)\)/g)].map((match) => `--${match[1]}`);
  assert.equal(properties.length, 1, "the whole original stylesheet has one variable");
  return properties[0];
}

function observe(host, property, sameRoot, diagnostics) {
  const root = host.querySelector("p.x");
  if (root) {
    assert.equal(root.style.length, 1, "no missing or additional root CSS properties");
    assert.equal(root.style.item(0), property, "stylesheet and actual root property agree");
  }
  const tree = observeChildren(host);
  // Production names are deliberately backend-specific. Preserve both raw
  // modules/CSS and normalize only this authenticated single property name.
  for (const node of tree) {
    assert.equal(node.tag, "p");
    if (node.attributes.style)
      node.attributes.style = node.attributes.style.replace(property, "--bound-color");
  }
  return {
    tree,
    cssValues: root ? [root.style.getPropertyValue(property)] : [],
    sameRoot: Boolean(root && sameRoot === root),
    diagnostics: [...diagnostics],
  };
}

async function mounted(compiled, fixture, capture) {
  const component = await evaluate(compiled.code, runtime);
  capture.scopeId = component.__scopeId;
  const app = (fixture.vapor ? runtime.createVaporApp : runtime.createApp)(component);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const property = propertyName(compiled.css);
  const raw = [];
  capture.raw = raw;
  try {
    app.mount(host);
    await runtime.nextTick();
    const original = host.querySelector("p.x");
    assert.ok(original, "original root exists");
    raw.push({ html: host.innerHTML, property });
    const result = [observe(host, property, original, diagnostics)];
    capture.result = result;
    original.click();
    await runtime.nextTick();
    raw.push({ html: host.innerHTML, property });
    result.push(observe(host, property, original, diagnostics));
    app.unmount();
    await runtime.nextTick();
    assert.equal(host.childNodes.length, 0, "unmount removes the complete component");
    raw.push({ html: host.innerHTML, property });
    result.push(observe(host, property, original, diagnostics));
    assert.deepEqual(result, expected.mounted);
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}

async function ssr(compiled, capture) {
  const component = await evaluate(compiled.code, stable);
  capture.scopeId = component.__scopeId;
  const app = stable.createSSRApp(component);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const html = await server.renderToString(app);
  const host = window.document.createElement("div");
  host.innerHTML = html;
  const property = propertyName(compiled.css);
  const { tree, cssValues } = observe(host, property, null, diagnostics);
  const result = { tree, cssValues, diagnostics };
  Object.assign(capture, { result, raw: { html, property } });
  assert.deepEqual(result, expected.ssr);
}

const rows = [];
try {
  for (const fixture of input.cases) {
    assert.equal(fixture.source, originals.get(sourceNames[fixture.name]));
    assert.equal(fixture.vapor, fixture.name !== "vdom");
    assert.equal(typeof fixture.ssr, "boolean");
    const reference = official(fixture);
    const actualModule = fixture.compiled.result;
    const referenceObservation = {};
    const actualObservation = {};
    const row = {
      name: fixture.name,
      ssr: fixture.ssr,
      production: input.production,
      reference,
      actualObservation,
      referenceObservation,
      versions: { client: runtime.version, ssr: stable.version },
      authority:
        fixture.ssr && fixture.vapor ? "existing-vdom-ssr-fallback" : "real-mounted-vue-runtime",
      status: "RUNNING",
    };
    rows.push(row);
    try {
      if (fixture.ssr) {
        await ssr(reference, referenceObservation);
        await ssr(actualModule, actualObservation);
      } else {
        await mounted(reference, fixture, referenceObservation);
        await mounted(actualModule, fixture, actualObservation);
      }
      assert.deepEqual(actualObservation.result, referenceObservation.result, fixture.name);
      row.status = "PASS";
    } catch (error) {
      row.status = "FAIL";
      row.error = { name: error.name, message: error.message, stack: error.stack };
      throw error;
    }
  }
} finally {
  process.stdout.write(JSON.stringify(rows));
  delete globalThis.__cssVarsModules;
  await window.happyDOM.close();
}
