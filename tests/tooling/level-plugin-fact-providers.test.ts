// Real providers cross the native host, feed declared rules and invalidate results.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { defineFactProvider, definePlugin } from "../../npm/plugin-sdk/index.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const native = createRequire(import.meta.url)(path.join(root, "npm/native/index.js"));
const source = '<template><button class="danger">Go</button></template>';
const lint = (plugins, factProviders, options = {}) =>
  native.lintWithPlugins(source, plugins, {
    filename: "ProviderButton.vue",
    factProviders,
    ...options,
  });

function rule(name = "contrast-rule", demands = ["tokens/colors"]) {
  return definePlugin({
    name,
    version: "1",
    visit: ["ui.element"],
    demands,
    cacheInputs: [],
    rules: {
      contrast(ctx) {
        const colors = ctx.facts(demands[0]);
        for (const node of ctx.nodes) {
          const color = colors.get(node.id);
          if (typeof color === "string") ctx.report(node, `Use contrast for ${color}`);
        }
      },
    },
  });
}

function provider(color = "red", version = "1", cacheInputs = []) {
  return defineFactProvider({
    name: "tokens",
    version,
    visit: ["ui.element"],
    provides: ["tokens/colors"],
    cacheInputs,
    provide(batch) {
      assert.deepEqual(Object.keys(batch.facts), []);
      return { "tokens/colors": batch.nodes.map((node) => [node.id, color]) };
    },
  });
}

test("a native rule consumes provided facts and reports on the host's actual span", () => {
  const result = lint([rule()], [provider()]);
  assert.equal(result.diagnostics.length, 1);
  const diagnostic = result.diagnostics[0];
  assert.equal(diagnostic.ruleId, "contrast-rule/contrast");
  assert.equal(diagnostic.message, "Use contrast for red");
  assert.equal(
    source.slice(diagnostic.start, diagnostic.end),
    '<button class="danger">Go</button>',
  );
  assert.deepEqual(
    result.factProviders.map(({ name, providedGroups, audited }) => ({
      name,
      providedGroups,
      audited,
    })),
    [{ name: "tokens", providedGroups: 1, audited: true }],
  );
  assert.ok(result.factProviders[0].elapsedNs >= result.factProviders[0].jsNs);
  assert.ok(result.factProviders[0].batchBytes > 0);
});

test("the demanded closure runs once per document and never leaks dependencies into the consumer", () => {
  let calls = 0;
  const tokens = provider();
  const counted = {
    ...tokens,
    run(batch) {
      calls += 1;
      return tokens.run(batch);
    },
  };
  const usage = defineFactProvider({
    name: "usage",
    version: "1",
    visit: ["ui.element"],
    provides: ["usage/contrast"],
    demands: ["tokens/colors"],
    cacheInputs: [],
    provide(batch) {
      assert.deepEqual(Object.keys(batch.facts), ["tokens/colors"]);
      return { "usage/contrast": batch.facts["tokens/colors"] };
    },
  });
  const unused = defineFactProvider({
    name: "unused",
    version: "1",
    provides: ["unused/secret"],
    cacheInputs: [],
    provide() {
      throw new Error("an undemanded provider must not run");
    },
  });
  const first = rule("closure-one", ["usage/contrast"]);
  const inspected = {
    ...first,
    run(batch) {
      assert.deepEqual(Object.keys(JSON.parse(batch).facts), ["usage/contrast"]);
      return first.run(batch);
    },
  };
  const result = lint(
    [inspected, rule("closure-two", ["usage/contrast"])],
    [usage, counted, unused],
  );
  assert.equal(calls, 2, "one provider execution has two same-input audit calls");
  assert.equal(result.diagnostics.length, 2);
  assert.deepEqual(
    result.factProviders.map(({ name }) => name),
    ["tokens", "usage"],
  );
});

test("provider version, code and ambient inputs invalidate a cached consumer", () => {
  const consumer = rule("provider-manifest-cache");
  const baseline = provider("red", "cache-1", [{ name: "palette", value: "red" }]);
  const first = lint([consumer], [baseline], { cache: true });
  const hit = lint([consumer], [baseline], { cache: true });
  assert.deepEqual([first.plugins[0].cached, hit.plugins[0].cached], [false, true]);
  assert.equal(first.plugins[0].contentKey, hit.plugins[0].contentKey);
  for (const changed of [
    { ...baseline, version: "cache-2" },
    { ...baseline, fingerprint: "changed-provider-code" },
    { ...baseline, cacheInputs: [{ name: "palette", value: "blue" }] },
  ]) {
    const result = lint([consumer], [changed], { cache: true });
    assert.equal(result.plugins[0].cached, false);
    assert.notEqual(result.plugins[0].contentKey, first.plugins[0].contentKey);
    assert.notEqual(result.factProviders[0].contentKey, first.factProviders[0].contentKey);
  }
});

test("audited provider tables and consumer diagnostics reuse their independent cache entries", () => {
  let calls = 0;
  const tokens = defineFactProvider({
    name: "tokens",
    version: "provider-warm-cache",
    provides: ["tokens/colors"],
    cacheInputs: [],
    provide() {
      calls += 1;
      return { "tokens/colors": [[0, "red"]] };
    },
  });
  const consumer = rule("provider-warm-cache");
  const first = lint([consumer], [tokens], { cache: true });
  const hit = lint([consumer], [tokens], { cache: true });
  assert.deepEqual([first.factProviders[0].cached, hit.factProviders[0].cached], [false, true]);
  assert.deepEqual([first.factProviders[0].audited, hit.factProviders[0].audited], [true, false]);
  assert.deepEqual([first.plugins[0].cached, hit.plugins[0].cached], [false, true]);
  assert.equal(first.plugins[0].contentKey, hit.plugins[0].contentKey);
  assert.deepEqual(hit.diagnostics, first.diagnostics);
  assert.equal(hit.factProviders[0].jsNs, 0);
  assert.equal(calls, 2);
});

test("provider manifests reject primary overwrites, namespace theft and cyclic or unknown demands before calls", () => {
  let calls = 0;
  const base = {
    ...provider(),
    run() {
      calls += 1;
      return "{}";
    },
  };
  for (const bad of [
    { ...base, provides: ["templateScopes"] },
    { ...base, provides: ["other/colors"] },
    { ...base, name: "@vize", provides: ["@vize/colors"] },
    { ...base, provides: ["tokens/colors", "tokens/colors"] },
    { ...base, cacheInputs: undefined },
    { ...base, cacheInputs: [{ name: "@vize/fact:tokens/colors", value: "spoof" }] },
    { ...base, demands: ["missing/colors"] },
    { ...base, demands: ["tokens/colors"] },
  ])
    assert.throws(() => lint([rule()], [bad]));
  const a = { ...base, name: "a", provides: ["a/data"], demands: ["b/data"] };
  const b = { ...base, name: "b", provides: ["b/data"], demands: ["a/data"] };
  assert.throws(() => lint([], [a, b]), /provider dependency cycle/);
  assert.equal(calls, 0);
});

test("native providers refuse undeclared output, malformed tables and same-input nondeterminism", () => {
  const base = provider();
  for (const output of [
    "{}",
    '{"tokens/colors":[],"templateScopes":[]}',
    '{"tokens/colors":{}}',
    '{"tokens/colors":[[0,"red"],[0,"blue"]]}',
    '{"tokens/colors":[[{},"red"]]}',
  ])
    assert.throws(() => lint([rule()], [{ ...base, run: () => output }]), /invalid fact provider/);
  let calls = 0;
  const unstable = {
    ...base,
    run() {
      calls += 1;
      return JSON.stringify({ "tokens/colors": [[0, calls % 2 ? "red" : "blue"]] });
    },
  };
  assert.throws(() => lint([rule()], [unstable]), /same-input runs produced different fact tables/);
});

test("rules and providers cannot read undeclared tables", () => {
  const undeclared = definePlugin({
    name: "undeclared-provider-rule",
    version: "1",
    demands: [],
    cacheInputs: [],
    rules: {
      bad(ctx) {
        ctx.facts("tokens/colors");
      },
    },
  });
  assert.throws(() => lint([undeclared], [provider()]), /was not declared in demands/);
  const inspected = defineFactProvider({
    name: "inspect",
    version: "1",
    provides: ["inspect/data"],
    demands: [],
    cacheInputs: [],
    provide(batch) {
      assert.deepEqual(batch.facts, {});
      return { "inspect/data": [] };
    },
  });
  const result = lint([rule("empty-inspect", ["inspect/data"])], [provider(), inspected]);
  assert.deepEqual(
    result.factProviders.map(({ name }) => name),
    ["inspect"],
  );
});
