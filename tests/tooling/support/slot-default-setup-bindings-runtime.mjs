import assert from "node:assert/strict";
import fs from "node:fs";
import crypto from "node:crypto";
import { createRequire } from "node:module";
import { pinnedVueRuntime } from "./n8n-default-slot-loop-runtime.mjs";
import { typedSlotHost } from "./slot-default-setup-bindings-runtime-host.mjs";

const hash = (body) => crypto.createHash("sha256").update(body).digest("hex");
const rawInput = fs.readFileSync(0);
const input = JSON.parse(rawInput);
const base = new URL(
  "../../_fixtures/differential/compiler/slot-default-setup-bindings-8142/",
  import.meta.url,
);
const casesRaw = fs.readFileSync(new URL("original-runtime-cases.json", base));
const lawsRaw = fs.readFileSync(new URL("original-runtime-desired-laws.json", base));
const cases = JSON.parse(casesRaw);
const laws = JSON.parse(lawsRaw);
assert.equal(input.schema, "vize.slot-default-setup-bindings.actual-whole-sfc-packets");
assert.equal(input.version, 1);
assert.equal(input.observations.length, 6);
assert.deepEqual(input.authoredCases, cases);
assert.equal(input.authoredCasesSha256, hash(casesRaw));
assert.equal(laws.sourceCasesSha256, hash(casesRaw));
assert.equal(laws.lawCount, 12);
assert.equal(process.ppid, input.producer.pid, "actual Rust observer owns this child");
assert.equal(hash(fs.readFileSync(input.producer.executable)), input.producer.sha256);
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const entryPath = fs.realpathSync(fromUi.resolve("typescript"));
const packagePath = fs.realpathSync(fromUi.resolve("typescript/package.json"));
const entryBytes = fs.readFileSync(entryPath);
const packageBytes = fs.readFileSync(packagePath);
assert.equal(hash(entryBytes), "569177652966bd528c319171c7dd22860dbf72bde116cbc4f644f1d02bb12e39");
assert.equal(
  hash(packageBytes),
  "9332e97c30d3e53ed54910b89207ed657fb444066484df6e5b6965bf130865e9",
);
const ts = fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const tsEntry = {
  version: ts.version,
  entry: { path: entryPath, sha256: hash(entryBytes) },
  packageJson: {
    path: packagePath,
    sha256: hash(packageBytes),
    contents: JSON.parse(packageBytes),
  },
};
const compilerOptions = {
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  removeComments: false,
  sourceMap: true,
  inlineSources: true,
};
const logs = [];
const runtimeConsole = Object.fromEntries(
  ["warn", "error", "log", "info", "debug"].map((method) => [
    method,
    (...args) => logs.push({ method, args: args.map(String) }),
  ]),
);
const { Vue, code: rawVue, provenance } = await pinnedVueRuntime(runtimeConsole);
assert.equal(Vue.version, "3.5.26");
globalThis.__typedPinnedVue = Vue;
const bridge =
  Object.keys(Vue)
    .filter((name) => /^[a-zA-Z_$][\w$]*$/.test(name))
    .map((name) => `export const ${name} = globalThis.__typedPinnedVue.${name};`)
    .join("\n") + "\n";
const bridgePath = `data:text/javascript;base64,${Buffer.from(bridge).toString("base64")}`;
const errorPacket = (error) => ({
  ...Object.fromEntries(Object.getOwnPropertyNames(error).map((k) => [k, error[k]])),
  name: error.name,
});
const diagnosticPacket = (d) => ({
  category: d.category,
  code: d.code,
  start: d.start ?? null,
  length: d.length ?? null,
  messageText: d.messageText,
  source: d.source ?? null,
  reportsUnnecessary: d.reportsUnnecessary ?? null,
  reportsDeprecated: d.reportsDeprecated ?? null,
  file: d.file
    ? {
        fileName: d.file.fileName,
        text: d.file.text,
        languageVersion: d.file.languageVersion,
        languageVariant: d.file.languageVariant,
        scriptKind: d.file.scriptKind,
      }
    : null,
  relatedInformation: d.relatedInformation?.map(diagnosticPacket) ?? null,
});
const snapshot = (value) =>
  value === undefined ? { $undefined: true } : JSON.parse(JSON.stringify(value));
const capture = {
  schema: "vize.slot-default-setup-bindings.actual-pinned-runtime",
  version: 1,
  input: { sha256: hash(rawInput), packets: input },
  authoredCases: { sha256: hash(casesRaw), packets: cases },
  desiredLaws: { sha256: hash(lawsRaw), packets: laws },
  node: {
    path: process.execPath,
    version: process.version,
    pid: process.pid,
    parentPid: process.ppid,
    sha256: hash(fs.readFileSync(process.execPath)),
  },
  runtime: {
    version: Vue.version,
    url: provenance.vueRuntimeUrl,
    sha256: hash(rawVue),
    source: rawVue,
  },
  typescript: { ...tsEntry, compilerOptions },
  bridge: { source: bridge, sha256: hash(bridge) },
  observations: [],
};
for (const row of input.observations) {
  const authored = cases.cases.find((x) => x.id === row.id);
  assert.equal(row.source, authored.source);
  for (const compiler of row.compilers) {
    const record = {
      id: row.id,
      lane: compiler.lane,
      originalCompleteCompileResult: compiler.completeCompileResult,
      logs: [],
      frameworkErrors: [],
      hostCalls: [],
      phases: [],
      records: [],
      handlerIdentities: null,
      initialTree: null,
      updatedTree: null,
      initialSemanticTree: null,
      updatedSemanticTree: null,
      unmountedTree: null,
      error: null,
    };
    const logStart = logs.length;
    let app;
    try {
      const original = compiler.completeCompileResult.Ok;
      assert.ok(original, "actual SFC compile must return Ok");
      assert.deepEqual(original.errors, []);
      assert.deepEqual(original.warnings, []);
      assert.equal(original.css, null);
      assert.deepEqual(original.macroArtifacts, []);
      const transpiled = ts.transpileModule(original.code, {
        fileName: row.filename + ".ts",
        compilerOptions,
        reportDiagnostics: true,
      });
      record.typescriptTransform = {
        fileName: row.filename + ".ts",
        compilerOptions,
        originalSource: original.code,
        originalSha256: hash(original.code),
        outputText: transpiled.outputText,
        sourceMapText: transpiled.sourceMapText ?? null,
        diagnostics: transpiled.diagnostics?.map(diagnosticPacket) ?? [],
        outputSha256: hash(transpiled.outputText),
      };
      assert.deepEqual(record.typescriptTransform.diagnostics, []);
      const linked = transpiled.outputText
        .replaceAll('from "vue"', `from ${JSON.stringify(bridgePath)}`)
        .replaceAll("from 'vue'", `from ${JSON.stringify(bridgePath)}`);
      assert.notEqual(linked, transpiled.outputText, "actual Vue module import must be linked");
      const modulePath = `data:text/javascript;base64,${Buffer.from(linked).toString("base64")}`;
      record.executedModule = {
        path: modulePath,
        source: linked,
        sha256: hash(linked),
        onlyChange:
          "Pinned actual TS consumer output links only Vue import specifier to complete runtime exports",
      };
      const component = (await import(modulePath)).default;
      assert.equal(typeof component.setup, "function");
      globalThis.__typedSlotRecords = [];
      const { host, make, freeze, find, functionId, hostValue } = typedSlotHost(record, snapshot);
      const payload = Vue.ref(snapshot(authored.initialSlotPayload));
      const Panel = {
        name: "Panel",
        render() {
          return Vue.h("div", null, this.$slots.default(payload.value));
        },
      };
      const container = make("root");
      const click = (label, fn, event) => {
        const begin = globalThis.__typedSlotRecords.length;
        const returned = fn(event);
        record.phases.push({
          label,
          event: snapshot(event),
          handlerId: functionId(fn),
          returned: hostValue(returned),
          records: globalThis.__typedSlotRecords.slice(begin).map(snapshot),
        });
      };
      app = Vue.createRenderer(host).createApp(component);
      app.config.errorHandler = (error, instance, info) =>
        record.frameworkErrors.push({ error: errorPacket(error), info: String(info) });
      app.component("Panel", Panel);
      app.mount(container);
      const initialInside = find(container, "slot").props.onClick;
      const initialOutside = find(container, "outside").props.onClick;
      record.initialTree = freeze(container);
      record.initialSemanticTree = freeze(container, true);
      click("initial-inside", initialInside, { type: "click", phase: "initial" });
      click("initial-outside", initialOutside, { type: "click", phase: "initial" });
      payload.value = snapshot(authored.updatedSlotPayload);
      await Vue.nextTick();
      const updatedInside = find(container, "slot").props.onClick;
      const updatedOutside = find(container, "outside").props.onClick;
      record.updatedTree = freeze(container);
      record.updatedSemanticTree = freeze(container, true);
      click("updated-inside", updatedInside, { type: "click", phase: "updated" });
      click("updated-outside", updatedOutside, { type: "click", phase: "updated" });
      click("saved-initial-inside-after-update", initialInside, {
        type: "click",
        phase: "saved-initial",
      });
      record.handlerIdentities = {
        insideStable: initialInside === updatedInside,
        outsideStable: initialOutside === updatedOutside,
        initialInsideId: functionId(initialInside),
        updatedInsideId: functionId(updatedInside),
        initialOutsideId: functionId(initialOutside),
        updatedOutsideId: functionId(updatedOutside),
      };
      record.records = globalThis.__typedSlotRecords.map(snapshot);
      app.unmount();
      app = null;
      await Vue.nextTick();
      record.unmountedTree = freeze(container);
    } catch (error) {
      record.error = errorPacket(error);
      if (app) app.unmount();
    }
    record.logs = logs.slice(logStart);
    capture.observations.push(record);
  }
}

// Retain every whole compiler result, transform, runtime error, tree, host call,
// callback identity and cleanup result before judging any desired product law.
assert.equal(hash(fs.readFileSync(process.execPath)), capture.node.sha256);
assert.equal(hash(fs.readFileSync(input.producer.executable)), input.producer.sha256);
assert.equal(hash(fs.readFileSync(entryPath)), tsEntry.entry.sha256);
assert.equal(hash(fs.readFileSync(packagePath)), tsEntry.packageJson.sha256);
fs.writeFileSync(input.runtimeReceipt, JSON.stringify(capture, null, 2) + "\n");
console.log(JSON.stringify(capture, null, 2));
assert.equal(capture.observations.length, 12);
assert.deepEqual(
  capture.observations.map(({ id, lane }) => ({ id, lane })),
  laws.laws.map(({ id, lane }) => ({ id, lane })),
);
for (const [index, actual] of capture.observations.entries()) {
  const completeDesiredLaw = {
    error: actual.error,
    logs: actual.logs,
    frameworkErrors: actual.frameworkErrors,
    initialSemanticTree: actual.initialSemanticTree,
    updatedSemanticTree: actual.updatedSemanticTree,
    unmountedTree: actual.unmountedTree,
    records: actual.records,
    phases: actual.phases.map(
      ({ handlerId: _handlerId, ...wholeAuthoredPhase }) => wholeAuthoredPhase,
    ),
    handlerIdentityLaws: actual.handlerIdentities
      ? {
          insideStable: actual.handlerIdentities.insideStable,
          outsideStable: actual.handlerIdentities.outsideStable,
        }
      : null,
  };
  assert.deepEqual(
    completeDesiredLaw,
    laws.laws[index].completeDesiredLaw,
    `${actual.id}/${actual.lane}: whole original typed callback/runtime laws`,
  );
}
