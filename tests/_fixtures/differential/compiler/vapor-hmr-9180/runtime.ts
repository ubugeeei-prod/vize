import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { transformSync } from "@babel/core";
import { Window } from "happy-dom";

// This report predates rc.11. Exercise today's supported runtime explicitly;
// the historical vue-vapor-runtime alias intentionally still resolves rc.9.
const fromUi = createRequire(new URL("../../../../../npm/ui/package.json", import.meta.url));
const version = fromUi("vue-vapor-runtime-supported/package.json").version;
assert.equal(version, "3.6.0-rc.11");
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
if (!input.vapor) {
  // An independent same-source oracle; Vapor modules always come from the
  // Rust test's current-source Vize compiler, never this reference compiler.
  const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime-supported/package.json"));
  const compiler = fromVue("@vue/compiler-sfc");
  assert.equal(compiler.version, version);
  const compile = (source: string, filename: string) => {
    const parsed = compiler.parse(source, { filename });
    assert.deepEqual(parsed.errors, []);
    const script = compiler.compileScript(parsed.descriptor, {
      id: "9180",
      genDefaultAs: "__component",
      inlineTemplate: input.production,
      isProd: input.production,
    });
    if (input.production) return `${script.content}\nexport default __component;`;
    const template = compiler.compileTemplate({
      source: parsed.descriptor.template.content,
      filename,
      id: "9180",
      compilerOptions: { bindingMetadata: script.bindings },
    });
    assert.deepEqual(template.errors, []);
    return `${script.content}\n${template.code}\n__component.render = render;\nexport default __component;`;
  };
  input.parent = compile(input.parent, "Parent.vue");
  input.children = input.children.map((source) => compile(source, "Child.vue"));
}
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
  "SVGElement",
  "Event",
  "ShadowRoot",
])
  globalThis[key] = key === "window" ? window : window[key];

const runtime = fromUi.resolve(
  `vue-vapor-runtime-supported/dist/vue.runtime-with-vapor.esm-browser${input.production ? ".prod" : ""}.js`,
);
const originalInfo = console.info;
console.info = (...args) => console.error(...args);
const vue = await import(pathToFileURL(runtime).href);
console.info = originalInfo;
const diagnostics = [];
const originalWarn = console.warn;
const originalError = console.error;
console.warn = (...args) => diagnostics.push(args.map(String).join(" "));
console.error = (...args) => diagnostics.push(args.map(String).join(" "));
const events = [];
globalThis.__hmr9180Modules = { vue, "./fixture": { record: (value) => events.push(value) } };

async function evaluate(code: string) {
  const compiled = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            const module = globalThis.__hmr9180Modules[source];
            assert.ok(module, source);
            path.replaceWith(
              t.variableDeclaration(
                "const",
                path.node.specifiers.map((specifier) => {
                  const name = t.isImportDefaultSpecifier(specifier)
                    ? "default"
                    : specifier.imported.name;
                  assert.ok(Object.hasOwn(module, name), `${source}: ${name}`);
                  return t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(
                      t.memberExpression(
                        t.memberExpression(
                          t.identifier("globalThis"),
                          t.identifier("__hmr9180Modules"),
                        ),
                        t.stringLiteral(source),
                        true,
                      ),
                      t.stringLiteral(name),
                      true,
                    ),
                  );
                }),
              ),
            );
          },
        },
      }),
    ],
  }).code;
  return (await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`))
    .default;
}

const child = await evaluate(input.children[0]);
child.__hmrId = "9180-child";
globalThis.__hmr9180Modules["./Child.vue"] = { default: child };
const parent = await evaluate(input.parent);
parent.__hmrId = "9180-parent";
if (!input.production) {
  assert.equal(globalThis.__VUE_HMR_RUNTIME__.createRecord(child.__hmrId, child), true);
  assert.equal(globalThis.__VUE_HMR_RUNTIME__.createRecord(parent.__hmrId, parent), true);
}
const app = (input.vapor ? vue.createVaporApp : vue.createApp)(parent);
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("div");
window.document.body.append(host);
const snapshots = [];
const expected = input.expected[input.production ? "production" : "development"];
const observe = (node) => {
  if (node.nodeType === 3) return node.textContent;
  if (node.nodeType !== 1) return null;
  return {
    tag: node.localName,
    namespace: node.namespaceURI,
    attributes: Object.fromEntries(
      [...node.attributes]
        .map((attr) => [attr.name, attr.value])
        .sort(([left], [right]) => left.localeCompare(right)),
    ),
    children: [...node.childNodes].map(observe).filter((child) => child !== null && child !== ""),
  };
};
const snapshot = () => {
  const [count, childText] = expected[snapshots.length];
  assert.deepEqual(diagnostics, [], "HMR must not request a full reload or report render errors");
  assert.equal(
    host.querySelector("#increment")?.textContent,
    String(count),
    "parent setup state survives child reload",
  );
  assert.equal(host.querySelector(".child")?.textContent, childText);
  snapshots.push({ tree: [...host.childNodes].map(observe).filter(Boolean), events: [...events] });
};
try {
  app.mount(host);
  await vue.nextTick();
  snapshot();
  for (let count = 1; count <= 3; count++) {
    host.querySelector("#increment").click();
    await vue.nextTick();
    snapshot();
    if (!input.production && count < 3) {
      const oldChild = host.querySelector(".child");
      const replacement = await evaluate(input.children[count]);
      replacement.__hmrId = child.__hmrId;
      // Vue's real reload API rerenders the Vapor parent; calling render by hand
      // would miss the setupState failure reported by the consumer.
      globalThis.__VUE_HMR_RUNTIME__.reload(child.__hmrId, replacement);
      await vue.nextTick();
      snapshot();
      assert.equal(oldChild.isConnected, false, "superseded child must be removed");
    }
  }
  assert.equal(snapshots.length, expected.length, "all authored phases must run");
  app.unmount();
  await vue.nextTick();
  assert.equal(host.childNodes.length, 0, "unmount removes the whole rendered tree");
  assert.deepEqual(
    events,
    input.expected.finalEvents,
    "child reload does not recreate parent setup",
  );
  assert.deepEqual(diagnostics, []);
  snapshots.push({ tree: [], events: [...events] });
  process.stdout.write(JSON.stringify({ version, snapshots }));
} finally {
  if (host.childNodes.length) app.unmount();
  console.warn = originalWarn;
  console.error = originalError;
  delete globalThis.__hmr9180Modules;
  host.remove();
  await window.happyDOM.close();
}
