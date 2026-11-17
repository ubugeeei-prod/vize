// Plugin API v2 contract (#13074): manifests lower to one canonical plan that
// the native verifier re-reads byte for byte. The resolve fixtures are shared
// with `crates/vize_carton/src/config/plugin_plan/tests.rs`. Regenerate with:
//   VIZE_PLUGIN_PLAN_FIXTURES=write node --test tests/tooling/plugin-plan.test.ts
import assert from "node:assert/strict";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import {
  admitPluginPlan,
  inspectPluginPlan,
  pluginPlanRefusals,
  verifyPluginPlan,
} from "../../crates/vize_carton/src/config/loader/plugin-inspect.js";
import {
  PLUGIN_ENTRY,
  PLUGIN_REGISTRY,
  VizePluginPlanError,
  callbackKey,
  resolvePluginPlan,
} from "../../crates/vize_carton/src/config/loader/plugin-plan.js";
import { definePlugin } from "../../npm/plugin-sdk/index.js";
import { PLUGIN_ENTRY as SDK_ENTRY, defineVizePlugin } from "../../npm/plugin-sdk/v2.js";

type Manifest = { capabilities: { id: string }[] } & Record<string, unknown>;
type FixtureEntry = {
  manifest?: Manifest;
  v1?: Record<string, unknown> & { rules: string[] };
  skip?: false | null;
  descriptorApiVersion?: number;
  missingCallbacks?: boolean;
};
type Fixture = {
  entries: FixtureEntry[];
  runtime: string;
  expect: unknown;
};

const fixtures = new URL("../_fixtures/plugin-plan/resolve/", import.meta.url);
const noop = () => {};

function entryOf(fixture: FixtureEntry) {
  if ("skip" in fixture) return fixture.skip;
  if (fixture.v1) {
    const { rules, ...plugin } = fixture.v1;
    return { ...plugin, rules: Object.fromEntries(rules.map((rule) => [rule, noop])), run: noop };
  }
  const manifest = fixture.manifest!;
  const callbacks = manifest.capabilities.map(({ id }) =>
    fixture.missingCallbacks
      ? {}
      : Object.fromEntries(
          (PLUGIN_REGISTRY.capabilities[id]?.callbacks ?? []).map((name: string) => [name, noop]),
        ),
  );
  return {
    [PLUGIN_ENTRY]: { apiVersion: fixture.descriptorApiVersion ?? 2, callbacks, manifest },
  };
}

function outcome(fixture: Fixture): Record<string, unknown> {
  try {
    const resolved = resolvePluginPlan(fixture.entries.map(entryOf), {
      runtime: fixture.runtime as "node" | "bun",
    });
    return { digest: resolved.digest, json: resolved.json, plan: resolved.plan };
  } catch (error) {
    if (!(error instanceof VizePluginPlanError)) throw error;
    return { error: { code: error.code, message: error.message } };
  }
}

test("shared resolve fixtures produce their canonical plans or fail closed", () => {
  const names = readdirSync(fixtures)
    .filter((name) => name.endsWith(".json"))
    .sort();
  assert.ok(names.length >= 30);
  for (const name of names) {
    const file = new URL(name, fixtures);
    const fixture = JSON.parse(readFileSync(file, "utf8")) as Fixture;
    const { json, ...expect } = outcome(fixture) as { json?: string; digest?: string };
    if (process.env.VIZE_PLUGIN_PLAN_FIXTURES === "write") {
      writeFileSync(file, `${JSON.stringify({ ...fixture, expect }, null, 2)}\n`);
      continue;
    }
    assert.deepEqual(expect, fixture.expect, name);
    assert.equal("error" in expect, name.startsWith("error-"), name);
    if (json !== undefined) assert.equal(verifyPluginPlan(json, expect.digest).planSchema, 1);
  }
});

test("the SDK lowers setup once into an entry the host plans", () => {
  let setups = 0;
  const options = { strict: true, tokens: ["brand"] };
  const designSystem = defineVizePlugin({
    apiVersion: 2,
    stability: "experimental",
    name: "@acme/design-system",
    version: "0.1.0",
    cacheInputs: [{ name: "tokens", value: "v1" }],
    setup(api, authored: typeof options) {
      setups++;
      api.lint.rule({
        id: "no-raw-color",
        meta: { messages: { raw: "Use {token}." }, defaultLevel: "warn", fixable: "safe" },
        visit: ["ui.bind"],
        run(context) {
          if (authored.strict) context.report({ node: context.nodes[0]!, messageId: "raw" });
        },
      });
      api.sfc.block("design", { lang: "json" });
    },
  });
  const entry = designSystem(options);
  assert.equal(SDK_ENTRY, PLUGIN_ENTRY);
  assert.equal(setups, 1);
  assert.equal(Object.isFrozen(options), false, "authored options stay mutable");
  const resolved = resolvePluginPlan([entry], { runtime: "node" });
  assert.equal(setups, 1, "planning never re-runs setup");
  const [plugin] = resolved.plan.plugins;
  assert.deepEqual(
    plugin.capabilities.map(({ id, key }: { id: string; key: string }) => [id, key]),
    [
      ["lint.rule", "@acme/design-system/no-raw-color"],
      ["sfc.custom-block", "design:json"],
    ],
  );
  assert.deepEqual(plugin.options, { strict: true, tokens: ["brand"] });
  assert.match(plugin.fingerprint, /^[0-9a-f]{64}$/);
  const run = resolved.callbacks.get(
    callbackKey("@acme/design-system", "lint.rule", "@acme/design-system/no-raw-color", "run"),
  );
  assert.equal(typeof run, "function");
  assert.equal(resolved.callbacks.size, 1);
  assert.equal(resolved.entries.get("@acme/design-system"), entry);
  // A different option or rule body is a different plugin identity.
  assert.notEqual(
    resolvePluginPlan([designSystem({ strict: false, tokens: [] })]).plan.plugins[0]?.fingerprint,
    plugin.fingerprint,
  );
  // Identical inputs are byte-identical plans.
  assert.equal(resolvePluginPlan([designSystem(options)], { runtime: "node" }).json, resolved.json);
});

test("authored data that is not canonical fails closed before planning", () => {
  const plugin = defineVizePlugin({
    apiVersion: 2,
    stability: "experimental",
    name: "opts",
    version: "1",
    setup(api) {
      api.lint.rule({ id: "a", meta: { messages: { m: "m" } }, visit: ["ui.element"], run() {} });
    },
  });
  for (const [options, message] of [
    [{ callback: () => 1 }, /options\.callback is not canonical JSON data/],
    [{ when: new Date(0) }, /options\.when must be a plain object/],
    [{ big: 2 ** 60 }, /options\.big must be a safe integer/],
    [{ text: "\uD800" }, /options\.text contains a lone surrogate/],
  ] as const) {
    assert.throws(() => resolvePluginPlan([plugin(options as never)]), message);
  }
  assert.throws(
    () =>
      defineVizePlugin({
        apiVersion: 2,
        stability: "experimental",
        name: "x",
        version: "1",
        setup(api) {
          api.lint.rule({
            id: "a",
            meta: { messages: {}, severity: "error" } as never,
            visit: [],
            run() {},
          });
        },
      })(),
    /meta has unknown field `severity`/,
  );
  assert.throws(
    () => resolvePluginPlan({} as never),
    /\[plugin\/entry-invalid\] vue\.plugins: must be an array/,
  );
  assert.throws(() => resolvePluginPlan([[plugin()]]), /nested arrays and promises/);
  assert.throws(() => resolvePluginPlan([Promise.resolve(plugin())]), /nested arrays and promises/);
});

test("v1 rule plugins are hosted through the adapter without changing the plugin object", () => {
  const v1 = definePlugin({
    name: "boolean-attributes",
    version: "1.0.0",
    visit: ["ui.bind"],
    demands: [],
    cacheInputs: [],
    rules: { shorthand() {} },
  });
  const resolved = resolvePluginPlan([v1], { runtime: "node" });
  assert.equal(resolved.entries.get("boolean-attributes"), v1);
  assert.equal(
    resolved.callbacks.get(
      callbackKey("boolean-attributes", "lint.plugin-v1", "boolean-attributes", "run"),
    ),
    Reflect.get(v1, "run"),
  );
  assert.deepEqual(resolved.plan.plugins[0].capabilities[0].data, {
    demands: [],
    rules: ["shorthand"],
    visit: ["ui.bind"],
  });
  assert.equal(resolved.plan.plugins[0].fingerprint, v1.fingerprint);
  assert.throws(
    () =>
      resolvePluginPlan([
        { name: "t", version: "1", fingerprint: "f", run: noop, family: "output" },
      ]),
    /\[plugin\/entry-invalid\].*other v1 hooks keep their native APIs/,
  );
});

test("products refuse capabilities they do not implement and inspection says why", () => {
  const routes = defineVizePlugin({
    apiVersion: 2,
    stability: "experimental",
    name: "@acme/routes",
    version: "1",
    supports: { outputs: ["vdom"] },
    setup(api) {
      api.sfc.block("route", { lang: "json" });
    },
  });
  const resolved = resolvePluginPlan([routes()], { runtime: "node" });
  assert.deepEqual(pluginPlanRefusals(resolved.plan, "lint"), [
    {
      capability: "sfc.custom-block",
      key: "route:json",
      plugin: "@acme/routes",
      reason: "unadmitted",
    },
  ]);
  assert.throws(
    () => admitPluginPlan(resolved.plan, "compiler"),
    /\[plugin\/capability-unadmitted\] @acme\/routes: compiler does not implement sfc\.custom-block `route:json` yet/,
  );
  assert.throws(() => admitPluginPlan(resolved.plan, "bundler"), /\[plugin\/product-unknown\]/);
  const inspected = inspectPluginPlan(resolved);
  assert.equal(inspected.digest, resolved.digest);
  assert.deepEqual(inspected.order, ["@acme/routes"]);
  assert.deepEqual(
    Object.fromEntries(
      Object.entries(inspected.admission).map(([product, { status }]) => [product, status]),
    ),
    {
      compiler: "refused",
      typecheck: "refused",
      lint: "refused",
      format: "refused",
      lsp: "refused",
    },
  );
  const empty = resolvePluginPlan([], { runtime: "node" });
  admitPluginPlan(empty.plan, "compiler");
  assert.deepEqual(
    Object.values(inspectPluginPlan(empty).admission).map(({ status }) => status),
    ["unused", "unused", "unused", "unused", "unused"],
  );
});

test("verification rejects stale and non-canonical plan bytes", () => {
  const resolved = resolvePluginPlan([], { runtime: "node" });
  assert.equal(resolved.json, '{"claims":[],"planSchema":1,"plugins":[]}');
  assert.deepEqual(verifyPluginPlan(resolved.json, resolved.digest), resolved.plan);
  assert.throws(() => verifyPluginPlan(resolved.json, "0".repeat(64)), /\[plugin\/plan-stale\]/);
  assert.throws(
    () => verifyPluginPlan('{"planSchema":1,"claims":[],"plugins":[]}'),
    /\[plugin\/plan-invalid\] plan: is not in canonical form/,
  );
  assert.throws(
    () => verifyPluginPlan('{"claims":[],"planSchema":2,"plugins":[]}'),
    /schema 2 is not supported/,
  );
});

test("the runtime selection is explicit and part of host validation", () => {
  const bunOnly = defineVizePlugin({
    apiVersion: 2,
    stability: "experimental",
    name: "bun-only",
    version: "1",
    supports: { runtimes: ["bun"] },
    setup(api) {
      api.vue.directive("focus");
    },
  });
  assert.equal(resolvePluginPlan([bunOnly()], { runtime: "bun" }).plan.plugins.length, 1);
  assert.throws(
    () => resolvePluginPlan([bunOnly()], { runtime: "node" }),
    /\[plugin\/host-unsupported\] bun-only: does not support the node runtime/,
  );
});
