import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import { pinnedVueRuntime } from "./n8n-default-slot-loop-runtime.mjs";

const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const rawInput = Buffer.concat(chunks);
const input = JSON.parse(rawInput.toString("utf8"));
const fixtureBase = new URL(
  "../../_fixtures/differential/compiler/slot-parameter-entities-8142/",
  import.meta.url,
);
const [controlsRaw, expectedRaw, nodeBytes] = await Promise.all([
  readFile(new URL("controls.json", fixtureBase)),
  readFile(new URL("runtime.expected.json", fixtureBase)),
  readFile(process.execPath),
]);
const controls = JSON.parse(controlsRaw);
const expected = JSON.parse(expectedRaw);
assert.equal(input.schema, "vize.slot-parameter-entities.actual-whole-compiler-packets");
assert.equal(input.version, 1);
assert.equal(input.observations.length, 2);
assert.equal(process.ppid, input.producer.pid, "actual Rust observer launches this runtime child");
const nativeBytes = await readFile(input.producer.executable);
assert.equal(hash(nativeBytes), input.producer.sha256, "actual current Rust observer bytes");
const logs = [];
const runtimeConsole = Object.fromEntries(
  ["warn", "error", "log", "info", "debug"].map((method) => [
    method,
    (...args) => logs.push({ method, args: args.map(String) }),
  ]),
);
const { Vue, code: runtimeCode, provenance } = await pinnedVueRuntime(runtimeConsole);
globalThis.__vizeSlotEntityVue = Vue;
const bridge = Object.keys(Vue)
  .filter((name) => /^[a-zA-Z_$][\w$]*$/.test(name))
  .map((name) => `export const ${name} = globalThis.__vizeSlotEntityVue.${name};`)
  .join("\n");
const bridgeUrl = `data:text/javascript;base64,${Buffer.from(bridge).toString("base64")}`;
const receipt = {
  schema: "vize.slot-parameter-entities.actual-pinned-runtime",
  version: 1,
  input: { sha256: hash(rawInput), packets: input },
  controls: { sha256: hash(controlsRaw), packets: controls },
  expectations: { sha256: hash(expectedRaw), packets: expected },
  node: {
    path: process.execPath,
    pid: process.pid,
    parentPid: process.ppid,
    version: process.version,
    sha256: hash(nodeBytes),
  },
  runtime: {
    version: Vue.version,
    url: provenance.vueRuntimeUrl,
    sha256: hash(runtimeCode),
    source: runtimeCode,
  },
  bridge: { source: bridge, sha256: hash(bridge) },
  observations: [],
};
const freeze = (node) => ({
  type: node.type,
  props: node.props,
  text: node.text,
  children: node.children.map(freeze),
});

async function observe(phase, id, compiler) {
  const assembled = `${compiler.codegen.preamble}\n${compiler.codegen.code}`;
  const record = {
    phase,
    id,
    lane: compiler.lane,
    prefixIdentifiers: compiler.prefixIdentifiers,
    compiler,
    originalAssembled: assembled,
    originalAssembledSha256: hash(assembled),
    hostCalls: [],
    logs: [],
    initialTree: null,
    updatedTree: null,
    error: null,
    cleanupError: null,
  };
  const logStart = logs.length;
  let app;
  try {
    let render;
    if (compiler.prefixIdentifiers) {
      const linked = assembled
        .replaceAll('from "vue"', `from ${JSON.stringify(bridgeUrl)}`)
        .replaceAll("from 'vue'", `from ${JSON.stringify(bridgeUrl)}`);
      assert.notEqual(linked, assembled, "actual module Vue import must be linked");
      record.evaluation = {
        source: linked,
        sha256: hash(linked),
        onlyChange: "Vue import specifier links the complete pinned runtime exports",
      };
      const label = `${phase}-${id}-${compiler.lane}`;
      const moduleUrl = `data:text/javascript;base64,${Buffer.from(linked).toString("base64")}#${label}`;
      render = (await import(moduleUrl)).render;
    } else {
      const body = `${assembled}\nreturn render;\n`;
      record.evaluation = {
        source: body,
        sha256: hash(body),
        onlyChange: "Evaluation wrapper returns the original declared render function",
      };
      // oxlint-disable-next-line typescript/no-implied-eval
      render = new Function("Vue", body)(Vue);
    }
    assert.equal(typeof render, "function");
    const make = (type, text = null) => ({ type, props: {}, text, children: [], parent: null });
    const host = {
      createElement(type) {
        record.hostCalls.push({ method: "createElement", type });
        return make(type);
      },
      createText(text) {
        record.hostCalls.push({ method: "createText", text });
        return make("#text", text);
      },
      createComment(text) {
        record.hostCalls.push({ method: "createComment", text });
        return make("#comment", text);
      },
      setText(node, text) {
        record.hostCalls.push({ method: "setText", text });
        node.text = text;
      },
      setElementText(node, text) {
        record.hostCalls.push({ method: "setElementText", text });
        for (const child of node.children) child.parent = null;
        node.text = text;
        node.children = [];
      },
      patchProp(node, key, previous, next) {
        record.hostCalls.push({ method: "patchProp", key, previous, next });
        node.props[key] = next;
      },
      insert(node, parent, anchor = null) {
        record.hostCalls.push({
          method: "insert",
          type: node.type,
          parentType: parent.type,
          anchorType: anchor?.type ?? null,
        });
        if (node.parent) node.parent.children.splice(node.parent.children.indexOf(node), 1);
        const index = anchor ? parent.children.indexOf(anchor) : parent.children.length;
        assert(index >= 0);
        parent.children.splice(index, 0, node);
        node.parent = parent;
      },
      remove(node) {
        record.hostCalls.push({ method: "remove", type: node.type });
        if (node.parent) {
          node.parent.children.splice(node.parent.children.indexOf(node), 1);
          node.parent = null;
        }
      },
      parentNode: (node) => node.parent,
      nextSibling: (node) => node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
    };
    const value = Vue.ref("entity-slot-value");
    const Panel = {
      name: "Panel",
      render() {
        return Vue.h("div", null, this.$slots.default({ item: value.value }));
      },
    };
    const container = make("root");
    app = Vue.createRenderer(host).createApp({ render });
    app.component("Panel", Panel);
    app.config.warnHandler = (message) =>
      logs.push({ method: "vue.warn", args: [String(message)] });
    app.config.errorHandler = (error) =>
      logs.push({ method: "vue.error", args: [String(error), error.stack] });
    app.mount(container);
    record.initialTree = freeze(container);
    value.value = "updated-slot-value";
    await Vue.nextTick();
    record.updatedTree = freeze(container);
  } catch (error) {
    record.error = Object.fromEntries(
      Object.getOwnPropertyNames(error).map((name) => [name, error[name]]),
    );
    record.error.name = error.name;
  } finally {
    try {
      app?.unmount();
    } catch (error) {
      record.cleanupError = Object.fromEntries(
        Object.getOwnPropertyNames(error).map((name) => [name, error[name]]),
      );
      record.cleanupError.name = error.name;
    }
  }
  record.logs = logs.slice(logStart);
  receipt.observations.push(record);
}

for (const [fixture, actual] of controls.cases.map((fixture, index) => [
  fixture,
  input.observations[index],
])) {
  assert.equal(actual.id, fixture.id);
  assert.equal(actual.source, fixture.source);
  assert.equal(fixture.before.length, 4);
  assert.equal(actual.compilers.length, 4);
  // These before modules are authenticated historical packets, not a fresh Rust rebuild.
  for (const compiler of fixture.before) await observe("historical-before", fixture.id, compiler);
  for (const compiler of actual.compilers) await observe("actual-current", fixture.id, compiler);
}
assert.equal(hash(await readFile(process.execPath)), receipt.node.sha256);
assert.equal(hash(await readFile(input.producer.executable)), input.producer.sha256);
await mkdir(dirname(input.runtimeReceipt), { recursive: true });
await writeFile(input.runtimeReceipt, JSON.stringify(receipt, null, 2) + "\n");
process.stdout.write(JSON.stringify(receipt));

// Raw complete observations above are retained before any expected semantic law.
assert.equal(receipt.observations.length, 16);
assert.equal(expected.cases.length, 8);
for (const observation of receipt.observations) {
  const laws = expected.cases.filter(
    (law) =>
      law.id === observation.id &&
      law.lane === observation.lane &&
      law.prefixIdentifiers === observation.prefixIdentifiers,
  );
  assert.equal(laws.length, 1, "each complete observation has one independently authored law");
  const key = observation.phase === "historical-before" ? "beforeExpected" : "currentExpected";
  const law = expected[laws[0][key]];
  assert.equal(observation.cleanupError, null, "actual cleanup must complete");
  if (law.error) {
    assert.deepEqual(
      { name: observation.error?.name, message: observation.error?.message },
      law.error,
    );
    assert.deepEqual(observation.hostCalls, law.hostCalls);
  } else {
    assert.equal(observation.error, null);
    assert.ok(observation.hostCalls.length > 0, "successful laws execute the real renderer");
  }
  assert.deepEqual(observation.logs, law.logs ?? []);
  assert.deepEqual(observation.initialTree, law.initialTree);
  assert.deepEqual(observation.updatedTree, law.updatedTree);
}
