/** Original #6890: complete routes, nine common cells, one explicit reference conflict. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import type { Block, VNode } from "vue";
import { Vue, window, installed, linkedFunction, transformVueJsxVapor } from "./runtime.ts";

type Original = {
  source: string;
  initial: { value: { $jsxValue: string } };
  steps: { patch: { value: { $jsxValue: string } } }[];
  expectedText: string[];
  expectedElementIdentities: number[][];
};
type Module = { code: string; functionName: string };
type ElementNode = ReturnType<typeof window.document.createElement>;
type DomNode = ElementNode["childNodes"][number];
type Observed = string | { tag: string; attributes: Record<string, string>; children: Observed[] };
type Phase = {
  dom: Observed[];
  text: string;
  elementIdentities: number[];
  warnings: string[];
  events: [];
  reads: number;
};
type Trace = { phases: Phase[]; readsAfterUnmount: number; completeUnmount: boolean };

const originalFile = new URL(
  "../../_fixtures/differential/compiler/jsx-child-values-6890/original.json",
  import.meta.url,
);
const original: Original = JSON.parse(fs.readFileSync(originalFile, "utf8"));
assert.equal(
  createHash("sha256").update(JSON.stringify(original)).digest("hex"),
  "9c7f9c6593fef9d8962be316a99e4066dd4f6e75d4936f2a028538c504105712",
);
assert.deepEqual(installed, { packageVersion: "3.2.26", vueVersion: "3.6.0-rc.10" });
const conflict = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/jsx/child-values-6890-reference-conflict.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  phase: number;
  vapor: Omit<Phase, "reads">;
  babel: Omit<Phase, "reads">;
};
assert.equal(conflict.phase, 8);

function observe(parent: { childNodes: Iterable<DomNode> }): Observed[] {
  const children: Observed[] = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      const text = node.textContent;
      if (!text) continue;
      const previous = children.at(-1);
      if (typeof previous === "string") children[children.length - 1] = previous + text;
      else children.push(text);
    } else if (node.nodeType === 1) {
      const element = node as ElementNode;
      children.push({
        tag: element.localName,
        attributes: Object.fromEntries(
          [...element.attributes]
            .map(({ name, value }): [string, string] => {
              assert.equal(typeof value, "string");
              return [name, value!];
            })
            .sort(([a], [b]) => a.localeCompare(b)),
        ),
        children: observe(element),
      });
    }
  }
  return children;
}

function value(name: string, nativeNode: boolean): unknown {
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
    case "nested-vnode": {
      if (!nativeNode)
        return Vue.h("i", { "data-kind": "nested" }, [
          Vue.h("b", null, "inner"),
          Vue.h("u", null, "leaf"),
        ]);
      const node = window.document.createElement("i");
      node.setAttribute("data-kind", "nested");
      for (const [tag, text] of [
        ["b", "inner"],
        ["u", "leaf"],
      ]) {
        const child = window.document.createElement(tag);
        child.textContent = text;
        node.append(child);
      }
      return node;
    }
    default:
      assert.fail(`unknown original value ${name}`);
  }
}

async function trace(
  module: Module,
  target: "vapor" | "vdom",
  authored: boolean,
  nativeNode = false,
): Promise<Trace> {
  let reads = 0;
  const reactive = Vue.shallowReactive<Record<string, unknown>>({
    value: value(original.initial.value.$jsxValue, nativeNode),
  });
  const state = new Proxy(reactive, {
    get(object, property, receiver) {
      if (property === "value") reads++;
      return Reflect.get(object, property, receiver);
    },
  });
  const render = linkedFunction(module.code, module.functionName, state, [], {
    propPublications: 0,
  });
  if (authored && target === "vapor") {
    assert.ok(
      render && typeof render === "object" && "__vapor" in render && render.__vapor === true,
    );
  } else assert.equal(typeof render, "function");
  const app =
    target === "vapor"
      ? Vue.createVaporApp(
          authored
            ? (render as Parameters<typeof Vue.createVaporApp>[0])
            : Vue.defineVaporComponent({ setup: () => (render as () => Block)() }),
        )
      : Vue.createApp({ setup: () => () => (render as () => VNode)() });
  const warnings: string[] = [];
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap<object, number>();
  let nextIdentity = 0;
  const snapshot = (): Phase => {
    const elements = [...host.querySelectorAll("*")];
    for (const element of elements)
      if (!identities.has(element)) identities.set(element, nextIdentity++);
    return {
      dom: observe(host),
      text: host.textContent,
      elementIdentities: elements.map((element) => identities.get(element)!),
      warnings: [...warnings],
      events: [],
      reads,
    };
  };
  let mounted = false;
  try {
    app.mount(host as unknown as Parameters<typeof app.mount>[0]);
    mounted = true;
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const step of original.steps) {
      state.value = value(step.patch.value.$jsxValue, nativeNode);
      await Vue.nextTick();
      phases.push(snapshot());
    }
    app.unmount();
    mounted = false;
    await Vue.nextTick();
    const before = reads;
    state.value = "after-unmount";
    await Vue.nextTick();
    assert.equal(reads, before, "disposed child ownership cannot keep evaluating authored getters");
    return { phases, readsAfterUnmount: reads, completeUnmount: host.childNodes.length === 0 };
  } finally {
    if (mounted) app.unmount();
    host.remove();
  }
}

let text = "";
for await (const chunk of process.stdin) text += chunk;
const input = JSON.parse(text) as {
  original: Original;
  vapor: Module;
  vdom: Module;
  shadowedSource: string;
  shadowedVapor: Module;
};
assert.deepEqual(
  input.original,
  original,
  "the full original input and desired vectors remain immutable",
);
const publicVapor = transformVueJsxVapor(
  original.source,
  "/semantic-oracle/children/dynamic-value.jsx?vapor=true",
);
assert.equal(typeof publicVapor.code, "string");
const publicBabel = transformSync(original.source, {
  configFile: false,
  babelrc: false,
  filename: "/semantic-oracle/children/dynamic-value.jsx",
  plugins: [babelJsx],
});
assert.equal(typeof publicBabel?.code, "string");
const referenceVapor: Module = { code: publicVapor.code!, functionName: "App" };
const referenceBabel: Module = { code: publicBabel!.code!, functionName: "App" };
assert.equal(
  input.shadowedSource,
  "const Node=1;const document=1;const Array=1;const String=1;" + original.source,
);
const publicShadowed = transformVueJsxVapor(
  input.shadowedSource,
  "/semantic-oracle/children/shadowed-value.jsx?vapor=true",
);
assert.equal(typeof publicShadowed.code, "string");
const referenceShadowed: Module = { code: publicShadowed.code!, functionName: "App" };
const capture: Record<string, unknown> = {
  original,
  installed,
  input,
  referenceVapor,
  referenceBabel,
  referenceShadowed,
  routes: {},
};
const routes = capture.routes as Record<
  string,
  Trace | { error: string; stack: string | undefined }
>;
process.on("exit", () => {
  if (process.env.VIZE_JSX_CHILD_TRACE_CAPTURE)
    fs.writeFileSync(
      process.env.VIZE_JSX_CHILD_TRACE_CAPTURE,
      JSON.stringify(capture, null, 2) + "\n",
    );
});
const work: [string, Module, "vapor" | "vdom", boolean, boolean][] = [
  ["public-vapor", referenceVapor, "vapor", false, false],
  ["public-babel", referenceBabel, "vdom", false, false],
  ["source-vapor", input.vapor, "vapor", true, false],
  ["source-vdom", input.vdom, "vdom", false, false],
  ["node-control-public", referenceVapor, "vapor", false, true],
  ["node-control-source", input.vapor, "vapor", true, true],
  ["shadowed-public", referenceShadowed, "vapor", false, false],
  ["shadowed-source", input.shadowedVapor, "vapor", true, false],
];
for (const [name, module, target, authored, nativeNode] of work) {
  try {
    routes[name] = await trace(module, target, authored, nativeNode);
  } catch (error) {
    routes[name] = {
      error: String(error),
      stack: error instanceof Error ? error.stack : undefined,
    };
  }
}
const get = (name: string): Trace => {
  const route = routes[name];
  assert.ok(route && "phases" in route, `${name}: complete runtime route ${JSON.stringify(route)}`);
  assert.equal(route.phases.length, 10);
  assert.equal(route.completeUnmount, true, `${name}: complete original unmount`);
  for (const [index, phase] of route.phases.entries()) {
    assert.deepEqual(phase.warnings, [], `${name}: all diagnostics retained`);
    assert.equal(phase.reads, index + 1, `${name}: one original getter evaluation per publication`);
  }
  return route;
};
const reference = get("public-vapor"),
  babel = get("public-babel"),
  actual = get("source-vapor"),
  vdom = get("source-vdom");
const withoutReads = ({ reads: _reads, ...phase }: Phase) => phase;
assert.deepEqual(
  withoutReads(reference.phases[8]),
  conflict.vapor,
  "complete independent observed Vapor frontier",
);
assert.deepEqual(
  withoutReads(babel.phases[8]),
  conflict.babel,
  "complete independent observed Babel frontier",
);
assert.notDeepEqual(conflict.vapor, conflict.babel);
const qualified: number[] = [];
for (let index = 0; index < 10; index++) {
  assert.equal(babel.phases[index].text, original.expectedText[index]);
  assert.deepEqual(
    babel.phases[index].elementIdentities,
    original.expectedElementIdentities[index],
  );
  if (index === 8) continue;
  assert.deepEqual(
    reference.phases[index],
    babel.phases[index],
    `independent references must agree in cell${index}`,
  );
  assert.deepEqual(
    actual.phases[index],
    reference.phases[index],
    `complete Vapor repair cell${index}`,
  );
  qualified.push(index);
}
// Matching the target-specific reference does not qualify this disputed cell.
assert.deepEqual(
  actual.phases[8],
  reference.phases[8],
  "retain the complete Vapor target outcome at the frontier",
);
const nodePublic = get("node-control-public"),
  nodeActual = get("node-control-source");
assert.deepEqual(
  nodeActual,
  nodePublic,
  "real DOM nodes, arrays, replacement and disposal require node ownership",
);
assert.equal(nodeActual.phases[8].text, "innerleaf");
assert.deepEqual(nodeActual.phases[8].elementIdentities, [0, 1, 2, 3]);
const shadowedPublic = get("shadowed-public"),
  shadowedActual = get("shadowed-source");
assert.deepEqual(
  shadowedActual,
  shadowedPublic,
  "authored Node/document/Array/String bindings cannot capture the DOM normalizer",
);
const receipt = {
  originalPhases: 10,
  qualifiedVaporPhases: qualified.length,
  qualifiedVaporIndices: qualified,
  referenceConflictPhases: 1,
  referenceConflictIndices: [8],
  qualificationTarget: "Vapor only",
  vdomMatchesBabel: vdom.phases.map(
    (phase, index) => JSON.stringify(phase) === JSON.stringify(babel.phases[index]),
  ),
  wholeCorpusQualified: false,
  issue6890Complete: false,
  unchangedMainOracleRows: 23,
  originalDesiredVectorsUnchanged: true,
  allFourCompleteRawRoutesRetained: true,
  physicalNodeControls: 10,
  authoredShadowingControls: 10,
};
capture.receipt = receipt;
await window.happyDOM.close();
console.log(JSON.stringify(receipt));
