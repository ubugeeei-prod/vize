// Complete source-built modules are mounted, not reconstructed from setup bodies.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { loadRuntime } from "./davinci-mounted-trace.mjs";

const root = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(readFileSync(new URL("page-meta-runtime.manifest.json", root)));
const pinned = manifest.cases[0];
for (const file of [
  ...pinned.inputs.files.map((file) => ({ ...file, path: `${pinned.inputs.root}/${file.path}` })),
  pinned.reference,
])
  assert.equal(
    createHash("sha256")
      .update(readFileSync(new URL(file.path, root)))
      .digest("hex"),
    file.sha256,
    file.path,
  );
const expected = JSON.parse(readFileSync(new URL(pinned.reference.path, root)));
assert.equal(process.env.NODE_ENV, "production");
const window = new Window();
for (const name of [
  "window",
  "document",
  "Document",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Node",
  "Text",
  "Comment",
  "Event",
  "ShadowRoot",
])
  globalThis[name] = name === "window" ? window : window[name];
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const stable = fromUi("vue");
const stableCompiler = fromUi("vue/compiler-sfc");
const server = fromUi("vue/server-renderer");
const fromVapor = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const vaporCompiler = fromVapor("vue/compiler-sfc");
assert.equal(stable.version, "3.5.43");
assert.equal(vaporCompiler.version, "3.6.0-rc.9");
const vapor = await loadRuntime({ production: true });
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const { transformSync } = fromUi("@babel/core");
let sequence = 0;

function originalReference(fixture) {
  const isVapor = fixture.shape === "vapor";
  const compiler = isVapor ? vaporCompiler : stableCompiler;
  const parsed = compiler.parse(fixture.source, { filename: "Page.vue" });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.source, fixture.source);
  // Select the real official Vapor target while retaining the authored SFC.
  if (isVapor) parsed.descriptor.vapor = true;
  const result = compiler.compileScript(parsed.descriptor, {
    id: "page-meta",
    isProd: true,
    inlineTemplate: true,
    genDefaultAs: "__reference",
    templateOptions: { ssr: fixture.shape.endsWith("ssr") },
  });
  return `${result.content}\nexport default __reference;`;
}

async function evaluate(code, vue) {
  const plain = await transformWithOxc(code, "Page.vue.ts", { lang: "ts" });
  const record = (meta) => {
    const keys = Object.keys(meta).sort();
    assert.deepEqual(keys, keys.includes("middleware") ? ["layout", "middleware"] : ["layout"]);
    assert.equal(meta.layout, "plain");
    if (keys.includes("middleware")) assert.equal(typeof meta.middleware, "function");
    globalThis.__pageMetaCalls.push({
      layout: meta.layout,
      ...(meta.middleware ? { middleware: meta.middleware() } : {}),
    });
  };
  globalThis.__pageMetaRecord = record;
  globalThis.definePageMeta = record;
  const imports = { definePageMeta: record, useRoute: () => ({ path: "/test" }) };
  globalThis.__pageMetaModules = {
    vue,
    "vue/vapor": vue,
    "#imports": imports,
    "./meta": imports,
    "@vue/server-renderer": server,
    "vue/server-renderer": server,
  };
  const transformed = transformSync(plain.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            assert.ok(Object.hasOwn(globalThis.__pageMetaModules, source), source);
            const owner = t.memberExpression(
              t.identifier("globalThis"),
              t.identifier("__pageMetaModules"),
            );
            const declarations = path.node.specifiers.map((specifier) => {
              const imported = t.isImportDefaultSpecifier(specifier)
                ? "default"
                : specifier.imported.name;
              assert.ok(
                Object.hasOwn(globalThis.__pageMetaModules[source], imported),
                `${source}/${imported}`,
              );
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(owner, t.stringLiteral(source), true),
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
  return (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${transformed}\n// observation ${sequence++}`).toString("base64")}`
    )
  ).default;
}

async function observe(code, fixture) {
  globalThis.__pageMetaCalls = [];
  const vue = fixture.shape === "vapor" ? vapor : stable;
  const component = await evaluate(code, vue);
  assert.equal(typeof component.setup, "function", "the original complete default owns setup");
  let html;
  if (fixture.shape.endsWith("ssr")) {
    html = await server.renderToString(vue.createSSRApp(component));
  } else {
    const host = document.createElement("div");
    document.body.append(host);
    const app = (fixture.shape === "vapor" ? vue.createVaporApp : vue.createApp)(component);
    try {
      app.mount(host);
      await vue.nextTick();
      html = host.innerHTML;
    } finally {
      app.unmount();
      host.remove();
    }
  }
  return { html, calls: structuredClone(globalThis.__pageMetaCalls) };
}

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const fixtures = JSON.parse(Buffer.concat(chunks).toString("utf8"));
for (const fixture of fixtures) {
  assert.equal(
    fixture.source,
    readFileSync(new URL(`${pinned.inputs.root}/${fixture.name}.vue.txt`, root), "utf8"),
  );
  const reference = await observe(originalReference(fixture), fixture);
  const call = fixture.name === "mixed" ? expected.mixedCall : expected.call;
  assert.deepEqual(
    reference,
    { html: expected.html, calls: [call] },
    "independent official component behavior",
  );
  const actual = await observe(fixture.code, fixture);
  const extracted = fixture.nuxt && ["Page", "global", "mixed"].includes(fixture.name);
  if (extracted && fixture.name !== "mixed") {
    assert.equal(typeof fixture.artifact, "string");
    const metadata = await evaluate(fixture.artifact, stable);
    assert.deepEqual(Object.keys(metadata).sort(), Object.keys(call).sort());
    assert.deepEqual(
      {
        layout: metadata.layout,
        ...(metadata.middleware ? { middleware: metadata.middleware() } : {}),
      },
      call,
      "the complete Nuxt artifact retains the authored metadata and middleware",
    );
  } else if (!extracted) assert.equal(fixture.artifact, null);
  assert.deepEqual(
    actual,
    { html: expected.html, calls: extracted ? expected.nuxtCalls : [call] },
    `${fixture.shape}/${fixture.name}/${fixture.nuxt}`,
  );
  console.log(
    JSON.stringify({
      name: fixture.name,
      shape: fixture.shape,
      nuxt: fixture.nuxt,
      actual,
      vue: fixture.shape === "vapor" ? vaporCompiler.version : stable.version,
      authority: "complete-default-component/setup/production-runtime",
      ...(fixture.shape === "vapor-ssr" ? { ssr: "documented-vdom-fallback" } : {}),
    }),
  );
}
