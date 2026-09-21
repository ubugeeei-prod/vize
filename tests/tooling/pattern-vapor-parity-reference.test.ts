// VDOM / Vapor parity for patterned templates: the scenarios of the reference
// implementation's `packages/runtime-vapor/__tests__/vMatch.spec.ts`
// (vuejs/core#15531), compiled by Vize for both renderers and run against one
// Vue build that carries both.
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  buildPatternComponents,
  installDom,
  vaporRuntime,
} from "./support/upstream/pattern-runtime.ts";

await installDom();
const vue = await import(vaporRuntime);

type Data = { value: any };
type Outcome = { after: string; text: string };

/** `<script setup>` every ported component starts from unless it brings its own. */
const script = (names: string[]) =>
  `<script setup>\nimport { data } from "./state.mjs";\n${names
    .map((name) => `import ${name} from "./${name}.vue";\n`)
    .join("")}const components = { ${names.join(", ")} };\n`;

async function renderParity(
  sources: Record<string, string>,
  initial: string,
  act: (data: Data, root: HTMLElement) => void | Promise<void>,
  renderers: ReadonlyArray<"vdom" | "vapor"> = ["vdom", "vapor"],
): Promise<{ vdom: Outcome; vapor: Outcome }> {
  const others = Object.keys(sources).filter((name) => name !== "App");
  const components = Object.fromEntries(
    Object.entries(sources).map(([name, source]) => {
      const imports = name === "App" ? others : others.filter((other) => other !== name);
      const [extra, template] = source.startsWith("<script setup>")
        ? [
            source.slice("<script setup>".length, source.indexOf("</script>")),
            source.slice(source.indexOf("</script>") + "</script>".length),
          ]
        : ["", source];
      return [`${name}.vue`, `${script(imports)}${extra}</script>\n${template}\n`];
    }),
  );
  const results = {} as { vdom: Outcome; vapor: Outcome };
  for (const renderer of renderers) {
    const backend = renderer === "vapor" ? "vapor" : "dom";
    const build = buildPatternComponents(
      components,
      { "state.mjs": `import { ref } from "vue";\nexport const data = ref(${initial});\n` },
      { backends: [backend], runtime: vaporRuntime },
    );
    try {
      const { default: App } = await build.load<{ default: object }>(backend, "App.js");
      const { data } = await build.load<{ data: Data }>(backend, "state.mjs");
      // Vapor delegates events to the document, so the root must be attached.
      const root = document.body.appendChild(document.createElement("div"));
      const app = renderer === "vapor" ? vue.createVaporApp(App) : vue.createApp(App);
      try {
        app.use(vue.vaporInteropPlugin).mount(root);
        try {
          await act(data, root);
        } catch (error) {
          throw new Error(`${renderer}: ${(error as Error).message}`, { cause: error });
        }
        await vue.nextTick();
        results[renderer] = { after: root.innerHTML, text: root.textContent! };
      } finally {
        // A failed scenario must not leave its app mounted under `document.body`
        // for the scenarios that follow.
        app.unmount();
        root.remove();
      }
    } finally {
      build.dispose();
    }
  }
  return results;
}

const compact = (root: HTMLElement) => root.textContent!.replace(/\s/g, "");

test("root match preserves bindings, nested scopes and branch identity", async () => {
  await renderParity(
    {
      Panel: '<template><slot :text="data.slot"/></template>',
      App: `<template v-match="data.result">
        <section v-when="{ kind: 'ok', const text }" :title="text">
          <button @click="data.clicked.push(text)">{{ text }}</button>
          <b v-for="text in text">{{ text }}</b>
          <components.Panel v-slot="{ text }"><i>{{ text }}</i></components.Panel>
          <template v-match="text"><em v-when="const text">{{ text }}</em></template>
        </section>
        <p v-when="_">empty</p>
      </template>`,
    },
    `{ result: { kind: "ok", text: "ab" }, slot: "slot", clicked: [] }`,
    async (data, root) => {
      const section = root.querySelector("section")!;
      const button = root.querySelector("button")!;
      assert.equal(section.textContent!.replace(/\s/g, ""), "ababslotab");
      button.click();
      data.value.result.text = "cd";
      data.value.slot = "updated";
      await vue.nextTick();
      assert.equal(root.querySelector("section"), section);
      assert.equal(section.title, "cd");
      assert.equal(section.textContent!.replace(/\s/g, ""), "cdcdupdatedcd");
      button.click();
      assert.deepEqual([...data.value.clicked], ["ab", "cd"]);
      data.value.result.kind = "err";
      await vue.nextTick();
      assert.equal(root.querySelector("section"), null);
      assert.equal(root.textContent, "empty");
    },
  );
});

// Vapor has no keyed fragment yet (`createKeyedFragment`), for `v-if` branches
// as much as for arms, so an explicit key cannot remount there. Tracked in
// ubugeeei-prod/vize#6176; the scenario pins the VDOM renderer until then.
test("template arm keys remount while empty arms stop fallthrough", async () => {
  await renderParity(
    {
      App: `<template><template v-match="data"><template v-when="{ const id }" :key="id"><input :value="id"/></template><template v-when="null"></template><b v-when="_">fallback</b></template></template>`,
    },
    `{ id: "a" }`,
    async (data, root) => {
      const first = root.querySelector("input");
      data.value = { id: "b" };
      await vue.nextTick();
      assert.notEqual(root.querySelector("input"), first);
      data.value = null;
      await vue.nextTick();
      assert.equal(root.textContent, "");
      data.value = 42;
      await vue.nextTick();
      assert.equal(root.textContent, "fallback");
    },
    ["vdom"],
  );
});

for (const tag of ["input", "template"]) {
  test(`v-once retains bindings and arm identity on ${tag}`, async () => {
    const arm =
      tag === "input"
        ? `<input v-when="{ kind: 'a', const value }" v-once :value="value"/>`
        : `<template v-when="{ kind: 'a', const value }" v-once><input :value="value"/></template>`;
    await renderParity(
      {
        App: `<template><template v-match="data">${arm}<input v-when="{ kind: 'b', const value }" :value="value"/></template></template>`,
      },
      `{ kind: "a", value: "first" }`,
      async (data, root) => {
        const first = root.querySelector("input")!;
        assert.equal(first.value, "first");
        data.value.value = "updated";
        await vue.nextTick();
        assert.equal(root.querySelector("input"), first);
        assert.equal(first.value, "first");
        data.value = { kind: "b", value: "second" };
        await vue.nextTick();
        assert.notEqual(root.querySelector("input"), first);
        assert.equal(root.querySelector("input")!.value, "second");
      },
    );
  });
}

test("reactive selection, guards, rest and event binding lifetime", async () => {
  const results = await renderParity(
    {
      App: `<template><section v-match="data.value"><button v-when="{ kind: 'ok', const value, ...const rest } if (value > 0)" @click="data.clicked.push(value)">{{ value }}:{{ rest.label }}</button><p v-when="_">empty</p></section></template>`,
    },
    `{ value: { kind: "ok", value: 1, label: "first" }, clicked: [] }`,
    async (data, root) => {
      assert.equal(root.textContent, "1:first");
      const button = root.querySelector("button")!;
      button.click();
      data.value.value = { kind: "ok", value: 2, label: "second" };
      await vue.nextTick();
      assert.equal(root.querySelector("button"), button);
      button.click();
      assert.deepEqual([...data.value.clicked], [1, 2]);
      assert.equal(root.textContent, "2:second");
      data.value.value.value = 0;
      await vue.nextTick();
      assert.equal(root.querySelector("button"), null);
    },
  );
  assert.equal(results.vdom.text, "empty");
  assert.equal(results.vapor.text, results.vdom.text);
});

test("branch bindings compose with component props and scoped slots", async () => {
  await renderParity(
    {
      Panel: '<template><div><slot :suffix="data.suffix"/></div></template>',
      App: `<template><template v-match="data.result"><components.Panel v-when="{ const text }" :title="text"><template #default="{ suffix }"><span>{{ text }}:{{ suffix }}</span><template v-match="suffix"><b v-when="const ending">{{ ending }}</b></template></template></components.Panel><i v-when="_">empty</i></template></template>`,
    },
    `{ result: { text: "first" }, suffix: "!" }`,
    async (data, root) => {
      assert.equal(root.textContent, "first:!!");
      assert.equal(root.querySelector("div")!.title, "first");
      data.value.result.text = "second";
      data.value.suffix = "?";
      await vue.nextTick();
      assert.equal(root.textContent, "second:??");
      assert.equal(root.querySelector("div")!.title, "second");
    },
  );
});

test("nested match and array rest react to in-place updates", async () => {
  const results = await renderParity(
    {
      App: `<template><template v-match="data"><template v-when="[const first, ...const tail]"><template v-match="first"><b v-when="1">one:{{ tail.join(',') }}</b><b v-when="const other">{{ other }}</b></template></template><p v-when="[]">empty</p></template></template>`,
    },
    `[1, 2]`,
    async (data, root) => {
      assert.equal(root.textContent, "one:2");
      data.value.push(3);
      await vue.nextTick();
      assert.equal(root.textContent, "one:2,3");
      data.value = [];
    },
  );
  assert.equal(results.vdom.text, "empty");
  assert.equal(results.vapor.text, "empty");
});

test("arm bindings shadow setup, loop and slot bindings without leaking", async () => {
  await renderParity(
    {
      Panel: '<template><slot :value="data.slot"/></template>',
      App: `<script setup>const value = "setup";\n</script><template>
        <div v-for="value in data.rows">
          <template v-match="value">
            <section v-when="{ tag: value.tag, const value } if (value === 'arm')" :title="value">
              <b>{{ value }}</b>
              <i v-for="value in value">{{ value }}</i>
              <components.Panel v-slot="{ value }"><em>{{ value }}</em></components.Panel>
              <strong>{{ value }}</strong>
              <template v-match="value"><small v-when="const value">{{ value }}</small></template>
            </section>
            <u v-when="const value">{{ value.value }}</u>
          </template>
          <p>{{ value.value }}</p>
        </div>
        <footer>{{ value }}</footer>
      </template>`,
    },
    `{ rows: [{ tag: "setup", value: "arm" }, { tag: "setup", value: "other" }], slot: "slot" }`,
    async (data, root) => {
      assert.equal(compact(root), "armarmslotarmarmarmotherothersetup");
      assert.equal(root.querySelector("section")!.title, "arm");
      data.value.slot = "updated";
      data.value.rows[0].value = "fallback";
      await vue.nextTick();
      assert.equal(compact(root), "fallbackfallbackotherothersetup");
    },
  );
});

test("$event is local to an inline handler and shadows an arm binding", async () => {
  await renderParity(
    {
      App: `<template><template v-match="data.value">
        <button v-when="const $event if ($event === 'arm')" :title="$event" @click="data.clicked.push($event.type)">{{ $event }}</button>
        <i v-when="_"/>
      </template></template>`,
    },
    `{ value: "arm", clicked: [] }`,
    (data, root) => {
      const button = root.querySelector("button")!;
      assert.equal(button.title, "arm");
      assert.equal(button.textContent, "arm");
      button.click();
      assert.deepEqual([...data.value.clicked], ["click"]);
    },
  );
});

test("props and component slot parameters occupy different arm scopes", async () => {
  await renderParity(
    {
      Panel: `<script setup>defineProps(["label"]);\n</script><template><div :title="label"><slot :value="data.slot"/></div></template>`,
      Child: `<script setup>defineProps(["value"]);\n</script><template>
        <template v-match="data.row">
          <components.Panel v-when="{ const value } if (value === 'arm')" :label="value" v-slot="{ value }"><b>{{ value }}</b></components.Panel>
          <i v-when="_"/>
        </template>
        <p>{{ value }}</p>
      </template>`,
      App: '<template><components.Child :value="data.prop"/></template>',
    },
    `{ row: { value: "arm" }, prop: "prop", slot: "slot" }`,
    async (data, root) => {
      assert.equal(root.querySelector("div")!.title, "arm");
      assert.equal(root.querySelector("b")!.textContent, "slot");
      assert.equal(root.querySelector("p")!.textContent, "prop");
      data.value.prop = "updated";
      data.value.slot = "nested";
      await vue.nextTick();
      assert.equal(root.querySelector("div")!.title, "arm");
      assert.equal(root.querySelector("b")!.textContent, "nested");
      assert.equal(root.querySelector("p")!.textContent, "updated");
    },
  );
});
