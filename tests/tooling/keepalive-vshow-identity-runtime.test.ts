import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const fromUi = createRequire(
  process.env.VIZE_TEST_VUE_PACKAGE ?? new URL("../../npm/ui/package.json", import.meta.url),
);
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const vue = fromVue("vue");
const compiler = fromVue("@vue/compiler-dom");

type HostNode = {
  type: string;
  text: string;
  children: HostNode[];
  parent: HostNode | null;
  props: Record<string, unknown>;
  style: { display: string };
};
function element(type: string, text = ""): HostNode {
  return { type, text, children: [], parent: null, props: {}, style: { display: "" } };
}
function detach(child: HostNode) {
  if (!child.parent) return;
  const siblings = child.parent.children;
  const index = siblings.indexOf(child);
  if (index >= 0) siblings.splice(index, 1);
  child.parent = null;
}
function renderer() {
  return vue.createRenderer({
    patchProp(node: HostNode, key: string, _before: unknown, value: unknown) {
      node.props[key] = value;
    },
    insert(child: HostNode, parent: HostNode, anchor: HostNode | null = null) {
      detach(child);
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      if (index < 0) parent.children.push(child);
      else parent.children.splice(index, 0, child);
      child.parent = parent;
    },
    remove: detach,
    createElement: element,
    createText: (text: string) => element("#text", text),
    createComment: (text: string) => element("#comment", text),
    setText(node: HostNode, text: string) {
      node.text = text;
    },
    setElementText(node: HostNode, text: string) {
      node.text = text;
      node.children = [];
    },
    parentNode: (node: HostNode) => node.parent,
    nextSibling: (node: HostNode) =>
      node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  });
}
function tree(node: HostNode): unknown {
  return {
    type: node.type,
    text: node.text,
    display: node.style.display,
    children: node.children.map(tree),
  };
}

const wrapped = (body: string) => `<KeepAlive><div v-show="opened">${body}</div></KeepAlive>`;
const direct = (body: string) => `<KeepAlive>${body}</KeepAlive>`;
const fixturePath = new URL("../_fixtures/keepalive-vshow-identity.json", import.meta.url);
const corpus = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const fixtures: {
  id: string;
  transition: string;
  source: string;
  repair: string;
  warning: boolean;
}[] = corpus.cases;

async function observe(source: string, transition: string, multiple = false) {
  const events: Record<string, unknown>[] = [];
  const warnings: string[] = [];
  const instances: { id: number; name: string; key: unknown; state: { value: number } }[] = [];
  const state = {
    view: vue.ref("PanelA"),
    visible: vue.ref(true),
    identity: vue.ref("A"),
    items: vue.ref(multiple ? ["A", "B"] : ["A"]),
    opened: vue.ref(true),
  };
  function component(name: string) {
    return {
      name,
      setup() {
        const instance = {
          id: instances.length + 1,
          name,
          key: vue.getCurrentInstance().vnode.key,
          state: vue.ref(0),
        };
        instances.push(instance);
        for (const [event, hook] of [
          ["mounted", vue.onMounted],
          ["unmounted", vue.onUnmounted],
          ["activated", vue.onActivated],
          ["deactivated", vue.onDeactivated],
        ] as const)
          hook(() => events.push({ event, id: instance.id, name, key: instance.key }));
        events.push({ event: "setup", id: instance.id, name, key: instance.key });
        return () => vue.h("span", `${name}:${instance.id}:${instance.state.value}`);
      },
    };
  }
  const compiled = compiler.compile(source, { mode: "function" });
  const render = vue.compile(source);
  const root = element("root");
  const app = renderer().createApp({
    components: {
      PanelA: component("PanelA"),
      PanelB: component("PanelB"),
      BaseTransition: vue.BaseTransition,
    },
    setup: () => state,
    render,
  });
  app.config.warnHandler = (message: string) => warnings.push(message);
  app.mount(root);
  const phases: Record<string, unknown>[] = [];
  function capture(phase: string) {
    phases.push({
      phase,
      tree: tree(root),
      instances: instances.map(({ state, ...identity }) => ({ ...identity, value: state.value })),
      events: [...events],
      warnings: [...warnings],
    });
  }
  await vue.nextTick();
  capture("initial");
  const first = instances[0];
  first.state.value = 42;
  await vue.nextTick();
  capture("mutated");
  state.opened.value = false;
  await vue.nextTick();
  capture("hidden");
  state.opened.value = true;
  await vue.nextTick();
  capture("visible");
  for (const returning of [false, true]) {
    if (transition === "type") state.view.value = returning ? "PanelA" : "PanelB";
    if (transition === "condition") state.visible.value = returning;
    if (transition === "key") state.identity.value = returning ? "A" : "B";
    if (transition === "list")
      state.items.value = returning
        ? multiple
          ? ["A", "B"]
          : ["A"]
        : multiple
          ? ["C", "D"]
          : ["B"];
    await vue.nextTick();
    capture(returning ? "returned" : "away");
  }
  const receipt = {
    source,
    compiled: compiled.code,
    transition,
    multiple,
    phases,
    warnings: [...warnings],
  };
  app.unmount();
  return receipt;
}

function renderedSpanTexts(receipt: Awaited<ReturnType<typeof observe>>, phase: string) {
  const snapshot = receipt.phases.find((item) => item.phase === phase)!;
  const texts: string[] = [];
  function collect(node: any) {
    if (node.type === "span") texts.push(node.text);
    for (const child of node.children) collect(child);
  }
  collect(snapshot.tree);
  return texts;
}

test("KeepAlive native v-show ownership preserves static state but loses changing component identities", async () => {
  assert.match(vue.version, /^3\./);
  assert.equal(corpus.schema, "vize.patina.keepalive-vshow-identity");
  assert.equal(corpus.version, 1);
  const results = [];
  for (const fixture of fixtures) {
    results.push({
      id: fixture.id,
      transition: fixture.transition,
      warning: fixture.warning,
      candidate: await observe(fixture.source, fixture.transition),
      repair: await observe(fixture.repair, fixture.transition),
    });
  }
  const multiple = await observe(
    wrapped('<PanelA v-for="item in items" :key="item" />'),
    "list",
    true,
  );
  const multipleDirect = await observe(
    direct('<PanelA v-for="item in items" :key="item" />'),
    "list",
    true,
  );
  const packageVersions = [
    "vue",
    "@vue/runtime-core",
    "@vue/runtime-dom",
    "@vue/compiler-dom",
    "@vue/compiler-core",
    "@vue/reactivity",
    "@vue/shared",
  ].map((name) => ({
    name,
    path: fromVue.resolve(`${name}/package.json`),
    version: fromVue(`${name}/package.json`).version,
  }));
  assert.ok(packageVersions.every((item) => item.version === vue.version));
  const inputs = new Set([
    import.meta.filename,
    fileURLToPath(fixturePath),
    ...packageVersions.map((item) => item.path),
  ]);
  const pending = [
    fromVue.cache[fromVue.resolve("vue")],
    fromVue.cache[fromVue.resolve("@vue/compiler-dom")],
  ];
  while (pending.length) {
    const loaded = pending.pop();
    if (!loaded || inputs.has(loaded.filename)) continue;
    inputs.add(loaded.filename);
    pending.push(...loaded.children);
  }
  const artifacts = [...inputs].sort().map((path) => {
    const bytes = fs.readFileSync(path);
    return { path, bytes: bytes.length, sha256: createHash("sha256").update(bytes).digest("hex") };
  });
  console.log(
    JSON.stringify({
      schema: "vize.patina.keepalive-vshow-identity-runtime",
      version: 1,
      vueVersion: vue.version,
      nodeVersion: process.version,
      nodeEnv: process.env.NODE_ENV ?? null,
      packageVersions,
      scope:
        "Actual installed Vue compiler and renderer; no Vize candidate/native execution or Vue2 parity claim.",
      artifacts,
      results,
      multiple,
      multipleDirect,
    }),
  );
  for (const { id, warning, candidate, repair } of results) {
    for (const receipt of [candidate, repair]) {
      assert.deepEqual(receipt.warnings, [], id);
      assert.deepEqual(renderedSpanTexts(receipt, "visible"), ["PanelA:1:42"], id);
    }
    assert.deepEqual(renderedSpanTexts(repair, "returned"), ["PanelA:1:42"], id);
    if (!warning) {
      assert.deepEqual(renderedSpanTexts(candidate, "returned"), ["PanelA:1:42"], id);
    } else {
      const final = renderedSpanTexts(candidate, "returned");
      assert.equal(final.length, 1, id);
      assert.match(final[0], /^PanelA:[2-9]\d*:0$/, id);
      const returned = candidate.phases.find((item) => item.phase === "returned")!;
      assert.ok(
        (returned.events as any[]).some((event) => event.event === "unmounted" && event.id === 1),
      );
      const repaired = repair.phases.find((item) => item.phase === "returned")!;
      assert.ok(
        !(repaired.events as any[]).some((event) => event.event === "unmounted" && event.id === 1),
      );
      assert.ok(
        (repaired.events as any[]).filter((event) => event.event === "activated" && event.id === 1)
          .length >= 2,
      );
    }
  }
  assert.deepEqual(multiple.warnings, []);
  // A list beneath KeepAlive is not repaired merely by removing the div:
  // the compiler still emits a Fragment, not one cacheable component VNode.
  for (const receipt of [multiple, multipleDirect]) {
    assert.deepEqual(receipt.warnings, []);
    const returned = receipt.phases.find((phase) => phase.phase === "returned")!;
    const finalIds = renderedSpanTexts(receipt, "returned").map((text) =>
      Number(text.split(":")[1]),
    );
    assert.equal(finalIds.length, 2);
    for (const [index, key] of ["A", "B"].entries()) {
      const instance = (returned.instances as any[]).find((item) => item.id === finalIds[index]);
      assert.equal(instance.key, key);
      assert.equal(instance.value, 0);
      assert.ok(instance.id > 2);
    }
    for (const id of [1, 2]) {
      assert.ok(
        (returned.events as any[]).some((event) => event.event === "unmounted" && event.id === id),
      );
    }
  }
});
