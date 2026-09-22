// P4-16 (charter #29): one real custom JS rule through vitrine's napi lane —
// batched S2 visits, static fact demands, deterministic output, per-plugin
// cost in the lint output, content-keyed results. Needs the native build
// (`build:native:test`, which `test:scripts` runs first).
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { corpus, measure } from "./fixtures/davinci-plugin-sdk/bench.mjs";
import { definePlugin, runProxy } from "./fixtures/davinci-plugin-sdk/sdk.mjs";
import team from "./fixtures/davinci-plugin-sdk/team-conventions.mjs";
import { TODO_LIST } from "./fixtures/davinci-plugin-sdk/sources.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const native = createRequire(import.meta.url)(path.join(root, "npm/native/index.js"));
const lint = (source: string, plugins: unknown[], options = {}) =>
  native.lintWithPlugins(source, plugins, { filename: "TodoList.vue", ...options });

const EXPECTED = [
  {
    ruleId: "team-conventions/no-index-key",
    plugin: "team-conventions",
    severity: "warning",
    message: "Don't key a v-for by its index `i`; use a stable id such as `cell.id`.",
    start: 110,
    end: 118,
    line: 4,
    column: 42,
    endLine: 4,
    endColumn: 50,
  },
  {
    ruleId: "team-conventions/no-index-key",
    plugin: "team-conventions",
    severity: "warning",
    message: "Don't key a v-for by its index `index`; use a stable id such as `todo.id`.",
    start: 202,
    end: 214,
    line: 8,
    column: 40,
    endLine: 8,
    endColumn: 52,
  },
];

test("the team rule reports exactly the index keys, scope-correctly", () => {
  const out = lint(TODO_LIST, [team]);
  assert.deepEqual(out.diagnostics, EXPECTED);
  assert.equal(TODO_LIST.slice(110, 118), ':key="i"');
  assert.equal(TODO_LIST.slice(202, 214), ':key="index"');
});

test("the rule's output is byte-identical across two runs", () => {
  const [first, second] = [lint(TODO_LIST, [team]), lint(TODO_LIST, [team])];
  assert.equal(JSON.stringify(first.diagnostics), JSON.stringify(second.diagnostics));
  const stable = ({ name, version, contentKey, nodes, batchBytes, reports, cached }) =>
    JSON.stringify({ name, version, contentKey, nodes, batchBytes, reports, cached });
  assert.equal(first.plugins.map(stable).join(), second.plugins.map(stable).join());
});

test("every plugin's time is attributed in the lint output", () => {
  const slow = definePlugin({
    name: "slow-plugin",
    version: "0.0.1",
    visit: ["ui.element"],
    rules: {
      spin() {
        const until = process.hrtime.bigint() + 20_000_000n;
        while (process.hrtime.bigint() < until);
      },
    },
  });
  const out = lint(TODO_LIST, [team, slow]);
  assert.deepEqual(
    out.plugins.map(({ name, nodes, reports, cached }) => ({ name, nodes, reports, cached })),
    [
      { name: "team-conventions", nodes: 10, reports: 2, cached: false },
      { name: "slow-plugin", nodes: 8, reports: 0, cached: false },
    ],
  );
  const [fast, spin] = out.plugins;
  assert.equal(spin.jsNs >= 20_000_000, true);
  assert.equal(spin.elapsedNs >= spin.jsNs, true);
  assert.equal(fast.elapsedNs > 0 && fast.elapsedNs < spin.elapsedNs, true);
  assert.deepEqual(out.diagnostics, EXPECTED);
});

test("results are content-keyed by the plugin's own version and code", () => {
  const first = lint(TODO_LIST, [team], { cache: true });
  const hit = lint(TODO_LIST, [team], { cache: true });
  assert.equal(first.plugins[0].cached, false);
  assert.deepEqual(
    [hit.plugins[0].cached, hit.plugins[0].contentKey, hit.plugins[0].nodes],
    [true, first.plugins[0].contentKey, 0],
  );
  assert.equal(JSON.stringify(hit.diagnostics), JSON.stringify(first.diagnostics));
  const bumped = definePlugin({ ...team, version: "1.0.1" });
  const edited = definePlugin({ ...team, rules: { "no-index-key"() {} } });
  const keys = [bumped, edited].map((plugin) => lint(TODO_LIST, [plugin], { cache: true }));
  assert.deepEqual(
    keys.map((out) => [out.plugins[0].cached, out.plugins[0].contentKey === first.plugins[0].contentKey]),
    [
      [false, false],
      [false, false],
    ],
  );
});

test("demands are static: unknown groups refuse, undeclared reads throw", () => {
  const unknown = definePlugin({ ...team, demands: ["bindings"] });
  assert.throws(() => lint(TODO_LIST, [unknown]), {
    message:
      "team-conventions: fact group `bindings` is not available to JS plugins (available: templateScopes)",
  });
  const undeclared = definePlugin({ ...team, demands: [] });
  assert.throws(() => lint(TODO_LIST, [undeclared]), {
    message: "team-conventions/no-index-key: fact group `templateScopes` was not declared in demands",
  });
});

test("the measured proxy arm reads the same document to the same reports", () => {
  const handle = native.openPluginDocument(TODO_LIST, "TodoList.vue");
  const batch = JSON.parse(team.run(JSON.stringify(batchOf(handle))));
  assert.deepEqual(runProxy(team, handle), batch);
  assert.deepEqual(
    batch.map((report: { node: number }) => report.node),
    [7, 12],
  );
});

test("every measured arm (batch, proxy, sync, worker) agrees on real files", async () => {
  const files = corpus().slice(0, 12);
  for (const module of ["team-conventions.mjs", "design-system.mjs", "i18n.mjs"]) {
    const { arms } = await measure(files, 1, module);
    assert.deepEqual(arms.proxy, arms.sync, module);
    assert.deepEqual(arms.worker, arms.sync, module);
    const counts = arms.sync.map((nodes: string) => (nodes === "" ? 0 : nodes.split(",").length));
    assert.deepEqual(arms.batch, counts, module);
  }
});

// The batch the host would send, rebuilt from the proxy handle (so both
// arms are fed one document).
function batchOf(handle) {
  const nodes = [];
  const parents = [];
  const facts = [];
  for (let id = 0; id < handle.count(); id += 1) {
    parents.push(handle.parent(id));
    const kind = handle.kind(id);
    if (kind === "ui.for" || kind === "ui.bind") {
      const aliasValue = handle.field(id, "alias.value");
      const alias = aliasValue === null ? undefined : { value: aliasValue };
      nodes.push({ id, kind, name: handle.field(id, "name"), value: handle.field(id, "value"), alias });
    }
    const scope = handle.scope(id);
    if (scope) facts.push([id, scope]);
  }
  return { schema: 1, parents, nodes, facts: { templateScopes: facts } };
}
