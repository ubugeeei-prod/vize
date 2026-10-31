/** Real pre-review producer, repaired producer and unchanged Babel against the original ten values. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import { Vue, window, linkedFunction, installed } from "./runtime.ts";
import type { VNode } from "vue";

type Item = {
  kind: "sequence" | "slot";
  source: string;
  code: string;
  mapped: boolean;
  functionName: string;
  diagnostics: unknown[];
  optimizedParent: boolean;
};
type Packet = {
  original: {
    initial: { value: { $jsxValue: string } };
    steps: { patch: { value: { $jsxValue: string } } }[];
    expectedText: string[];
  };
  cases: Item[];
  refused: { source: string; code: string; map: unknown; diagnostics: unknown[] };
};
const [beforePath, afterPath, outputPath] = process.argv.slice(2);
assert.ok(beforePath && afterPath && outputPath);
const before = JSON.parse(fs.readFileSync(beforePath, "utf8")) as Packet;
const after = JSON.parse(fs.readFileSync(afterPath, "utf8")) as Packet;
const original = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/compiler/jsx-child-values-6890/original.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
assert.deepEqual(before.original, original);
assert.deepEqual(after.original, original);
assert.deepEqual(installed, { packageVersion: "3.2.26", vueVersion: "3.6.0-rc.10" });
assert.equal(before.cases.length, 6);
assert.equal(after.cases.length, 6);
const names = [
  original.initial.value.$jsxValue,
  ...original.steps.map(
    (s: { patch: { value: { $jsxValue: string } } }) => s.patch.value.$jsxValue,
  ),
];
assert.deepEqual(names, [
  "zero",
  "empty-string",
  "nan",
  "null",
  "undefined",
  "false",
  "true",
  "array",
  "nested-vnode",
  "zero",
]);

function value(name: string): unknown {
  switch (name) {
    case "zero":
      return 0;
    case "empty-string":
      return "";
    case "nan":
      return NaN;
    case "null":
      return null;
    case "undefined":
      return undefined;
    case "false":
      return false;
    case "true":
      return true;
    case "array":
      return [0, "", NaN, null, undefined, false];
    case "nested-vnode":
      return Vue.h("i", { "data-kind": "nested" }, [
        Vue.h("b", null, "inner"),
        Vue.h("u", null, "leaf"),
      ]);
    default:
      assert.fail(name);
  }
}
function observe(parent: Node): unknown[] {
  const result: unknown[] = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3 && node.textContent) {
      const previous = result.at(-1);
      if (typeof previous === "string") result[result.length - 1] = previous + node.textContent;
      else result.push(node.textContent);
    } else if (node.nodeType === 1) {
      const e = node as Element;
      result.push({
        tag: e.localName,
        attributes: Object.fromEntries(
          [...e.attributes].map((a) => [a.name, a.value]).sort(([a], [b]) => a!.localeCompare(b!)),
        ),
        children: observe(node),
      });
    }
  }
  return result;
}
async function trace(
  code: string,
  functionName: string,
  optimizedParent: boolean,
  runtimeFacts: unknown[],
) {
  let reads = 0;
  const effects: string[] = [];
  const reactive = Vue.shallowReactive<Record<string, unknown>>({
    value: value(names[0]!),
    record: (name: string) => {
      effects.push(name);
      return "leak-" + name;
    },
  });
  const state = new Proxy(reactive, {
    get(target, key, receiver) {
      if (key === "value") reads++;
      return Reflect.get(target, key, receiver);
    },
  });
  const render = linkedFunction(code, functionName, state, [], {
    propPublications: 0,
  }) as () => VNode;
  let owner: ReturnType<typeof Vue.getCurrentInstance>;
  const app = Vue.createApp({
    setup: () => {
      owner = Vue.getCurrentInstance();
      return () => {
        if (!optimizedParent) return render();
        Vue.openBlock();
        return Vue.createElementBlock("section", null, [render()]);
      };
    },
  });
  const recordRuntimeFacts = () => {
    const root = owner?.subTree;
    const host = optimizedParent
      ? (root?.children as VNode[] | null)?.[0]?.component
      : root?.component;
    const slotFlag = (host?.slots as { _?: number } | undefined)?._ ?? null;
    const fragment = host?.subTree as (VNode & { dynamicChildren?: VNode[] | null }) | undefined;
    runtimeFacts.push({
      optimizedParent,
      slotFlag,
      fragmentPatchFlag: fragment?.patchFlag ?? null,
      fragmentDynamicChildrenLength: Array.isArray(fragment?.dynamicChildren)
        ? fragment.dynamicChildren.length
        : null,
    });
  };
  const warnings: string[] = [];
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const ids = new WeakMap<object, number>();
  let next = 0,
    mounted = false;
  const snapshot = () => ({
    dom: observe(host as unknown as Node),
    text: host.textContent,
    elementIdentities: [...host.querySelectorAll("*")].map((e) => {
      if (!ids.has(e)) ids.set(e, next++);
      return ids.get(e)!;
    }),
    warnings: [...warnings],
    reads,
    effects: [...effects],
  });
  try {
    app.mount(host as unknown as Parameters<typeof app.mount>[0]);
    mounted = true;
    await Vue.nextTick();
    recordRuntimeFacts();
    const phases = [snapshot()];
    for (const name of names.slice(1)) {
      state.value = value(name);
      await Vue.nextTick();
      recordRuntimeFacts();
      phases.push(snapshot());
    }
    app.unmount();
    mounted = false;
    await Vue.nextTick();
    const disposed = { reads, effects: [...effects] };
    state.value = "after-unmount";
    await Vue.nextTick();
    assert.deepEqual(
      { reads, effects },
      disposed,
      "disposed slot/expression cannot consume another getter or side effect",
    );
    return {
      phases,
      readsAfterUnmount: reads,
      effectsAfterUnmount: [...effects],
      completeUnmount: host.childNodes.length === 0,
    };
  } finally {
    if (mounted) app.unmount();
    host.remove();
  }
}
const capture = {
  before,
  after,
  installed,
  cases: [] as unknown[],
  wholeCorpusQualified: false,
  issue6890Complete: false,
};
let failure: unknown;
try {
  assert.equal(after.refused.source, before.refused.source);
  assert.deepEqual(
    after.refused.diagnostics,
    before.refused.diagnostics,
    "whole original rejected-source diagnostic and authored byte span",
  );
  assert.equal(after.refused.diagnostics.length, 1);
  for (const [index, item] of after.cases.entries()) {
    const old = before.cases[index]!;
    for (const key of [
      "kind",
      "source",
      "mapped",
      "functionName",
      "diagnostics",
      "optimizedParent",
    ] as const)
      assert.deepEqual(item[key], old[key]);
    const reference = transformSync(item.source, {
      configFile: false,
      babelrc: false,
      filename: "review-original.jsx",
      plugins: [babelJsx],
    });
    assert.equal(typeof reference?.code, "string");
    const runtimeFacts = {
      before: [] as unknown[],
      after: [] as unknown[],
      reference: [] as unknown[],
    };
    const routes = {
      before: await trace(old.code, item.functionName, item.optimizedParent, runtimeFacts.before),
      after: await trace(item.code, item.functionName, item.optimizedParent, runtimeFacts.after),
      reference: await trace(
        reference!.code!,
        item.functionName,
        item.optimizedParent,
        runtimeFacts.reference,
      ),
    };
    const matches = routes.before.phases.map(
      (p, i) => JSON.stringify(p) === JSON.stringify(routes.reference.phases[i]),
    );
    capture.cases.push({
      item,
      reference: { code: reference!.code },
      routes,
      beforeMatchesBabel: matches,
      runtimeFacts,
    });
  }
  for (const [index, raw] of capture.cases.entries()) {
    const result = raw as {
      item: Item;
      routes: Record<"before" | "after" | "reference", Awaited<ReturnType<typeof trace>>>;
      beforeMatchesBabel: boolean[];
      runtimeFacts: Record<"before" | "after" | "reference", { slotFlag: number | null }[]>;
    };
    const { item, routes } = result;
    assert.deepEqual(
      result.beforeMatchesBabel,
      item.kind === "sequence"
        ? Array(10).fill(false)
        : item.optimizedParent
          ? [true, false, false, false, false, false, false, false, false, true]
          : Array(10).fill(true),
      "retain exact genuine pre-review regression",
    );
    if (item.kind === "slot")
      for (const role of ["before", "after", "reference"] as const) {
        const expectedFlag =
          item.optimizedParent && role !== "reference" ? (role === "before" ? 1 : 2) : null;
        assert.deepEqual(
          result.runtimeFacts[role].map((f) => f.slotFlag),
          Array(10).fill(expectedFlag),
          "actual optimized-parent slot flag witness",
        );
      }
    for (const route of Object.values(routes)) {
      assert.equal(route.phases.length, 10);
      assert.equal(route.readsAfterUnmount, 10);
      assert.equal(route.completeUnmount, true);
      route.phases.forEach((phase, i) => {
        assert.deepEqual(phase.warnings, []);
        assert.equal(phase.reads, i + 1);
        assert.deepEqual(
          phase.effects,
          item.kind === "sequence"
            ? Array.from({ length: i + 1 }, () => ["left", "right"]).flat()
            : [],
        );
      });
    }
    routes.reference.phases.forEach((phase, i) =>
      assert.equal(phase.text, original.expectedText[i]),
    );
    assert.deepEqual(
      routes.after,
      routes.reference,
      "full slot/expression DOM/order/identity/getters/evaluation order/disposal " + index,
    );
  }
} catch (error) {
  failure = error;
} finally {
  fs.writeFileSync(outputPath, JSON.stringify(capture, null, 2) + "\n");
}
if (failure) throw failure;
