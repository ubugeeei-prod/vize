import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import vm from "node:vm";
import { authenticatePackets } from "./n8n-default-slot-loop-custody.mjs";

const base = new URL(
  "../../_fixtures/differential/compiler/n8n-default-slot-loop/",
  import.meta.url,
);
const provenance = JSON.parse(await readFile(new URL("provenance.json", base), "utf8"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function pinned(url, sha) {
  const path = join(tmpdir(), `vize-default-slot-${sha}.js`);
  let bytes;
  try {
    bytes = await readFile(path);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    const response = await fetch(url);
    assert.equal(response.status, 200);
    bytes = Buffer.from(await response.arrayBuffer());
    assert.equal(hash(bytes), sha);
    await writeFile(path, bytes);
  }
  assert.equal(hash(bytes), sha);
  return bytes.toString("utf8");
}

export async function stock(source) {
  assert.equal(hash(source), provenance.authoredTemplateSha256);
  const code = await pinned(provenance.vueRuntimeUrl, provenance.vueRuntimeSha256);
  const scope = { console, setTimeout, clearTimeout };
  vm.runInNewContext(code, scope, { filename: provenance.vueRuntimeUrl });
  const Vue = scope.Vue;
  assert.equal(Vue.version, provenance.vueVersion);
  const compilerCode = await pinned(provenance.compilerUrl, provenance.compilerSha256);
  const compiler = await import(
    `data:text/javascript;base64,${Buffer.from(compilerCode).toString("base64")}`
  );
  assert.equal(compiler.version, provenance.vueVersion);
  const result = compiler.compileTemplate({
    source,
    filename: "DefaultSlot.vue",
    id: "default-slot",
    compilerOptions: provenance.officialOptions,
  });
  assert.deepEqual(result.errors, []);
  assert.deepEqual(result.tips, []);
  return {
    Vue,
    code: result.code,
    map: result.map,
    preamble: result.preamble,
    source: result.source,
    errors: result.errors,
    tips: result.tips,
  };
}

export async function trace(Vue, code, official = false) {
  const node = (kind, text = "", props = {}) => ({ kind, text, props, children: [], parent: null });
  const root = node("root");
  const host = {
    createElement: (tag) => node(tag),
    createText: (text) => node("text", text),
    createComment: (text) => node("comment", text),
    setText: (target, text) => {
      target.text = text;
    },
    setComment: (target, text) => {
      target.text = text;
    },
    setElementText(target, text) {
      for (const child of target.children) child.parent = null;
      target.children = [];
      if (text) {
        const child = node("text", text);
        child.parent = target;
        target.children.push(child);
      }
    },
    parentNode: (target) => target.parent,
    nextSibling: (target) =>
      target.parent?.children[target.parent.children.indexOf(target) + 1] ?? null,
    patchProp: (target, key, _old, value) => {
      target.props[key] = value;
    },
    insert(target, parent, anchor = null) {
      if (target.parent) host.remove(target);
      const index = anchor ? parent.children.indexOf(anchor) : parent.children.length;
      assert(index >= 0);
      parent.children.splice(index, 0, target);
      target.parent = parent;
    },
    remove(target) {
      if (target.parent) {
        const index = target.parent.children.indexOf(target);
        assert(index >= 0);
        target.parent.children.splice(index, 1);
        target.parent = null;
      }
    },
  };
  const renderer = Vue.createRenderer(host);
  const diagnostics = [];
  let calls = 0;
  const state = Vue.reactive({
    prefix: false,
    rows: [
      { id: "a", label: "A" },
      { id: "b", label: "B" },
    ],
    loadRows() {
      calls++;
      return state.rows;
    },
  });
  // Execute complete modules from the fixed fixture and checksum-pinned official compiler.
  // oxlint-disable-next-line typescript/no-implied-eval
  const render = new Function(
    "Vue",
    "state",
    official ? code : `with(state){${code}\nreturn render;}`,
  )(Vue, state);
  const Select = {
    setup(_props, { slots }) {
      return () =>
        Vue.h("section", null, [
          slots.prefix ? Vue.h("header", null, Vue.renderSlot(slots, "prefix")) : null,
          Vue.h("main", null, Vue.renderSlot(slots, "default")),
        ]);
    },
  };
  const Option = {
    props: ["rowId"],
    setup(props, { slots }) {
      return () => Vue.h("p", { "data-row": props.rowId }, Vue.renderSlot(slots, "default"));
    },
  };
  const app = renderer.createApp({ setup: () => state, render });
  app.component("Select", Select);
  app.component("Option", Option);
  app.config.warnHandler = (message) => diagnostics.push(String(message));
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const visible = (target) =>
    target.children.flatMap((child) => {
      if (child.kind === "comment" || (child.kind === "text" && !child.text)) return [];
      if (child.kind === "text") return [child.text];
      return [[child.kind, child.props, visible(child)]];
    });
  const rows = () => {
    const found = new Map();
    const visit = (target) => {
      if (target.props["data-row"]) found.set(target.props["data-row"], target);
      for (const child of target.children) visit(child);
    };
    visit(root);
    return found;
  };
  let previous = new Map();
  const phases = [];
  const capture = (delta) => {
    const current = rows();
    phases.push({
      tree: visible(root),
      calls: delta,
      identityPreserved: [...current].every(
        ([key, value]) => !previous.has(key) || previous.get(key) === value,
      ),
      removedAbsent: [...previous].every(([key, target]) => {
        if (current.has(key)) return true;
        for (let ancestor = target; ancestor; ancestor = ancestor.parent)
          if (ancestor === root) return false;
        return true;
      }),
      diagnostics: [...diagnostics],
    });
    previous = current;
  };
  app.mount(root);
  await Vue.nextTick();
  capture(calls);
  for (const patch of [
    {
      rows: [
        { id: "b", label: "B2" },
        { id: "a", label: "A2" },
        { id: "c", label: "C" },
      ],
    },
    { prefix: true },
    {
      rows: [
        { id: "a", label: "A2" },
        { id: "c", label: "C" },
      ],
    },
    { prefix: false },
  ]) {
    const before = calls;
    Object.assign(state, patch);
    await Vue.nextTick();
    capture(calls - before);
  }
  const before = calls;
  app.unmount();
  await Vue.nextTick();
  capture(calls - before);
  return phases;
}

function templateModules(packet) {
  assert.equal(hash(packet.originalSource), provenance.authoredTemplateSha256);
  const rows = packet.rows.filter(
    (row) => row.kind === "template" && row.prefixIdentifiers === false,
  );
  assert.equal(rows.length, 1, "one complete function-mode capture per source phase");
  const row = rows[0];
  assert.deepEqual(row.legacy.errors, []);
  assert.equal(typeof row.legacy.assembled, "string");
  assert.equal(typeof row.native.assembled, "string");
  assert.equal(row.native.error, null);
  assert.equal(row.native.production.status, "returned");
  assert.equal(row.native.production.assembled, row.native.assembled);
  assert.deepEqual(row.native.loweredDiagnostics, []);
  assert.deepEqual(row.native.effectiveDiagnostics, []);
  assert.equal(row.native.diagnosedEmission, null);
  return { legacy: row.legacy.assembled, current: row.native.assembled };
}

export async function capturePackets(beforeRaw, currentRaw, custody) {
  authenticatePackets(custody, beforeRaw, currentRaw, provenance.originalFixtureHead);
  const before = JSON.parse(beforeRaw);
  const current = JSON.parse(currentRaw);
  assert.equal(before.originalSource, current.originalSource);
  const baseline = templateModules(before);
  const modules = templateModules(current);
  const official = await stock(current.originalSource);
  const allModules = {
    beforeLegacy: baseline.legacy,
    beforeNative: baseline.current,
    ...modules,
    official: official.code,
  };
  const traces = {};
  for (const [name, code] of Object.entries(allModules)) {
    try {
      traces[name] = await trace(official.Vue, code, name === "official");
    } catch (error) {
      traces[name] = { error: String(error), stack: error.stack };
    }
  }
  return {
    provenance,
    custody,
    packets: { before, current },
    rawPackets: { before: beforeRaw, current: currentRaw },
    packetHashes: { before: hash(beforeRaw), current: hash(currentRaw) },
    source: current.originalSource,
    modules: allModules,
    official: {
      map: official.map,
      preamble: official.preamble,
      source: official.source,
      errors: official.errors,
      tips: official.tips,
    },
    moduleHashes: Object.fromEntries(
      Object.entries(allModules).map(([name, code]) => [name, hash(code)]),
    ),
    traces,
  };
}

export async function qualifyPackets(receipt) {
  const expected = JSON.parse(await readFile(new URL("expected.json", base), "utf8"));
  for (const compiler of ["official", "legacy", "current"])
    assert.deepEqual(receipt.traces[compiler], expected, `${compiler}: all six authored phases`);
  assert.equal(receipt.traces.beforeLegacy.length, expected.length);
  assert.deepEqual(
    receipt.traces.beforeLegacy.map((phase) => phase.calls),
    [2, 2, 2, 2, 2, 0],
    "immutable before-source legacy evaluates twice in each mounted phase",
  );
  for (const [index, phase] of receipt.traces.beforeLegacy.entries()) {
    assert.deepEqual(
      { ...phase, calls: expected[index].calls },
      expected[index],
      `before-source legacy: exact tree, identity and teardown in phase ${index}`,
    );
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv[2] === "--captures") {
    assert.equal(process.argv.length, 7, "--captures <before> <current> <custody> <receipt>");
    const [before, current, custodyRaw] = await Promise.all(
      process.argv.slice(3, 6).map((path) => readFile(path, "utf8")),
    );
    const receipt = await capturePackets(before, current, JSON.parse(custodyRaw));
    await mkdir(dirname(process.argv[6]), { recursive: true });
    await writeFile(process.argv[6], JSON.stringify(receipt, null, 2) + "\n");
    await qualifyPackets(receipt);
    process.stdout.write(JSON.stringify(receipt));
  } else {
    const chunks = [];
    for await (const chunk of process.stdin) chunks.push(chunk);
    const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    assert.equal(hash(input.source), provenance.authoredTemplateSha256);
    const official = await stock(input.source);
    const receipt = {
      provenance,
      source: input.source,
      modules: { ...input.modules, official: official.code },
      official: {
        map: official.map,
        preamble: official.preamble,
        source: official.source,
        errors: official.errors,
        tips: official.tips,
      },
      moduleHashes: Object.fromEntries(
        Object.entries({ ...input.modules, official: official.code }).map(([name, code]) => [
          name,
          hash(code),
        ]),
      ),
      traces: {
        legacy: await trace(official.Vue, input.modules.legacy),
        current: await trace(official.Vue, input.modules.current),
        official: await trace(official.Vue, official.code, true),
      },
    };
    await mkdir(dirname(input.receipt), { recursive: true });
    await writeFile(input.receipt, JSON.stringify(receipt, null, 2) + "\n");
    process.stdout.write(JSON.stringify(receipt));
  }
}
