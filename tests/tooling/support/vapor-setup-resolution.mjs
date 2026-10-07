import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVue("vue/compiler-sfc");
const { transformSync } = fromUi("@babel/core");
const { Window } = await import(fromUi.resolve("happy-dom"));
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const fixtureRoot = new URL("../../_fixtures/differential/compiler/", import.meta.url);
const manifest = JSON.parse(
  readFileSync(new URL("vapor-setup-resolution.manifest.json", fixtureRoot)),
);
for (const entry of manifest.files) {
  const bytes = readFileSync(new URL(`vapor-setup-resolution/${entry.path}`, fixtureRoot));
  assert.equal(bytes.length, entry.bytes);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.sha256);
  if (entry.path === `${input.fixture}.vue.txt`) assert.equal(bytes.toString("utf8"), input.source);
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
const vue = await loadRuntime({ production: input.production });
assert.equal(vue.version, compiler.version);
function official(source, filename) {
  const parsed = compiler.parse(source, { filename, ignoreEmpty: false });
  assert.deepEqual(parsed.errors, []);
  const compiled = compiler.compileScript(parsed.descriptor, {
    id: filename,
    isProd: input.production,
    inlineTemplate: true,
    genDefaultAs: "__reference",
  });
  return `${compiled.content}\nexport default __reference;`;
}
const reference = official(input.source, input.filename);
const referenceChild = official(input.child.source, input.child.filename);
let serial = 0;
async function evaluate(code, child) {
  globalThis.__setupResolutionModules = {
    vue,
    "vue/vapor": vue,
    "./Child.vue": { default: child },
  };
  const transformed = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            assert.ok(Object.hasOwn(globalThis.__setupResolutionModules, source), source);
            path.replaceWith(
              t.variableDeclaration(
                "const",
                path.node.specifiers.map((specifier) => {
                  const name = t.isImportDefaultSpecifier(specifier)
                    ? "default"
                    : specifier.imported.name;
                  assert.ok(Object.hasOwn(globalThis.__setupResolutionModules[source], name), name);
                  return t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(
                      t.memberExpression(
                        t.memberExpression(
                          t.identifier("globalThis"),
                          t.identifier("__setupResolutionModules"),
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
  return (
    await import(
      `data:text/javascript;base64,${Buffer.from(`${transformed}\n// ${serial++}`).toString("base64")}`
    )
  ).default;
}
const div = (mark, children) => ({ tag: "div", attributes: { "data-mark": mark }, children });
const tree = (depth) => div("yes", [String(depth), ...(depth < 2 ? [tree(depth + 1)] : [])]);
const expected =
  input.fixture === "Tree"
    ? [tree(0)]
    : input.fixture === "Imported"
      ? [div("local", [{ tag: "i", attributes: {}, children: ["imported"] }])]
      : [
          {
            tag: "div",
            attributes: { "data-global": "registry", "data-local": "yes" },
            children: ["mixed"],
          },
        ];
async function trace(code, childCode) {
  const child = await evaluate(childCode);
  const component = await evaluate(code, child);
  const host = window.document.createElement("section");
  window.document.body.append(host);
  const diagnostics = [];
  const props = vue.reactive({ depth: 0 });
  const app = vue.createVaporApp(component, input.fixture === "Tree" ? props : undefined);
  app.directive("global-mark", (element) => {
    element.dataset.global = "registry";
  });
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const frames = [];
  try {
    app.mount(host);
    await vue.nextTick();
    assert.deepEqual(diagnostics, []);
    frames.push(observeChildren(host));
    assert.deepEqual(frames[0], expected);
    if (input.fixture === "Tree") {
      const roots = [...host.querySelectorAll("div")];
      props.depth = 1;
      await vue.nextTick();
      const updated = [...host.querySelectorAll("div")];
      assert.equal(updated[0], roots[0]);
      assert.equal(updated[1], roots[1]);
      assert.equal(roots[2].isConnected, false, "removed recursive branch detached");
      frames.push(observeChildren(host));
      assert.deepEqual(frames[1], [tree(1)]);
      props.depth = 0;
      await vue.nextTick();
      frames.push(observeChildren(host));
      assert.deepEqual(frames[2], [tree(0)]);
      assert.equal(host.querySelector("div"), roots[0]);
    }
    assert.deepEqual(diagnostics, []);
    app.unmount();
    await vue.nextTick();
    assert.equal(host.childNodes.length, 0);
    assert.deepEqual(diagnostics, []);
    return frames;
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}
try {
  const stock = await trace(reference, referenceChild);
  const actual = await trace(input.code, input.child.code);
  assert.deepEqual(actual, stock);
  process.stdout.write(JSON.stringify({ version: vue.version, reference, stock, actual }));
} finally {
  delete globalThis.__setupResolutionModules;
  await window.happyDOM.close();
}
