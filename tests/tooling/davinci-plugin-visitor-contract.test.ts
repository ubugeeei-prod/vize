import assert from "node:assert/strict";
import { test } from "node:test";
import {
  VISITOR_API_VERSION,
  VISITOR_CAPABILITIES,
  VisitorPluginValidationError,
  defineVisitorPlugin,
  negotiateVisitorPlugin,
  runVisitorFixture,
} from "../../npm/plugin-sdk/visitor.js";

function expectFailure(callback: () => unknown, code: string, message: RegExp) {
  assert.throws(callback, (error: unknown) => {
    assert(error instanceof VisitorPluginValidationError);
    assert.equal(error.code, code);
    assert.match(error.message, message);
    return true;
  });
}

test("visitor descriptors preserve typed capabilities without registering a runtime", () => {
  const seen: string[] = [];
  const plugin = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "typed-observer",
    version: "1.0.0",
    capabilities: ["devtools-trace", "sfc", "virtual-ts"] as const,
    cacheInputs: [
      { name: "project", value: "dashboard" },
      { name: "dialect", value: "vue3" },
    ],
    visitors: {
      sfc(event) {
        assert.throws(() => {
          (event.payload.blocks as unknown[]).push({});
        }, TypeError);
        seen.push(`${event.capability}:${event.payload.blocks[0]?.kind}`);
      },
      "virtual-ts"(event) {
        seen.push(`${event.capability}:${event.payload.virtualFile}`);
      },
      "devtools-trace"(event) {
        seen.push(`${event.capability}:${event.payload.events[0]?.stage}`);
      },
    },
  });

  assert.deepEqual(plugin.capabilities, ["devtools-trace", "sfc", "virtual-ts"]);
  assert.deepEqual(plugin.cacheInputs, [
    { name: "dialect", value: "vue3" },
    { name: "project", value: "dashboard" },
  ]);
  assert.equal(Object.isFrozen(plugin), true);
  assert.equal(Object.isFrozen(plugin.visitors), true);
  assert.match(plugin.fingerprint, /^[a-f0-9]{64}$/);
  assert.deepEqual(
    negotiateVisitorPlugin(plugin, {
      apiVersion: VISITOR_API_VERSION,
      capabilities: VISITOR_CAPABILITIES,
    }),
    { apiVersion: VISITOR_API_VERSION, capabilities: plugin.capabilities },
  );

  runVisitorFixture(plugin, "sfc", {
    apiVersion: VISITOR_API_VERSION,
    capability: "sfc",
    file: "Button.vue",
    payload: {
      source: "<template><button /></template>",
      blocks: [
        {
          kind: "template",
          language: "html",
          range: { start: 0, end: 31, encoding: "utf8" },
          attributes: {},
        },
      ],
    },
  });
  assert.deepEqual(seen, ["sfc:template"]);
});

test("capability negotiation fails closed with stable codes and fields", () => {
  const plugin = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "virtual-observer",
    version: "1",
    capabilities: ["virtual-ts", "devtools-trace"] as const,
    cacheInputs: [],
    visitors: { "virtual-ts"() {}, "devtools-trace"() {} },
  });

  expectFailure(
    () =>
      negotiateVisitorPlugin(plugin, {
        apiVersion: VISITOR_API_VERSION,
        capabilities: ["sfc", "virtual-ts"],
      }),
    "unsupported-capability",
    /host\.capabilities: missing "devtools-trace"$/,
  );
  expectFailure(
    () =>
      negotiateVisitorPlugin(plugin, { apiVersion: 2, capabilities: VISITOR_CAPABILITIES } as any),
    "unsupported-api-version",
    /host\.apiVersion: 2 is unsupported; expected 1$/,
  );
  expectFailure(
    () =>
      negotiateVisitorPlugin({ ...plugin } as any, {
        apiVersion: 1,
        capabilities: VISITOR_CAPABILITIES,
      }),
    "invalid-plugin",
    /plugin: must be created by defineVisitorPlugin$/,
  );
});

test("invalid visitor manifests produce deterministic actionable failures", () => {
  const base = {
    apiVersion: VISITOR_API_VERSION,
    name: "observer",
    version: "1",
    capabilities: ["sfc"] as const,
    cacheInputs: [] as const,
    visitors: { sfc() {} },
  };
  const cases: Array<[string, RegExp, () => unknown]> = [
    [
      "invalid-name",
      /name: must be a non-empty string$/,
      () => defineVisitorPlugin({ ...base, name: "" }),
    ],
    [
      "unsupported-api-version",
      /apiVersion: 2 is unsupported; expected 1$/,
      () => defineVisitorPlugin({ ...base, apiVersion: 2 } as any),
    ],
    [
      "duplicate-capability",
      /capabilities: "sfc" is declared more than once$/,
      () => defineVisitorPlugin({ ...base, capabilities: ["sfc", "sfc"] } as any),
    ],
    [
      "unknown-capability",
      /capabilities\[0\]: "future" is not supported$/,
      () => defineVisitorPlugin({ ...base, capabilities: ["future"] } as any),
    ],
    [
      "missing-visitor",
      /visitors\.sfc: must be a synchronous function$/,
      () => defineVisitorPlugin({ ...base, visitors: {} } as any),
    ],
    [
      "undeclared-visitor",
      /visitors\.style: has no matching capability declaration$/,
      () => defineVisitorPlugin({ ...base, visitors: { ...base.visitors, style() {} } } as any),
    ],
    [
      "invalid-cache-inputs",
      /cacheInputs\[0\]\.name: uses a reserved host prefix$/,
      () => defineVisitorPlugin({ ...base, cacheInputs: [{ name: "@vize/private", value: "x" }] }),
    ],
    [
      "async-visitor",
      /visitors\.sfc: asynchronous visitors are unsupported$/,
      () => defineVisitorPlugin({ ...base, visitors: { async sfc() {} } }),
    ],
  ];
  for (const [code, message, callback] of cases) expectFailure(callback, code, message);
});

test("the fixture harness freezes events and refuses asynchronous or returned results", () => {
  const promise = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "promise",
    version: "1",
    capabilities: ["format"] as const,
    cacheInputs: [],
    visitors: { format: () => Promise.resolve() as any },
  });
  const event = {
    apiVersion: VISITOR_API_VERSION,
    capability: "format" as const,
    file: "Button.vue",
    payload: { source: "<p>x</p>", formatted: "<p>x</p>\n" },
  };
  expectFailure(
    () => runVisitorFixture(promise, "format", event),
    "async-visitor",
    /visitors\.format: returned a Promise; asynchronous visitors are unsupported$/,
  );

  const returned = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "returned",
    version: "1",
    capabilities: ["format"] as const,
    cacheInputs: [],
    visitors: { format: (() => "changed") as any },
  });
  expectFailure(
    () => runVisitorFixture(returned, "format", event),
    "invalid-visitor-result",
    /visitors\.format: must return undefined$/,
  );
});
