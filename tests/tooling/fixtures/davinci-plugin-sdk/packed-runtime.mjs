// Copied into a clean consumer directory; imports resolve only the installed tarball.
import assert from "node:assert/strict";
import * as sdk from "@vizejs/plugin-sdk";
import { applyFixes } from "@vizejs/plugin-sdk/fixes";
import { createSandboxRunner, SANDBOX_IMAGE } from "@vizejs/plugin-sdk/sandbox";
import {
  VISITOR_API_VERSION,
  createVirtualTsVisitorHost,
  defineVisitorPlugin,
  negotiateVisitorPlugin,
  runVisitorFixture,
} from "@vizejs/plugin-sdk/visitor";

const expected = [
  "BATCH_SCHEMA",
  "applyFixes",
  "defineFactProvider",
  "defineOutputPlugin",
  "definePlugin",
  "defineTransformPlugin",
  "runBatch",
  "runRules",
];
assert.deepEqual(Object.keys(sdk).sort(), expected.sort());
assert.equal(sdk.BATCH_SCHEMA, 1);
assert.equal(applyFixes, sdk.applyFixes);
assert.equal(applyFixes("🦀blue", [{ start: 4, end: 8, text: "red" }]), "🦀red");

const batch = {
  schema: 1,
  plugin: "packed",
  file: "Button.vue",
  parents: [-1],
  nodes: [{ id: 0, kind: "ui.element", name: "button" }],
  facts: { "packed/colors": [[0, "red"]] },
};
const plugin = sdk.definePlugin({
  name: "packed",
  version: "1",
  visit: ["ui.element"],
  demands: ["packed/colors"],
  cacheInputs: [],
  rules: {
    contrast(ctx) {
      const colors = ctx.facts("packed/colors");
      for (const node of ctx.nodes) ctx.report(node, colors.get(node.id));
    },
  },
});
const reports = [{ rule: "contrast", node: 0, message: "red" }];
assert.deepEqual(sdk.runBatch(plugin, batch), reports);
assert.deepEqual(JSON.parse(plugin.run(JSON.stringify(batch))), reports);
assert.deepEqual(
  sdk.runRules(plugin, {
    nodes: batch.nodes,
    byId: new Map([[0, batch.nodes[0]]]),
    facts: (name) => new Map(batch.facts[name]),
    parent: (id) => batch.parents[id],
  }),
  reports,
);

const provider = sdk.defineFactProvider({
  name: "packed",
  version: "1",
  provides: ["packed/colors"],
  cacheInputs: [],
  provide(input) {
    return { "packed/colors": input.nodes.map((node) => [node.id, "red"]) };
  },
});
assert.deepEqual(JSON.parse(provider.run(JSON.stringify(batch))), {
  "packed/colors": [[0, "red"]],
});

const transform = sdk.defineTransformPlugin({
  name: "packed-transform",
  version: "1",
  cacheInputs: [],
  transform(input) {
    return [
      { kind: "replace-static-attribute", node: input.nodes[0].id, name: "disabled", value: null },
    ];
  },
});
assert.deepEqual(
  JSON.parse(
    transform.run(JSON.stringify({ ...batch, stage: "s2-precanonical-static-attributes" })),
  ),
  {
    schema: 1,
    edits: [{ kind: "replace-static-attribute", node: 0, name: "disabled", value: null }],
  },
);

for (const family of ["formatter", "output"]) {
  const output = sdk.defineOutputPlugin({
    name: `packed-${family}`,
    version: "1",
    cacheInputs: [],
    family,
    output() {
      return family === "formatter"
        ? [{ start: 0, end: 0, text: "\n" }]
        : [{ placement: "append", comment: "packed" }];
    },
  });
  assert.deepEqual(
    JSON.parse(
      output.run(
        JSON.stringify({
          schema: 1,
          plugin: output.name,
          family,
          offsetEncoding: "utf8",
          compiled: { code: "const x=1", preamble: "", ast: {}, helpers: [] },
        }),
      ),
    ),
    family === "formatter"
      ? [{ start: 0, end: 0, text: "\n" }]
      : [{ placement: "append", comment: "packed" }],
  );
}

// Construction loads the shipped worker and SDK files without invoking Docker.
const sandbox = createSandboxRunner({
  name: "packed-sandbox",
  version: "1",
  cacheInputs: [],
  rules: { button: '(ctx) => { for (const node of ctx.nodes) ctx.report(node, "packed"); }' },
});
assert.equal(typeof sandbox.run, "function");
assert.match(sandbox.fingerprint, /^[a-f0-9]{64}$/);
assert.match(SANDBOX_IMAGE, /^node:.*@sha256:[a-f0-9]{64}$/);
const visitorEvents = [];
const visitor = defineVisitorPlugin({
  apiVersion: VISITOR_API_VERSION,
  name: "packed-visitor",
  version: "1",
  capabilities: ["virtual-ts"],
  cacheInputs: [],
  visitors: {
    "virtual-ts"(event) {
      visitorEvents.push(event.payload.virtualFile);
    },
  },
});
assert.deepEqual(negotiateVisitorPlugin(visitor, visitor), {
  apiVersion: VISITOR_API_VERSION,
  capabilities: ["virtual-ts"],
});
runVisitorFixture(visitor, "virtual-ts", {
  apiVersion: VISITOR_API_VERSION,
  capability: "virtual-ts",
  file: "Button.vue",
  payload: { sourceFile: "Button.vue", virtualFile: "Button.vue.ts", code: "", mappings: [] },
});
assert.deepEqual(visitorEvents, ["Button.vue.ts"]);
const virtualTsHost = createVirtualTsVisitorHost(visitor);
assert.equal(typeof virtualTsHost.run, "function");
assert.equal(
  virtualTsHost.run(
    JSON.stringify({
      apiVersion: VISITOR_API_VERSION,
      capability: "virtual-ts",
      file: "Button.vue",
      payload: { sourceFile: "Button.vue", virtualFile: "Button.vue.ts", code: "", mappings: [] },
    }),
  ),
  "",
);
assert.ok(import.meta.resolve("@vizejs/plugin-sdk").includes("/node_modules/@vizejs/plugin-sdk/"));
console.log("packed SDK runtime exports: 8 root, fixes, sandbox and visitor verified");
