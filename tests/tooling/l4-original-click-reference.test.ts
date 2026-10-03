import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

const pack = JSON.parse(
  fs.readFileSync(
    new URL("../../davinci/vize_l4/tests/fixtures/original-click-vue-3.5.35.json", import.meta.url),
    "utf8",
  ),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const compiler = fromVue("@vue/compiler-dom");
const runtime = fromVue("vue");
const url = (text: string) => `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
const runtimeUrl = url(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    ["openBlock", "createElementBlock", "createElementVNode", "Fragment"]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);

test("five whole original click byte-laws retain pinned code and complete reference maps", () => {
  assert.equal(pack.schema, "vize.original-click-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-dom", version: "3.5.35" });
  assert.equal(fromVue("@vue/compiler-dom/package.json").version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.deepEqual(pack.options, {
    mode: "module",
    hoistStatic: false,
    prefixIdentifiers: true,
    comments: true,
    filename: "Click.vue",
    sourceMap: true,
    bindingMetadata: {},
    cacheHandlers: false,
  });
  assert.equal(pack.fixtures.length, 5);
  assert.equal(new Set(pack.fixtures.map((fixture: any) => fixture.id)).size, 5);
  for (const fixture of pack.fixtures) {
    assert.equal(fixture.source, `<!--雪🌸--><template>${fixture.template}</template>`);
    const measured = compiler.compile(fixture.template, pack.options);
    assert.equal(measured.code, fixture.code, fixture.id);
    assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
  }
});

type Host = {
  type: string;
  children: Host[];
  props: Record<string, any>;
  text: string;
  parent: Host | null;
};
const node = (type: string): Host => ({ type, children: [], props: {}, text: "", parent: null });

function mountedRenderer() {
  // The real pinned Vue renderer owns creation, PROPS patching and unmount.
  // This independent host records those operations without a browser shim.
  const renderer = runtime.createRenderer({
    createElement: (type: string) => node(type),
    createText: (text: string) => ({ ...node("#text"), text }),
    createComment: (text: string) => ({ ...node("#comment"), text }),
    setText: (target: Host, text: string) => {
      target.text = text;
    },
    setElementText: (target: Host, text: string) => {
      target.text = text;
    },
    parentNode: (target: Host) => target.parent,
    nextSibling: (target: Host) => {
      const siblings = target.parent?.children ?? [];
      return siblings[siblings.indexOf(target) + 1] ?? null;
    },
    patchProp: (target: Host, key: string, _previous: any, value: any) => {
      target.props[key] = value;
    },
    insert: (target: Host, parent: Host, anchor: Host | null) => {
      if (target.parent) {
        const previous = target.parent.children;
        previous.splice(previous.indexOf(target), 1);
      }
      target.parent = parent;
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, target);
    },
    remove: (target: Host) => {
      assert(target.parent);
      target.parent.children.splice(target.parent.children.indexOf(target), 1);
      target.parent = null;
    },
  });
  return { renderer, container: node("root") };
}

function button(target: Host): Host | undefined {
  if (target.type === "button") return target;
  for (const child of target.children) {
    const found = button(child);
    if (found) return found;
  }
}

for (const fixture of pack.fixtures) {
  test(`${fixture.id} real Vue mount, callback replacement and unmount preserve event effects`, async () => {
    // The genuine Rust File test compares every native module byte to this
    // complete module. Only this loader's runtime import address is replaced.
    const loaded = await import(
      url(fixture.code.replace('from "vue"', `from ${JSON.stringify(runtimeUrl)}`))
    );
    const forbidden = new Proxy(
      {},
      {
        get(_target, key) {
          throw Error(`unexpected context read ${String(key)}`);
        },
      },
    );
    const { renderer, container } = mountedRenderer();
    let previous: ((event: any) => void) | undefined;
    let actualButton: Host | undefined;
    for (let phase = 0; phase < fixture.events.length; phase++) {
      const vnode = loaded.render(forbidden, [], forbidden, forbidden, forbidden, forbidden);
      renderer.render(vnode, container);
      const target = button(container);
      assert(target);
      if (actualButton) assert.equal(target, actualButton, "real Vue patches the existing element");
      actualButton = target;
      const handler = target.props.onClick;
      assert.equal(typeof handler, "function");
      if (previous)
        assert.notEqual(handler, previous, "uncached real PROPS patch replaces the callback");
      previous = handler;
      const event = structuredClone(fixture.events[phase]);
      const returned = handler(event);
      assert.equal(returned, undefined);
      assert.deepEqual(event, fixture.expected[phase]);
    }
    renderer.render(null, container);
    assert.deepEqual(container.children, []);
    assert(actualButton);
    assert.equal(button(container), undefined);
  });
}

test("pinned directive grammar rejects returns and exposes the separately refused local-read rewrite", () => {
  assert.throws(() => compiler.compile('<button @click="return $event;"/>', pack.options));
  const measured = compiler.compile('<button @click="var x=$event; x.count++;"/>', pack.options);
  assert.match(measured.code, /var x=\$event; _ctx\.x\.count\+\+;/);
});
