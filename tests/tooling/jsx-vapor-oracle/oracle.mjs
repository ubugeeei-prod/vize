import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import {
  Vue,
  window,
  installed,
  componentFixtures,
  linkedFunction,
  transformVueJsxVapor,
} from "./runtime.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, "..", "..", "..");
const fixtureDir = join(
  repoRoot,
  "crates",
  "vize_atelier_jsx",
  "tests",
  "vue_jsx_vapor_compat",
  "fixtures",
);
const recordedCorpus = JSON.parse(readFileSync(join(fixtureDir, "corpus.json"), "utf8"));
const verdicts = JSON.parse(readFileSync(join(fixtureDir, "verdicts.json"), "utf8"));
const inventory = readFileSync(
  join(repoRoot, "crates", "vize_atelier_jsx", "tests", "VUE_JSX_VAPOR_COMPAT_INVENTORY.md"),
  "utf8",
);
const babelCorpus = JSON.parse(
  readFileSync(
    join(
      repoRoot,
      "crates",
      "vize_atelier_jsx",
      "tests",
      "babel_compat",
      "fixtures",
      "corpus.json",
    ),
    "utf8",
  ),
);
const babelVerdicts = readFileSync(
  join(repoRoot, "crates", "vize_atelier_jsx", "tests", "babel_compat", "verdicts.rs"),
  "utf8",
);

function observeChildren(parent) {
  const children = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      if (!node.data) continue;
      if (typeof children.at(-1) === "string") children[children.length - 1] += node.data;
      else children.push(node.data);
      continue;
    }
    if (node.nodeType !== 1) continue;
    const attributes = Object.fromEntries(
      [...node.attributes]
        .map(({ name, value }) => [name, value])
        .sort(([left], [right]) => left.localeCompare(right)),
    );
    const element = { tag: node.localName, attributes, children: observeChildren(node) };
    if (node.localName === "input") {
      element.value = node.value;
      element.checked = node.checked;
    }
    children.push(element);
  }
  return children;
}

function materialize(value) {
  if (Array.isArray(value)) return value.map(materialize);
  if (!value || typeof value !== "object") return value;
  if (Object.hasOwn(value, "$jsxValue")) {
    assert.deepEqual(Object.keys(value), ["$jsxValue"]);
    switch (value.$jsxValue) {
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
        assert.fail(`unknown JSX value descriptor: ${value.$jsxValue}`);
    }
  }
  return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, materialize(item)]));
}

async function trace(code, functionName, spec, runtime = "vapor", authoredModule = false) {
  const initial = materialize(structuredClone(spec.initial));
  const state = spec.valueMatrix ? Vue.shallowReactive(initial) : Vue.reactive(initial);
  const reads = [];
  const observations = { propPublications: 0 };
  const render = linkedFunction(code, functionName, state, reads, observations);
  if (authoredModule) {
    assert.equal(runtime, "vapor");
    assert.equal(functionName, "App");
    assert.equal(typeof render, "object", `${spec.id}: actual authored component object`);
    assert.equal(render.__vapor, true, `${spec.id}: actual Vapor component role`);
    assert.equal(typeof render.setup, "function", `${spec.id}: authored setup closure`);
  }
  const component = authoredModule
    ? render
    : runtime === "vapor"
      ? Vue.defineVaporComponent({ setup: () => render({}) })
      : { setup: () => () => render({}, []) };
  const app = runtime === "vapor" ? Vue.createVaporApp(component) : Vue.createApp(component);
  if (runtime === "vapor" && spec.componentFixtures?.length) app.use(Vue.vaporInteropPlugin);
  for (const fixture of spec.componentFixtures ?? []) {
    assert.ok(Object.hasOwn(componentFixtures, fixture), `${spec.id}: known component fixture`);
    app.component(fixture, componentFixtures[fixture]);
  }
  const warnings = [];
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap();
  let nextIdentity = 0;
  const events = [];

  function snapshot() {
    assert.deepEqual(warnings, [], `${spec.id}: runtime diagnostics`);
    const elements = [...host.querySelectorAll("*")];
    for (const element of elements) {
      if (!identities.has(element)) identities.set(element, nextIdentity++);
    }
    return {
      dom: observeChildren(host),
      text: host.textContent,
      elementIdentities: elements.map((element) => identities.get(element)),
      warnings: [...warnings],
      events: structuredClone(events),
      ...(spec.expectedReads ? { reads: [...reads] } : {}),
      ...(spec.expectedPropPublications ? { propPublications: observations.propPublications } : {}),
    };
  }

  try {
    app.mount(host);
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const step of spec.steps) {
      if (Object.hasOwn(step, "patch")) {
        assert.deepEqual(Object.keys(step), ["patch"], `${spec.id}: update shape`);
        Object.assign(state, materialize(structuredClone(step.patch)));
      } else {
        assert.deepEqual(Object.keys(step), ["event"], `${spec.id}: event shape`);
        assert.deepEqual(
          Object.keys(step.event).sort(),
          step.event.assign ? ["assign", "selector", "type"] : ["selector", "type"],
          `${spec.id}: event descriptor`,
        );
        const target = host.querySelector(step.event.selector);
        assert.notEqual(target, null, `${spec.id}: event target ${step.event.selector}`);
        if (step.event.assign) {
          for (const [property, value] of Object.entries(step.event.assign)) {
            assert.ok(property in target, `${spec.id}: event target property ${property}`);
            target[property] = materialize(structuredClone(value));
          }
        }
        const event = new window.Event(step.event.type, { bubbles: true, cancelable: true });
        const dispatchResult = target.dispatchEvent(event);
        const receipt = {
          selector: step.event.selector,
          type: step.event.type,
          defaultPrevented: event.defaultPrevented,
          dispatchResult,
        };
        if (step.event.assign) receipt.assigned = structuredClone(step.event.assign);
        if (target instanceof window.HTMLInputElement) {
          receipt.input = { value: target.value, checked: target.checked };
        }
        events.push(receipt);
      }
      await Vue.nextTick();
      phases.push(snapshot());
    }
    app.unmount();
    await Vue.nextTick();
    assert.equal(host.childNodes.length, 0, `${spec.id}: complete unmount`);
    return phases;
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
  }
}

function vaporReferenceCode(spec) {
  const result = transformVueJsxVapor(spec.source, `/semantic-oracle/${spec.id}.jsx?vapor=true`);
  assert.equal(typeof result.code, "string", `${spec.id}: oracle output`);
  return result.code;
}

function babelReferenceCode(spec) {
  const result = transformSync(spec.source, {
    configFile: false,
    babelrc: false,
    filename: `/semantic-oracle/${spec.id}.jsx`,
    plugins: [babelJsx],
  });
  assert.equal(typeof result?.code, "string", `${spec.id}: Babel oracle output`);
  return result.code;
}

function assertInventory(corpus) {
  assert.deepEqual(installed, {
    packageVersion: corpus.oracle.packageVersion,
    vueVersion: corpus.oracle.vueVersion,
  });
  assert.equal(verdicts.oraclePackageVersion, installed.packageVersion);
  assert.equal(verdicts.vueVersion, installed.vueVersion);
  const ids = corpus.cases.map(({ id }) => id).sort();
  assert.deepEqual(Object.keys(verdicts.cases).sort(), ids);
  for (const id of ids) {
    assert.ok(["equivalent", "divergent"].includes(verdicts.cases[id].verdict), id);
    assert.equal(typeof verdicts.cases[id].basis, "string", id);
  }
  const babelIds = new Set(babelCorpus.cases.map(({ id }) => id));
  for (const spec of corpus.cases.filter(({ babelCorpusId }) => babelCorpusId)) {
    assert.ok(babelIds.has(spec.babelCorpusId), `${spec.id}: registered Babel corpus row`);
    assert.ok(
      babelVerdicts.includes(`("${spec.babelCorpusId}", Same)`),
      `${spec.id}: equivalent Babel verdict`,
    );
  }
  for (const spec of corpus.cases) {
    const divergent = verdicts.cases[spec.id].verdict === "divergent";
    assert.equal(
      "expectedReferenceElementIdentities" in spec,
      divergent,
      `${spec.id}: divergent rows pin reference identity`,
    );
  }
  const documented = [
    ...inventory.matchAll(/^\| `([^`]+)`\s+\| (?:equivalent|divergent|deferred)\s+\|/gmu),
  ]
    .map((match) => match[1])
    .sort();
  assert.deepEqual(documented, ids);
}

function assertBehavior(spec, phases, expectedElementIdentities = spec.expectedElementIdentities) {
  assert.equal(phases.length, spec.steps.length + 1, `${spec.id}: initial render plus updates`);
  assert.deepEqual(
    phases.map(({ text }) => text),
    spec.expectedText,
    `${spec.id}: visible update trace`,
  );
  assert.deepEqual(
    phases.map(({ elementIdentities }) => elementIdentities),
    expectedElementIdentities,
    `${spec.id}: element update identity trace`,
  );
  if (spec.expectedDom) {
    assert.deepEqual(
      phases.map(({ dom }) => dom),
      spec.expectedDom,
      `${spec.id}: complete DOM trace`,
    );
  }
  if (spec.expectedReads) {
    assert.deepEqual(
      phases.map(({ reads }) => reads),
      spec.expectedReads,
      `${spec.id}: source evaluation order and count`,
    );
  }
  if (spec.expectedPropPublications) {
    assert.deepEqual(
      phases.map(({ propPublications }) => propPublications),
      spec.expectedPropPublications,
      `${spec.id}: one prop publication per effect`,
    );
  }
  if (spec.expectedEvents) {
    assert.deepEqual(
      phases.map(({ events }) => events),
      spec.expectedEvents,
      `${spec.id}: dispatched event trace`,
    );
  } else {
    assert(
      phases.every(({ events }) => events.length === 0),
      `${spec.id}: unexpected events`,
    );
  }
}

function semanticTrace(phases) {
  return phases.map(({ dom, text, warnings, events }) => ({ dom, text, warnings, events }));
}

assertInventory(recordedCorpus);

// Keep the unmodified emitted references and every runtime phase, including a
// partial failure, beside the separately captured complete compiler payload.
const evidence = { installed, cases: {}, failures: 0 };
process.on("uncaughtExceptionMonitor", (error) => {
  evidence.failures += 1;
  evidence.exception = { message: error.message, stack: error.stack };
});
if (process.env.VIZE_JSX_VAPOR_COMPAT_TRACE_CAPTURE) {
  process.on("exit", () => {
    writeFileSync(
      process.env.VIZE_JSX_VAPOR_COMPAT_TRACE_CAPTURE,
      `${JSON.stringify(evidence, null, 2)}\n`,
    );
  });
}

if (process.argv.includes("--reference")) {
  const reference = {};
  for (const spec of recordedCorpus.cases) {
    reference[spec.id] = {
      vapor: await trace(vaporReferenceCode(spec), "App", spec),
    };
    if (spec.babelCorpusId) {
      reference[spec.id].babel = await trace(babelReferenceCode(spec), "App", spec, "vdom");
    }
  }
  console.log(JSON.stringify(reference, null, 2));
} else {
  let payload = "";
  for await (const chunk of process.stdin) payload += chunk;
  const input = JSON.parse(payload);
  assert.deepEqual(input.corpus, recordedCorpus);
  assert.deepEqual(
    input.compiled.map(({ id, source }) => ({ id, source })),
    recordedCorpus.cases.map(({ id, source }) => ({ id, source })),
  );

  const receipt = {
    cases: recordedCorpus.cases.length,
    equivalent: 0,
    divergent: 0,
    babelCases: recordedCorpus.cases.filter(({ babelCorpusId }) => babelCorpusId).length,
    babelEquivalent: 0,
    failures: 0,
  };
  for (const [index, spec] of recordedCorpus.cases.entries()) {
    const compiled = input.compiled[index];
    const row = (evidence.cases[spec.id] = {});
    row.referenceCode = vaporReferenceCode(spec);
    const reference = (row.reference = await trace(row.referenceCode, "App", spec));
    const actual = (row.actual = await trace(
      compiled.code,
      compiled.functionName,
      spec,
      "vapor",
      true,
    ));
    if (verdicts.cases[spec.id].verdict === "equivalent") {
      assert.deepEqual(actual, reference, spec.id);
      assertBehavior(spec, actual);
      receipt.equivalent += 1;
    } else {
      assert.deepEqual(semanticTrace(actual), semanticTrace(reference), `${spec.id}: semantics`);
      assert.notDeepEqual(actual, reference, `${spec.id}: recorded divergence remains observable`);
      assertBehavior(spec, actual);
      assertBehavior(spec, reference, spec.expectedReferenceElementIdentities);
      receipt.divergent += 1;
    }
    if (spec.babelCorpusId) {
      assert.deepEqual(Object.keys(compiled.vdom), ["code", "functionName"]);
      row.babelCode = babelReferenceCode(spec);
      const babel = (row.babel = await trace(row.babelCode, "App", spec, "vdom"));
      const vdom = (row.vdom = await trace(
        compiled.vdom.code,
        compiled.vdom.functionName,
        spec,
        "vdom",
      ));
      assert.deepEqual(vdom, babel, `${spec.id}: VDOM/Babel`);
      assert.deepEqual(vdom, actual, `${spec.id}: VDOM/Vapor`);
      assertBehavior(spec, vdom);
      receipt.babelEquivalent += 1;
    }
  }
  await window.happyDOM.close();
  console.log(JSON.stringify(receipt));
}
