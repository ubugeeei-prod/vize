import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { loadRuntime } from "./davinci-mounted-trace.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const { lane, source, code } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(["dom", "ssr", "vapor"].includes(lane));
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
])
  globalThis[name] = name === "window" ? window : window[name];

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromStable = createRequire(fromUi.resolve("vue/package.json"));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = (lane === "vapor" ? fromVue : fromUi)("vue/compiler-sfc");
const vue = lane === "vapor" ? await loadRuntime({ production: false }) : fromUi("vue");
const server = fromUi("vue/server-renderer");
assert.equal(compiler.version, lane === "vapor" ? "3.6.0-rc.9" : "3.5.35");
assert.equal(vue.version, compiler.version);
assert.equal(fromStable("@vue/server-renderer/package.json").version, "3.5.35");
const { transformSync } = fromUi("@babel/core");
const fromPlugin = createRequire(fromUi.resolve("@vitejs/plugin-vue/package.json"));
const { transformWithOxc } = await import(pathToFileURL(fromPlugin.resolve("vite")).href);
const options = {
  id: "enum-template-bindings-7893",
  inlineTemplate: true,
  templateOptions: { ssr: lane === "ssr" },
};
const parsed = compiler.parse(source, { filename: "Badge.vue" });
assert.deepEqual(parsed.errors, []);
assert.equal(parsed.descriptor.source, source);
parsed.descriptor.vapor = lane === "vapor";
const official = compiler.compileScript(parsed.descriptor, options);
const reference = await transformWithOxc(official.content, "Badge.vue.ts", { lang: "ts" });
assert.deepEqual(reference.warnings ?? [], []);
const capture = {
  source,
  actualCode: code,
  official: {
    version: compiler.version,
    filename: "Badge.vue",
    options,
    vapor: parsed.descriptor.vapor,
    content: official.content,
    bindings: official.bindings,
    map: official.map,
    transformedCode: reference.code,
    transformationWarnings: reference.warnings ?? [],
  },
  actual: [],
  reference: [],
};
globalThis.__enum7893 = {
  vue,
  "vue/vapor": vue,
  "vue/server-renderer": server,
  "@vue/server-renderer": server,
};
let sequence = 0;

async function evaluate(moduleCode) {
  const transformed = transformSync(moduleCode, {
    configFile: false,
    babelrc: false,
    sourceType: "module",
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const owner = path.node.source.value;
            assert.ok(Object.hasOwn(globalThis.__enum7893, owner), owner);
            const declarations = path.node.specifiers.map((specifier) => {
              assert.ok(
                t.isImportSpecifier(specifier),
                "only named Vue runtime imports are expected",
              );
              const imported = specifier.imported.name;
              assert.ok(Object.hasOwn(globalThis.__enum7893[owner], imported), imported);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(
                    t.memberExpression(t.identifier("globalThis"), t.identifier("__enum7893")),
                    t.stringLiteral(owner),
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
  });
  return (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${transformed.code}\n// ${sequence++}`).toString("base64")}`
    )
  ).default;
}

async function observe(moduleCode, observations) {
  const component = await evaluate(moduleCode);
  for (const [props, expected] of [
    [{}, '<span class="">info</span>'],
    [{ tone: "warn" }, '<span class="warn">warn</span>'],
    [{ tone: "info" }, '<span class="">info</span>'],
  ]) {
    const diagnostics = [];
    const app = (
      lane === "ssr" ? vue.createSSRApp : lane === "vapor" ? vue.createVaporApp : vue.createApp
    )(component, props);
    app.config.warnHandler = (message) => diagnostics.push(message);
    app.config.errorHandler = (error) => diagnostics.push(String(error));
    const observation = { props, diagnostics, html: null, unmounted: null };
    observations.push(observation);
    if (lane === "ssr") {
      observation.html = await server.renderToString(app);
      assert.equal(observation.html, expected);
    } else {
      const host = window.document.createElement("div");
      window.document.body.append(host);
      try {
        app.mount(host);
        await vue.nextTick();
        observation.html = host.innerHTML;
        assert.equal(observation.html, expected);
        app.unmount();
        await vue.nextTick();
        observation.unmounted = host.innerHTML;
        assert.equal(observation.unmounted, "");
      } finally {
        if (host.childNodes.length) app.unmount();
        host.remove();
      }
    }
    assert.deepEqual(diagnostics, []);
  }
}

try {
  await observe(reference.code, capture.reference);
  await observe(code, capture.actual);
  assert.deepEqual(capture.actual, capture.reference);
} catch (error) {
  capture.failure = { name: error.name, message: error.message, stack: error.stack };
  throw error;
} finally {
  process.stdout.write(JSON.stringify(capture));
  delete globalThis.__enum7893;
  await window.happyDOM.close();
}
