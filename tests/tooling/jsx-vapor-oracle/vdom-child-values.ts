/** Real VNode block ownership and multi-update controls for the original #6890 values. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import type { VNode } from "vue";
import { Vue, window, linkedFunction, installed } from "./runtime.ts";

const [inputPath, outputPath] = process.argv.slice(2);
assert.ok(inputPath && outputPath);
const input = JSON.parse(fs.readFileSync(inputPath, "utf8")) as {
  original: {
    initial: { value: { $jsxValue: string } };
    steps: { patch: { value: { $jsxValue: string } } }[];
    expectedText: string[];
  };
  cases: { source: string; code: string; functionName: string; tail: string }[];
};
const baseline = process.env.VIZE_JSX_VDOM_EXPECT_OLD === "1";
assert.deepEqual(installed, { packageVersion: "3.2.26", vueVersion: "3.6.0-rc.10" });
const names = [
  input.original.initial.value.$jsxValue,
  ...input.original.steps.map((step) => step.patch.value.$jsxValue),
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
      return Number.NaN;
    case "null":
      return null;
    case "undefined":
      return undefined;
    case "false":
      return false;
    case "true":
      return true;
    case "array":
      return [0, "", Number.NaN, null, undefined, false];
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
    if (node.nodeType === 3) {
      const text = node.textContent;
      if (!text) continue;
      const previous = result.at(-1);
      if (typeof previous === "string") result[result.length - 1] = previous + text;
      else result.push(text);
    } else if (node.nodeType === 1) {
      const element = node as Element;
      result.push({
        tag: element.localName,
        attributes: Object.fromEntries(
          [...element.attributes]
            .map((a) => [a.name, a.value])
            .sort(([a], [b]) => a!.localeCompare(b!)),
        ),
        children: observe(element),
      });
    }
  }
  return result;
}
async function trace(code: string, functionName: string) {
  let reads = 0;
  const reactive = Vue.shallowReactive<Record<string, unknown>>({ value: value(names[0]!) });
  const state = new Proxy(reactive, {
    get(object, key, receiver) {
      if (key === "value") reads++;
      return Reflect.get(object, key, receiver);
    },
  });
  const render = linkedFunction(code, functionName, state, [], {
    propPublications: 0,
  }) as () => VNode;
  const warnings: string[] = [];
  const app = Vue.createApp({ setup: () => () => render() });
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap<object, number>();
  let next = 0;
  const snapshot = () => ({
    dom: observe(host as unknown as Node),
    text: host.textContent,
    elementIdentities: [...host.querySelectorAll("*")].map((e) => {
      if (!identities.has(e)) identities.set(e, next++);
      return identities.get(e)!;
    }),
    warnings: [...warnings],
    reads,
  });
  let mounted = false;
  try {
    app.mount(host as unknown as Parameters<typeof app.mount>[0]);
    mounted = true;
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const name of names.slice(1)) {
      state.value = value(name);
      await Vue.nextTick();
      phases.push(snapshot());
    }
    app.unmount();
    mounted = false;
    await Vue.nextTick();
    const before = reads;
    state.value = "after-unmount";
    await Vue.nextTick();
    assert.equal(reads, before, "disposed render cannot consume another getter");
    return { phases, readsAfterUnmount: reads, completeUnmount: host.childNodes.length === 0 };
  } finally {
    if (mounted) app.unmount();
    host.remove();
  }
}
const capture: {
  input: typeof input;
  installed: typeof installed;
  baseline: boolean;
  cases: unknown[];
} = { input, installed, baseline, cases: [] };
process.on("exit", () => fs.writeFileSync(outputPath, JSON.stringify(capture, null, 2) + "\n"));
for (const item of input.cases) {
  const reference = transformSync(item.source, {
    configFile: false,
    babelrc: false,
    filename: "vdom-original-6890.jsx",
    plugins: [babelJsx],
  });
  assert.equal(typeof reference?.code, "string");
  const actual = await trace(item.code, item.functionName),
    expected = await trace(reference!.code!, item.functionName);
  const state = { value: 0 };
  const render = linkedFunction(item.code, item.functionName, state, [], {
    propPublications: 0,
  }) as () => VNode;
  Vue.openBlock();
  const before = Vue.createVNode("b", null, "before", 1);
  const vnode = render();
  const after = Vue.createVNode("u", null, "after", 1);
  const parent = Vue.createElementBlock("section", null, [before, vnode, after]);
  assert.ok("dynamicChildren" in vnode && "dynamicChildren" in parent);
  const childDynamic = vnode.dynamicChildren;
  const parentDynamic = parent.dynamicChildren;
  assert.ok(childDynamic === null || Array.isArray(childDynamic));
  assert.ok(Array.isArray(parentDynamic));
  const block = {
    dynamicChildrenNull: childDynamic === null,
    parentOwnsBefore: parentDynamic.includes(before),
    parentOwnsAfter: parentDynamic.includes(after),
    parentDynamicCount: parentDynamic.length,
  };
  const matches = actual.phases.map(
    (phase, i) => JSON.stringify(phase) === JSON.stringify(expected.phases[i]),
  );
  capture.cases.push({
    item,
    reference: { code: reference!.code },
    actual,
    expected,
    block,
    matches,
  });
  for (const route of [actual, expected]) {
    assert.equal(route.phases.length, 10);
    assert.equal(route.completeUnmount, true);
    assert.equal(route.readsAfterUnmount, 10);
    route.phases.forEach((phase, i) => {
      assert.deepEqual(phase.warnings, []);
      assert.equal(phase.reads, i + 1);
    });
  }
  expected.phases.forEach((phase, i) =>
    assert.equal(phase.text, input.original.expectedText[i]! + item.tail),
  );
  assert.equal(block.parentOwnsBefore, true, "child producer must not close its parent's block");
  assert.equal(block.parentOwnsAfter, true, "child producer must not replace its parent's block");
  if (!baseline) {
    assert.equal(
      block.dynamicChildrenNull,
      true,
      "opaque raw child requires actual null, not a truthy empty block array",
    );
    assert.deepEqual(
      actual,
      expected,
      "full DOM/order/identity/getters/disposal against unchanged independent Babel",
    );
  } else
    assert.deepEqual(
      matches,
      [true, true, true, true, true, false, false, false, false, true],
      "retain actual old failure, do not rewrite its oracle",
    );
}
await window.happyDOM.close();
console.log(
  JSON.stringify({
    cases: capture.cases.length,
    phasesPerRoute: 10,
    baseline,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  }),
);
