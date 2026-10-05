// #7822. Execute complete source-built modules against independent pinned
// Vue compilers, including SSR's push and vnode-slot fallback branches.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { officialCompilerVapor, vueVaporVersion } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const vapor = input.target === "vapor";
const ssr = ["ssr", "vapor-ssr-fallback"].includes(input.target);
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
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const compiler = fromUi("vue/compiler-sfc");
const vue = vapor ? await loadRuntime({ production: false }) : fromUi("vue");
const server = fromUi("vue/server-renderer");
assert.equal(vue.version, vapor ? "3.6.0-rc.9" : "3.5.35");
if (vapor) assert.equal(vueVaporVersion, vue.version);
globalThis.__slotCommentRuntime = { vue, server };

async function load(code) {
  const body = code.replace(
    /import\s*\{([^}]*)\}\s*from\s*["'](vue|@vue\/server-renderer|vue\/server-renderer)["'];?/gu,
    (_, names, source) =>
      `const {${names.replace(/\s+as\s+/gu, ": ")}} = globalThis.__slotCommentRuntime.${source === "vue" ? "vue" : "server"};`,
  );
  const module = await import(
    `data:text/javascript;base64,${Buffer.from(body).toString("base64")}`
  );
  return (
    module.default ??
    (vapor
      ? { __vapor: true, render: module.render }
      : ssr
        ? { ssrRender: module.ssrRender }
        : { render: module.render })
  );
}

function reference(source, serverTarget = ssr) {
  const parsed = compiler.parse(source, { filename: "Reported.vue" });
  assert.deepEqual(parsed.errors, []);
  const template = parsed.descriptor.template.content;
  if (vapor) {
    const errors = [];
    const result = officialCompilerVapor.compile(template, {
      mode: "module",
      comments: true,
      onError: (error) => errors.push(error),
    });
    assert.deepEqual(errors, []);
    return result.code;
  }
  const result = compiler.compileTemplate({
    source: template,
    filename: "Reported.vue",
    id: "probe",
    ssr: serverTarget,
    ssrCssVars: [],
    compilerOptions: { comments: true },
  });
  assert.deepEqual(result.errors, []);
  assert.deepEqual(result.tips, []);
  return result.code;
}

const slotChildSource = "<template><section><slot><em>fallback</em></slot></section></template>";
const childCode = reference(slotChildSource);
const cases = [];

async function observe(code, mode) {
  const component = await load(code);
  const slotCalls = [];
  const Confirm = vapor
    ? vue.defineVaporComponent({
        name: "Confirm",
        setup: () => {
          const button = window.document.createElement("button");
          button.textContent = "confirm";
          return button;
        },
      })
    : { name: "Confirm", render: () => vue.h("button", "confirm") };
  const Dialog =
    mode === "slot-probe"
      ? {
          name: "Dialog",
          render() {
            const nodes = this.$slots.default?.() ?? [];
            slotCalls.push({
              keys: Object.keys(this.$slots).sort(),
              count: nodes.length,
              types: nodes.map((node) =>
                node.type === vue.Comment
                  ? "comment"
                  : node.type === Confirm
                    ? "Confirm"
                    : String(node.type),
              ),
            });
            return vue.h("section", null, this.$slots.default ? nodes : vue.h("em", "fallback"));
          },
        }
      : await load(childCode);
  const app = (vapor ? vue.createVaporApp : ssr ? vue.createSSRApp : vue.createApp)(component);
  app.component("Dialog", Dialog);
  app.component("Confirm", Confirm);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  if (ssr) return { html: await server.renderToString(app), diagnostics, slotCalls };
  const host = window.document.createElement("div");
  window.document.body.append(host);
  try {
    app.mount(host);
    await vue.nextTick();
    const walker = window.document.createTreeWalker(host, 128);
    const comments = [];
    while (walker.nextNode()) {
      const text = walker.currentNode.data;
      if (text && !["[", "]"].includes(text)) comments.push(text);
    }
    const result = { tree: observeChildren(host), comments, diagnostics, slotCalls };
    app.unmount();
    await vue.nextTick();
    assert.equal(host.childNodes.length, 0, "unmount must remove the whole tree");
    result.afterUnmount = [];
    return result;
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}

try {
  for (const fixture of input.cases) {
    assert.deepEqual(Object.keys(fixture.current).sort(), [
      "bindings",
      "code",
      "css",
      "errors",
      "macroArtifacts",
      "map",
      "warnings",
    ]);
    const officialCode = reference(fixture.source);
    const modes = vapor ? ["slot-outlet"] : ["slot-probe", "slot-outlet"];
    const observations = [];
    for (const mode of modes) {
      const expected = await observe(officialCode, mode);
      const actual = await observe(fixture.current.code, mode);
      assert.deepEqual(actual, expected, `${input.target}/${fixture.name}/${mode}`);
      assert.deepEqual(actual.diagnostics, [], `${fixture.name}: diagnostics`);
      if (ssr) {
        assert.equal(actual.html.includes("<!-- note -->"), !fixture.drop, fixture.name);
      } else {
        assert.deepEqual(actual.comments, fixture.drop ? [] : [" note "], fixture.name);
      }
      if (mode === "slot-probe") {
        const absent = fixture.name === "comment-only";
        const count = absent
          ? 0
          : ["explicit-default", "ordinary-default"].includes(fixture.name)
            ? 2
            : 1;
        assert.equal(actual.slotCalls.length, 1);
        assert.equal(actual.slotCalls[0].count, count, fixture.name);
        assert.equal(actual.slotCalls[0].keys.includes("default"), !absent, fixture.name);
      }
      observations.push({ mode, expected, actual });
    }
    cases.push({
      name: fixture.name,
      source: fixture.source,
      current: fixture.current,
      officialCode,
      observations,
    });
  }
  process.stdout.write(JSON.stringify({ target: input.target, version: vue.version, cases }));
} finally {
  delete globalThis.__slotCommentRuntime;
  await window.happyDOM.close();
}
