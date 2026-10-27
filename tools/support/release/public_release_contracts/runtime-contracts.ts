import assert from "node:assert/strict";
import { compileFunction } from "node:vm";
import { createRequire } from "node:module";
import { join, resolve, dirname } from "node:path";
import { writeFileSync, readFileSync } from "node:fs";
interface HostNode {
  type: string;
  text: string;
  children: HostNode[];
  parent: HostNode | null;
  props: Record<string, unknown>;
}
interface HostOperations {
  patchProp(el: HostNode, key: string, old: unknown, value: unknown): void;
  insert(el: HostNode, parent: HostNode, anchor?: HostNode | null): void;
  remove(el: HostNode): void;
  createElement(type: string): HostNode;
  createText(text: string): HostNode;
  createComment(text: string): HostNode;
  setText(el: HostNode, text: string): void;
  setElementText(el: HostNode, text: string): void;
  parentNode(el: HostNode): HostNode | null;
  nextSibling(el: HostNode): HostNode | null;
}
interface VueRuntime {
  version: string;
  createRenderer(ops: HostOperations): {
    createApp(component: object): { mount(root: HostNode): unknown; unmount(): void };
  };
  h(component: unknown, props?: Record<string, unknown>, children?: unknown[]): unknown;
  nextTick(): Promise<void>;
}
interface Compiler {
  compile(source: string, options: object): { errors?: unknown[]; preamble: string; code: string };
  compileSfc(
    source: string,
    options: object,
  ): { errors?: unknown[]; code?: string; script?: { code: string } };
}
interface WasmCompiler extends Compiler {
  init(bytes: Uint8Array): Promise<unknown>;
}
interface PublicInstance {
  msg?: unknown;
  $?: { exposed?: unknown };
}
interface SlottedInstance {
  label: unknown;
  $emit(name: string): void;
  $slots: { default?: () => unknown[] };
}
type Row = { id: string; observed: Record<string, unknown>; passed: boolean };

const [workspace, out, version, backend, mode = "assert"] = process.argv.slice(2);
assert(workspace && out && version);
assert.match(version, /^0\.\d+\.0$/);
assert(backend === "native" || backend === "wasm");
assert(mode === "assert" || mode === "capture");
const require = createRequire(join(resolve(workspace), "package.json"));
const Vue = require("vue") as VueRuntime;
assert.equal(Vue.version, "3.6.0-rc.13");
const packagePath = join(dirname(require.resolve("@vizejs/" + backend)), "package.json");
const packageMetadata = JSON.parse(readFileSync(packagePath, "utf8")) as {
  name?: unknown;
  version?: unknown;
};
assert.equal(packageMetadata.name, "@vizejs/" + backend);
assert.equal(packageMetadata.version, version);
let native: Compiler;
if (backend === "native") native = require("@vizejs/native") as Compiler;
else {
  const wasmEntry = require.resolve("@vizejs/wasm");
  const wasm = (await import(wasmEntry)) as WasmCompiler;
  await wasm.init(readFileSync(join(dirname(wasmEntry), "vize_vitrine_bg.wasm")));
  native = wasm;
}
function render(template: string): (...args: unknown[]) => unknown {
  const x = native.compile(template, { mode: "function", prefixIdentifiers: true });
  assert.deepEqual(x.errors ?? [], []);
  return compileFunction(x.preamble + "\n" + x.code + "\nreturn render;", ["Vue"])(Vue);
}
function component(source: string): object {
  const x = native.compileSfc(source, { filename: "Private.vue", inlineTemplate: false });
  assert.deepEqual(x.errors ?? [], []);
  const code = (backend === "native" ? x.code! : x.script!.code)
    .replace(
      /import\s*\{([^}]+)\}\s*from\s*['"]vue['"];?/g,
      (_match: string, names: string) => `const {${names.replace(/\bas\b/g, ":")}} = Vue;`,
    )
    .replace("export default", "return");
  return compileFunction(code, ["Vue"])(Vue);
}
const node = (type: string, text = ""): HostNode => ({
  type,
  text,
  children: [],
  parent: null,
  props: {},
});
const renderer = Vue.createRenderer({
  patchProp(el, k, old, v) {
    el.props[k] = v;
  },
  insert(el, parent, anchor = null) {
    if (el.parent) {
      const old = el.parent.children;
      old.splice(old.indexOf(el), 1);
    }
    el.parent = parent;
    const i = anchor ? parent.children.indexOf(anchor) : -1;
    parent.children.splice(i < 0 ? parent.children.length : i, 0, el);
  },
  remove(el) {
    if (el.parent) {
      el.parent.children.splice(el.parent.children.indexOf(el), 1);
      el.parent = null;
    }
  },
  createElement: (t) => node(t),
  createText: (t) => node("#text", t),
  createComment: (t) => node("#comment", t),
  setText(el, t) {
    el.text = t;
  },
  setElementText(el, t) {
    el.children = [];
    el.text = t;
  },
  parentNode: (el) => el.parent,
  nextSibling: (el) => el.parent?.children[el.parent.children.indexOf(el) + 1] ?? null,
});
function find(root: HostNode, type: string): HostNode | undefined {
  if (root.type === type) return root;
  for (const x of root.children) {
    const found = find(x, type);
    if (found) return found;
  }
}
function text(root: HostNode): string {
  return root.text + root.children.map(text).join("");
}
const rows: Row[] = [];
{
  const root = node("root"),
    app = renderer.createApp({
      data: () => ({ n: 0 }),
      methods: {
        inc(this: { n: number }) {
          this.n++;
        },
      },
      render: render(
        '<button v-once :title="String(n)" @click="inc">{{n}}</button><output>{{n}}</output>',
      ),
    });
  app.mount(root);
  const button = find(root, "button")!;
  for (let i = 0; i < 2; i++) {
    (button.props.onClick as (() => void) | null | undefined)?.();
    await Vue.nextTick();
  }
  const observed = {
    handler: typeof button.props.onClick,
    button: text(button),
    title: button.props.title,
    output: text(find(root, "output")!),
  };
  rows.push({
    id: "10875-native-once-click",
    observed,
    passed:
      observed.handler === "function" &&
      observed.button === "0" &&
      observed.title === "0" &&
      observed.output === "2",
  });
  app.unmount();
}
{
  const root = node("root");
  const Child = {
    props: ["label"],
    emits: ["ping"],
    render(this: SlottedInstance) {
      return Vue.h("button", { onClick: () => this.$emit("ping") }, [
        String(this.label) + ":",
        ...(this.$slots.default?.() ?? []),
      ]);
    },
  };
  const app = renderer.createApp({
    components: { Child },
    data: () => ({ n: 0 }),
    methods: {
      inc(this: { n: number }) {
        this.n++;
      },
    },
    render: render(
      '<Child v-once :label="n" @ping="inc"><span>{{n}}</span></Child><output>{{n}}</output>',
    ),
  });
  app.mount(root);
  const button = find(root, "button")!;
  for (let i = 0; i < 2; i++) {
    (button.props.onClick as () => void)();
    await Vue.nextTick();
  }
  const observed = { button: text(button), output: text(find(root, "output")!) };
  rows.push({
    id: "10815-component-once-props-events-slots",
    observed,
    passed: observed.button === "0:2" && observed.output === "2",
  });
  app.unmount();
}
for (const exposed of [false, true]) {
  const source = `<script setup>const msg = 1;${exposed ? "defineExpose({msg});" : ""}</script><template><span>{{msg}}</span></template>`;
  const root = node("root"),
    Child = component(source);
  let ref = undefined as PublicInstance | undefined;
  const app = renderer.createApp({
    render() {
      return Vue.h(Child, {
        ref: (x: PublicInstance) => {
          ref = x;
        },
      });
    },
  });
  app.mount(root);
  const observed = {
    text: text(find(root, "span")!),
    publicMsg: ref?.msg ?? null,
    exposed: ref?.$?.exposed ?? null,
  };
  rows.push({
    id: exposed ? "10891-explicit-expose" : "10891-private-default",
    observed,
    passed:
      observed.text === "1" &&
      (exposed
        ? observed.publicMsg === 1
        : observed.publicMsg === null && observed.exposed !== null),
  });
  app.unmount();
}
const result =
  backend === "native"
    ? { vue: Vue.version, native: version, rows }
    : { vue: Vue.version, backend: "wasm", packageVersion: version, rows };
writeFileSync(out, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result));
if (mode === "assert") assert.ok(rows.every((x) => x.passed));
