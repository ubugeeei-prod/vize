// The JS half of the plugin lint host (#13074): every job arrives in one
// request; v2 rules see immutable records through a host-owned context and v1
// plugins keep their exact string packets.
import assert from "node:assert/strict";
import { test } from "node:test";

import { runPluginLintJobs } from "../../crates/vize_carton/src/config/loader/plugin-lint.js";
import { resolvePluginPlan } from "../../crates/vize_carton/src/config/loader/plugin-plan.js";
import { definePlugin } from "../../npm/plugin-sdk/index.js";
import { defineVizePlugin, type PluginLintRule } from "../../npm/plugin-sdk/v2.js";

const batch = JSON.stringify({
  schema: 1,
  plugin: "@acme/design",
  file: "Button.vue",
  parents: [-1, 0, 1],
  nodes: [
    { id: 0, kind: "ui.element", name: "div" },
    { id: 1, kind: "ui.element", name: "button", parent: 0 },
    { id: 2, kind: "ui.bind", name: "color", value: "'red'", parent: 1 },
  ],
  facts: { templateScopes: [[0, []]] },
});

function plan(rule: Partial<PluginLintRule> & Pick<PluginLintRule, "run">) {
  const design = defineVizePlugin<{ token: string }>({
    apiVersion: 2,
    stability: "experimental",
    name: "@acme/design",
    version: "1.0.0",
    setup(api) {
      api.lint.rule({
        id: "no-raw-color",
        meta: { messages: { raw: "Use {token} instead of {value}." }, fixable: "safe" },
        visit: ["ui.element", "ui.bind"],
        demands: ["templateScopes"],
        ...rule,
      });
    },
  });
  return resolvePluginPlan([design({ token: "$brand" })], { runtime: "node" });
}

const request = (
  resolved: { digest: string },
  capability = "lint.rule",
  key = "@acme/design/no-raw-color",
  plugin = "@acme/design",
) => ({
  digest: resolved.digest,
  jobs: [{ plugin, capability, key, documents: [{ file: "Button.vue", batch }] }],
});

test("a v2 rule reports through the host context once per document", () => {
  let runs = 0;
  const resolved = plan({
    run(context) {
      runs++;
      assert.equal(context.file, "Button.vue");
      assert.deepEqual(context.options, { token: "$brand" });
      assert.deepEqual([...context.facts("templateScopes").keys()], [0]);
      const bind = context.nodes.find((node) => node.kind === "ui.bind")!;
      assert.deepEqual(
        [...context.ancestors(bind, "ui.element")].map((node) => node.name),
        ["button", "div"],
      );
      assert.throws(() => {
        (bind as { value?: string }).value = "x";
      }, TypeError);
      context.report({
        node: bind,
        messageId: "raw",
        data: { token: "$brand", value: bind.value! },
        fix: "$brand",
      });
    },
  });
  const response = runPluginLintJobs(resolved, request(resolved));
  assert.equal(runs, 1);
  assert.equal(response.digest, resolved.digest);
  const [job] = response.jobs;
  assert.equal(job.capability, "lint.rule");
  assert.equal(typeof job.jsNs, "number");
  assert.deepEqual(job.documents, [
    {
      file: "Button.vue",
      reports: [
        { data: { token: "$brand", value: "'red'" }, fix: "$brand", messageId: "raw", node: 2 },
      ],
    },
  ]);
});

test("reports, facts and visits outside the declared contract fail closed", () => {
  const cases: [PluginLintRule["run"], RegExp][] = [
    [
      (context) => context.report({ node: { ...context.nodes[0]! }, messageId: "raw" }),
      /a report needs a visited node/,
    ],
    [
      (context) => context.report({ node: context.nodes[0]!, messageId: "other" as "raw" }),
      /undeclared message id `other`/,
    ],
    [
      (context) =>
        context.report({ node: context.nodes[0]!, messageId: "raw", data: { token: {} as never } }),
      /report data values/,
    ],
    [(context) => void context.facts("bindings"), /fact group `bindings` was not demanded/],
    [
      (context) => void [...context.ancestors(context.nodes[2]!, "ui.for")],
      /ancestors of kind ui.for need ui.for in visit/,
    ],
    [
      () => {
        throw new Error("boom");
      },
      /\[plugin\/rule-failed\] @acme\/design @acme\/design\/no-raw-color: Button\.vue: boom/,
    ],
    [(() => Promise.resolve()) as never, /asynchronous callbacks are unsupported/],
    [(() => 1) as never, /a rule returns nothing/],
  ];
  for (const [run, message] of cases) {
    const resolved = plan({ run });
    assert.throws(() => runPluginLintJobs(resolved, request(resolved)), message);
  }
  const unfixable = plan({
    meta: { messages: { raw: "raw" } },
    run(context) {
      context.report({ node: context.nodes[0]!, messageId: "raw", fix: "x" });
    },
  });
  assert.throws(() => runPluginLintJobs(unfixable, request(unfixable)), /fixes need meta.fixable/);
});

test("stale or unknown jobs are refused before any callback", () => {
  let runs = 0;
  const resolved = plan({ run: () => void runs++ });
  assert.throws(
    () => runPluginLintJobs(resolved, { ...request(resolved), digest: "0".repeat(64) }),
    /\[plugin\/plan-stale\]/,
  );
  assert.throws(
    () => runPluginLintJobs(undefined as never, request(resolved)),
    /no longer lists plugins/,
  );
  assert.throws(
    () => runPluginLintJobs(resolved, request(resolved, "sfc.custom-block")),
    /is not a lint capability of the resolved plan/,
  );
  assert.equal(runs, 0);
});

test("v1 plugins keep their exact string packets through the adapter", () => {
  const received: string[] = [];
  const v1 = definePlugin({
    name: "team-conventions",
    version: "1.0.0",
    visit: ["ui.bind"],
    demands: [],
    cacheInputs: [],
    rules: {
      check(ctx) {
        for (const node of ctx.nodes) ctx.report(node, `bind ${node.name}`);
      },
    },
  });
  const original = Reflect.get(v1, "run") as (json: string) => string;
  const spied = { ...v1, run: (json: string) => (received.push(json), original(json)) };
  const resolved = resolvePluginPlan([spied], { runtime: "node" });
  const response = runPluginLintJobs(
    resolved,
    request(resolved, "lint.plugin-v1", "team-conventions", "team-conventions"),
  );
  assert.deepEqual(received, [batch]);
  assert.equal(response.jobs[0].documents[0].reports, original(batch));
});
