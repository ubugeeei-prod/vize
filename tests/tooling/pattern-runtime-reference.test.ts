// The rendering contract of the patterned-template reference implementation
// (vuejs/core#15531, `packages/vue/__tests__/vMatch.spec.ts`), run against
// Vize's own client and server output with the installed Vue runtime.
import assert from "node:assert/strict";
import { test } from "node:test";
import { buildPatternComponents, installDom } from "./support/upstream/pattern-runtime.ts";

await installDom();
const { createApp, createSSRApp, nextTick } = await import("vue");
const { renderToString } = await import("vue/server-renderer");

const stateModule = (body: string) => `import { ref } from "vue";\n${body}\n`;
const sfc = (imports: string, template: string, extra = "") =>
  `<script setup>\nimport { ${imports} } from "./state.mjs";\n${extra}</script>\n\n${template}\n`;

for (const wrapper of ["", "Transition", "KeepAlive"]) {
  test(`single-root match preserves fallthrough through ${wrapper || "no wrapper"}`, async () => {
    const match = `<template v-match="state"><p v-when="{ const text }">{{ text }}</p><i v-when="_">empty</i></template>`;
    const build = buildPatternComponents(
      {
        "App.vue": sfc(
          "state",
          wrapper ? `<template><${wrapper}>${match}</${wrapper}></template>` : match,
        ),
      },
      { "state.mjs": stateModule(`export const state = ref({ text: "a" });`) },
    );
    try {
      const { default: ServerApp } = await build.load<{ default: object }>("ssr", "App.js");
      const { default: ClientApp } = await build.load<{ default: object }>("dom", "App.js");
      const server = await renderToString(createSSRApp(ServerApp, { id: "fallthrough" }));
      assert.equal(server, '<p id="fallthrough">a</p>');
      const root = document.createElement("div");
      root.innerHTML = server;
      const first = root.querySelector("p");
      const app = createSSRApp(ClientApp, { id: "fallthrough" });
      app.mount(root);
      assert.equal(root.querySelector("p"), first);
      assert.equal(first!.id, "fallthrough");
      app.unmount();
    } finally {
      build.dispose();
    }
  });
}

test("explicit arm keys react within the binding scope", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": sfc(
        "state",
        `<template v-match="state"><template v-when="{ const id }" :key="id"><input :value="id"/></template></template>`,
      ),
    },
    { "state.mjs": stateModule(`export const state = ref({ id: "a" });`) },
  );
  try {
    const { default: App } = await build.load<{ default: object }>("dom", "App.js");
    const { state } = await build.load<{ state: { value: { id: string } } }>("dom", "state.mjs");
    const root = document.createElement("div");
    const app = createApp(App);
    app.mount(root);
    const first = root.querySelector("input");
    state.value.id = "b";
    await nextTick();
    assert.notEqual(root.querySelector("input"), first);
    assert.equal(root.querySelector("input")!.value, "b");
    app.unmount();
  } finally {
    build.dispose();
  }
});

test("client and server agree, and hydration survives reactive arm changes", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": sfc(
        "result, read, received",
        `<template><section v-match="read()"><button v-when="{ kind: 'ok', const data } if (data > 0)" @click="received.push(data)">{{ data }}</button><p v-when="{ kind: 'error', ...const rest }">{{ rest.message }}</p><template v-when="_"></template></section></template>`,
      ),
    },
    {
      "state.mjs": stateModule(
        `export const result = ref({ kind: "ok", data: 1 });\nexport const calls = { read: 0 };\nexport const read = () => (calls.read++, result.value);\nexport const received = [];`,
      ),
    },
  );
  try {
    const { default: ServerApp } = await build.load<{ default: object }>("ssr", "App.js");
    const { default: ClientApp } = await build.load<{ default: object }>("dom", "App.js");
    const { result, calls, received } = await build.load<{
      result: { value: unknown };
      calls: { read: number };
      received: unknown[];
    }>("dom", "state.mjs");
    const root = document.createElement("div");
    root.innerHTML = await renderToString(createSSRApp(ServerApp));
    assert.equal(calls.read, 1, "the subject is evaluated once per server render");
    const button = root.querySelector("button")!;
    const app = createSSRApp(ClientApp);
    app.mount(root);
    assert.equal(root.querySelector("button"), button);
    assert.equal(calls.read, 2, "the subject is evaluated once per client render");
    button.click();
    assert.deepEqual(received, [1]);
    result.value = { kind: "ok", data: 2 };
    await nextTick();
    assert.equal(root.querySelector("button"), button);
    button.click();
    assert.deepEqual(received, [1, 2]);
    assert.equal(calls.read, 3);
    result.value = { kind: "error", message: "<failure>" };
    await nextTick();
    assert.equal(root.textContent, "<failure>");
    assert.equal(root.querySelector("button"), null);
    const server = document.createElement("div");
    server.innerHTML = await renderToString(createSSRApp(ServerApp));
    assert.equal(server.textContent, root.textContent);
    assert.equal(server.querySelector("p")!.outerHTML, root.querySelector("p")!.outerHTML);
    const serverParagraph = server.querySelector("p");
    const hydrated = createSSRApp(ClientApp);
    hydrated.mount(server);
    assert.equal(server.querySelector("p"), serverParagraph);
    hydrated.unmount();
    result.value = null;
    await nextTick();
    assert.equal(root.textContent, "");
    app.unmount();
  } finally {
    build.dispose();
  }
});

test("identical tags in different arms get distinct identity", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": sfc(
        "state",
        `<template v-match="state"><input v-when="'a'" value="a"/><input v-when="'b'" value="b"/></template>`,
      ),
    },
    { "state.mjs": stateModule(`export const state = ref("a");`) },
  );
  try {
    const { default: App } = await build.load<{ default: object }>("dom", "App.js");
    const { state } = await build.load<{ state: { value: string } }>("dom", "state.mjs");
    const root = document.createElement("div");
    const app = createApp(App);
    app.mount(root);
    const first = root.querySelector("input")!;
    first.value = "edited";
    state.value = "b";
    await nextTick();
    assert.notEqual(root.querySelector("input"), first);
    assert.equal(root.querySelector("input")!.value, "b");
    state.value = "unmatched";
    await nextTick();
    assert.equal(root.querySelector("input"), null);
    app.unmount();
  } finally {
    build.dispose();
  }
});

test("nested scopes do not leak and array rest stays reactive", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": sfc(
        "items",
        `<template><div v-for="item in items"><template v-match="item"><template v-when="[const head, ...const tail]"><template v-match="head"><b v-when="1">one:{{ tail.join(',') }}</b><b v-when="const value">{{ value }}:{{ tail.length }}</b></template></template><i v-when="[]">empty</i></template></div></template>`,
      ),
    },
    { "state.mjs": stateModule(`export const items = ref([[1, 2], [3]]);`) },
  );
  try {
    const { default: App } = await build.load<{ default: object }>("dom", "App.js");
    const { items } = await build.load<{ items: { value: number[][] } }>("dom", "state.mjs");
    const root = document.createElement("div");
    const app = createApp(App);
    app.mount(root);
    assert.equal(root.textContent, "one:23:0");
    items.value[0].push(4);
    items.value[1] = [];
    await nextTick();
    assert.equal(root.textContent, "one:2,4empty");
    app.unmount();
  } finally {
    build.dispose();
  }
});

test("server render and hydration preserve shadowed loop bindings and props", async () => {
  const build = buildPatternComponents(
    {
      "App.vue": sfc(
        "rows",
        `<template><main><div v-for="value in rows"><template v-match="value"><section v-when="{ const value }" :title="value"><b v-for="value in [1, 2]">{{ value }}</b><i>{{ value }}</i></section></template><p>{{ value.value }}</p></div><footer>{{ value }}</footer></main></template>`,
        `defineProps(["value"]);\n`,
      ),
    },
    { "state.mjs": stateModule(`export const rows = ref([{ value: "arm" }]);`) },
  );
  try {
    const { default: ServerApp } = await build.load<{ default: object }>("ssr", "App.js");
    const { default: ClientApp } = await build.load<{ default: object }>("dom", "App.js");
    const { rows } = await build.load<{ rows: { value: Array<{ value: string }> } }>(
      "dom",
      "state.mjs",
    );
    const root = document.createElement("div");
    root.innerHTML = await renderToString(createSSRApp(ServerApp, { value: "prop" }));
    assert.equal(root.textContent, "12armarmprop");
    const section = root.querySelector("section")!;
    assert.equal(section.title, "arm");
    const app = createSSRApp(ClientApp, { value: "prop" });
    app.mount(root);
    assert.equal(root.querySelector("section"), section);
    rows.value[0].value = "updated";
    await nextTick();
    assert.equal(root.textContent, "12updatedupdatedprop");
    assert.equal(section.title, "updated");
    app.unmount();
  } finally {
    build.dispose();
  }
});
