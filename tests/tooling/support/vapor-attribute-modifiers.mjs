import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { loadRuntime } from "./davinci-mounted-trace.mjs";

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
  readFileSync(new URL("vapor-attribute-modifiers.manifest.json", fixtureRoot)),
);
for (const entry of manifest.files) {
  const bytes = readFileSync(new URL(`vapor-attribute-modifiers/${entry.path}`, fixtureRoot));
  assert.equal(bytes.length, entry.bytes);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.sha256);
  if (entry.path === `${input.filename}.txt`) assert.equal(bytes.toString("utf8"), input.source);
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
const parsed = compiler.parse(input.source, { filename: input.filename });
assert.deepEqual(parsed.errors, []);
const reference = compiler.compileScript(parsed.descriptor, {
  id: input.filename,
  isProd: input.production,
  inlineTemplate: true,
  genDefaultAs: "__reference",
});
let serial = 0;
async function evaluate(code) {
  globalThis.__attributeModifierRuntime = vue;
  const transformed = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            assert.ok(["vue", "vue/vapor"].includes(path.node.source.value));
            path.replaceWith(
              t.variableDeclaration(
                "const",
                path.node.specifiers.map((specifier) => {
                  assert.ok(t.isImportSpecifier(specifier));
                  const name = specifier.imported.name;
                  assert.ok(Object.hasOwn(vue, name), name);
                  return t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(
                      t.memberExpression(
                        t.identifier("globalThis"),
                        t.identifier("__attributeModifierRuntime"),
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
function frame(host) {
  const inputElement = host.querySelector("input");
  const divs = [...host.querySelectorAll("div")];
  assert.ok(inputElement);
  return {
    valueAttribute: inputElement.getAttribute("value"),
    divs: divs.map((element) => ({
      id: element.id,
      titleAttribute: element.getAttribute("title"),
      titleProperty: element.title,
    })),
  };
}
const initial =
  input.filename === "App.vue"
    ? { valueAttribute: "x", divs: [{ id: "static", titleAttribute: "x", titleProperty: "x" }] }
    : {
        valueAttribute: "x",
        divs: [
          { id: "static", titleAttribute: "x", titleProperty: "x" },
          { id: "from-object", titleAttribute: "x", titleProperty: "x" },
        ],
      };
const updated = {
  valueAttribute: "y",
  divs: [
    { id: "static", titleAttribute: "y", titleProperty: "y" },
    { id: "next-object", titleAttribute: "y", titleProperty: "y" },
  ],
};
async function trace(code) {
  const component = await evaluate(code);
  const host = window.document.createElement("section");
  window.document.body.append(host);
  const diagnostics = [];
  const app = vue.createVaporApp(component);
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const frames = [];
  try {
    app.mount(host);
    await vue.nextTick();
    assert.deepEqual(diagnostics, []);
    frames.push(frame(host));
    assert.deepEqual(frames[0], initial);
    const button = host.querySelector("button");
    if (button) {
      const nodes = [...host.querySelectorAll("input, div")];
      button.click();
      await vue.nextTick();
      assert.deepEqual(
        [...host.querySelectorAll("input, div")],
        nodes,
        "attribute updates retain node identity",
      );
      frames.push(frame(host));
      assert.deepEqual(frames[1], updated);
      assert.deepEqual(diagnostics, []);
    }
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
  const stock = await trace(`${reference.content}\nexport default __reference;`);
  const actual = await trace(input.code);
  assert.deepEqual(actual, stock);
  process.stdout.write(
    JSON.stringify({ version: vue.version, reference: reference.content, stock, actual }),
  );
} finally {
  delete globalThis.__attributeModifierRuntime;
  await window.happyDOM.close();
}
