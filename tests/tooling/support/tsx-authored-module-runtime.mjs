import assert from "node:assert/strict";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { Window } from "happy-dom";
import { build } from "vite-plus";
import { vueVaporRuntimeEntry } from "./vue-vapor-release.mjs";

let payload = "";
for await (const chunk of process.stdin) payload += chunk;
const modules = JSON.parse(payload);
const plainModule = modules.plain;
delete modules.plain;
const window = new Window();
for (const key of [
  "window",
  "document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Event",
]) {
  globalThis[key] = key === "window" ? window : window[key];
}
const messages = [];
for (const [scenario, module] of Object.entries(modules)) {
  const entry = fileURLToPath(new URL("./tsx-authored-module-fixture.ts", import.meta.url));
  const child = "virtual:tsx-authored-child.ts";
  const plain = "virtual:tsx-authored-plain.ts";
  const compiled = await build({
    configFile: false,
    logLevel: "silent",
    define: {
      "process.env.NODE_ENV": JSON.stringify("development"),
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
    plugins: [
      {
        name: "tsx-authored-module-fixture",
        resolveId(id) {
          if (id === "vue") return vueVaporRuntimeEntry;
          if (id === entry || id === child || id === plain) return `\0${id}`;
          if (id === "./Child") return `\0${child}`;
          if (id === "./Plain") return `\0${plain}`;
        },
        load(id) {
          if (id === `\0${entry}`)
            return `${module}\nexport { createApp, h, reactive, nextTick } from "vue";`;
          if (id === `\0${child}`)
            return `import { h } from "vue"; export default { props: ['label'], setup(props, { slots }) { return () => h('strong', slots.default ? [slots.default({ label: props.label }), slots.footer?.({ label: props.label }), slots.outer?.()] : props.label); } };`;
          if (id === `\0${plain}`) return plainModule;
        },
      },
    ],
    build: { write: false, minify: false, lib: { entry, formats: ["es"] } },
  });
  const outputs = Array.isArray(compiled) ? compiled : [compiled];
  const chunks = outputs.flatMap((output) => output.output.filter((item) => item.type === "chunk"));
  assert.equal(chunks.length, 1);
  const vue = await import(
    `data:text/javascript;base64,${Buffer.from(chunks[0].code).toString("base64")}`
  );
  const state = vue.reactive({ label: "first" });
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const component =
    scenario === "mixed"
      ? {
          setup: () => () =>
            vue.h("section", [vue.h(vue.Stateful), vue.h(vue.Pure, { label: state.label })]),
        }
      : scenario === "typed"
        ? { setup: () => () => vue.h(vue.Typed, { label: state.label }) }
        : { setup: () => () => vue.h(vue.default, { label: state.label }) };
  const app = vue.createApp(component);
  app.config.warnHandler = (message) => messages.push(message);
  app.config.errorHandler = (error) => messages.push(String(error));
  app.mount(host);
  if (scenario === "expression") {
    assert.equal(vue.suffix, "!");
    assert.equal(host.querySelector("strong").textContent, "first!");
    state.label = "second";
    await vue.nextTick();
    assert.equal(host.querySelector("strong").textContent, "second!");
  } else if (scenario === "mixed") {
    assert.equal(vue.default, vue.Stateful);
    host.querySelector("button").dispatchEvent(new window.Event("click", { bubbles: true }));
    await vue.nextTick();
    assert.equal(host.querySelector("button").textContent, "1");
    state.label = "second";
    await vue.nextTick();
    assert.equal(host.querySelector("button").textContent, "1");
    assert.equal(host.querySelector("i").textContent, "second");
  } else if (scenario === "default") {
    assert.equal(host.textContent, "i");
  } else if (scenario === "typed") {
    assert.equal(vue.default(), "retained");
    assert.equal(vue.afterTyped, "retained-after-setup");
    assert.equal(host.textContent, "first");
    state.label = "second";
    await vue.nextTick();
    assert.equal(host.textContent, "second");
  } else if (scenario === "options") {
    assert.equal(host.textContent, "lexical:lexical");
    assert.equal(vue.widgets.marker, "kept");
    assert.equal(vue.widgets.render().type, "i");
  } else if (scenario === "factory") {
    assert.equal(host.textContent, "retained-factory");
    assert.deepEqual(vue.make("other")().children, ["other"]);
    state.label = "second";
    await vue.nextTick();
    assert.equal(host.textContent, "retained-factory");
  } else if (scenario === "slots") {
    assert.equal(host.querySelector("i").textContent, "first");
    assert.equal(host.querySelector("b").textContent, "first");
    assert.equal(host.querySelector("u").textContent, "outer");
    state.label = "second";
    await vue.nextTick();
    assert.equal(host.querySelector("i").textContent, "second");
    assert.equal(host.querySelector("b").textContent, "second");
    assert.equal(host.querySelector("u").textContent, "outer");
  } else {
    assert.fail(`unknown scenario ${scenario}`);
  }
  app.unmount();
  assert.equal(host.childNodes.length, 0);
  host.remove();
}
assert.deepEqual(messages, []);
await window.happyDOM.abort();
console.log("7 mounted TSX module scenarios passed");
