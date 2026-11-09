import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import { dirname, join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { transformSync } from "@babel/core";
import babelJsx from "@vue/babel-plugin-jsx";
import { Vue, window, installed, transformVueJsxVapor } from "./runtime.ts";
import {
  surfaceFixtures,
  directiveFixtures,
  linkedFunction,
  materialize,
  observeChildren,
} from "./surface.ts";
import { assertInventory, expectation, traceFields } from "./inventory.ts";

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

async function trace(code, functionName, spec, runtime = "vapor", authoredModule = false) {
  const initial = materialize(structuredClone(spec.initial));
  const state = spec.valueMatrix ? Vue.shallowReactive(initial) : Vue.reactive(initial);
  const reads = [];
  const observations = { propPublications: 0 };
  // TSX rows keep erasable type syntax in both emitted modules, as a bundler would receive them.
  const executable = spec.lang === "tsx" ? stripTypeScriptTypes(code) : code;
  const render = linkedFunction(executable, functionName, state, reads, observations);
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
    assert.ok(Object.hasOwn(surfaceFixtures, fixture), `${spec.id}: known component fixture`);
    app.component(fixture, surfaceFixtures[fixture]);
  }
  // VDOM resolves directives by registered name; Vapor reads the in-scope binding.
  for (const [name, directive] of Object.entries(directiveFixtures)) app.directive(name, directive);
  const warnings = [];
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const identities = new WeakMap();
  let nextIdentity = 0;
  const events = [];

  function snapshot() {
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

  // An error escaping the runtime is an observation like an error-handler report.
  // Actions stay synchronous so receipts read the DOM right after them.
  const uncaught = (error) => warnings.push(`uncaught: ${String(error)}`);
  const observe = (action) => {
    try {
      action();
    } catch (error) {
      uncaught(error);
    }
  };
  try {
    observe(() => app.mount(host));
    await Vue.nextTick();
    const phases = [snapshot()];
    for (const step of spec.steps) {
      if (Object.hasOwn(step, "patch")) {
        assert.deepEqual(Object.keys(step), ["patch"], `${spec.id}: update shape`);
        observe(() => Object.assign(state, materialize(structuredClone(step.patch))));
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
        // Vue's VDOM invoker drops events stamped no later than its attachment;
        // a real interaction always follows the mount by at least one tick of the clock.
        await new Promise((resolve) => setTimeout(resolve, 2));
        const event = new window.Event(step.event.type, { bubbles: true, cancelable: true });
        let dispatchResult;
        observe(() => (dispatchResult = target.dispatchEvent(event)));
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
      await Vue.nextTick().catch(uncaught);
      phases.push(snapshot());
    }
    observe(() => app.unmount());
    await Vue.nextTick();
    if (host.childNodes.length) phases.at(-1).warnings.push("uncaught: incomplete unmount");
    return phases;
  } finally {
    host.remove();
  }
}

function vaporReferenceCode(spec) {
  const result = transformVueJsxVapor(
    spec.source,
    `/semantic-oracle/${spec.id}.${spec.lang ?? "jsx"}?vapor=true`,
  );
  assert.equal(typeof result.code, "string", `${spec.id}: oracle output`);
  return result.code;
}

// The linked Babel row's plugin options apply; TSX keeps its types for the linker to erase.
function babelReferenceCode(spec) {
  const row = babelCorpus.cases.find(({ id }) => id === spec.babelCorpusId);
  const result = transformSync(spec.source, {
    configFile: false,
    babelrc: false,
    filename: `/semantic-oracle/${spec.id}.${spec.lang ?? "jsx"}`,
    // syntax-jsx stands aside when TypeScript is enabled, so TSX names both.
    parserOpts: spec.lang === "tsx" ? { plugins: ["typescript", "jsx"] } : {},
    plugins: [[babelJsx, row.babelOptions ?? {}]],
  });
  assert.equal(typeof result?.code, "string", `${spec.id}: Babel oracle output`);
  return result.code;
}

function changedFields(left, right) {
  return traceFields.filter(
    (field) =>
      !isDeepStrictEqual(
        left.map((phase) => phase[field]),
        right.map((phase) => phase[field]),
      ),
  );
}

function assertBehavior(spec, phases, side = "") {
  const expectedElementIdentities = expectation(spec, side, "elementIdentities");
  const expectedText = expectation(spec, side, "text");
  const expectedWarnings = expectation(spec, side, "warnings");
  assert.deepEqual(
    phases.map(({ warnings }) => warnings),
    expectedWarnings ?? phases.map(() => []),
    `${spec.id}: runtime diagnostics`,
  );
  assert.equal(phases.length, spec.steps.length + 1, `${spec.id}: initial render plus updates`);
  assert.deepEqual(
    phases.map(({ text }) => text),
    expectedText,
    `${spec.id}: visible update trace`,
  );
  assert.deepEqual(
    phases.map(({ elementIdentities }) => elementIdentities),
    expectedElementIdentities,
    `${spec.id}: element update identity trace`,
  );
  if (expectation(spec, side, "dom")) {
    assert.deepEqual(
      phases.map(({ dom }) => dom),
      expectation(spec, side, "dom"),
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
  if (expectation(spec, side, "events")) {
    assert.deepEqual(
      phases.map(({ events }) => events),
      expectation(spec, side, "events"),
      `${spec.id}: dispatched event trace`,
    );
  } else {
    assert(
      phases.every(({ events }) => events.length === 0),
      `${spec.id}: unexpected events`,
    );
  }
}

assertInventory({
  corpus: recordedCorpus,
  verdicts,
  inventory,
  babelCorpus,
  babelVerdicts,
  coverage: JSON.parse(readFileSync(join(fixtureDir, "value-coverage.json"), "utf8")),
  installed,
});

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
    babelDivergent: 0,
    vaporRejected: 0,
    failures: 0,
  };
  for (const [index, spec] of recordedCorpus.cases.entries()) {
    const compiled = input.compiled[index];
    const row = (evidence.cases[spec.id] = {});
    const verdict = verdicts.cases[spec.id];
    receipt[verdict.verdict] += 1;
    // A pinned Vapor rejection leaves only the VDOM/Babel pair to compare.
    let actual = null;
    if (spec.vaporRejected) {
      assert.ok(spec.babelCorpusId && !verdict.differs, `${spec.id}: VDOM-only row`);
      assert.equal(compiled.code, null, `${spec.id}: rejected Vapor build`);
      receipt.vaporRejected += 1;
    } else {
      row.referenceCode = vaporReferenceCode(spec);
      const reference = (row.reference = await trace(row.referenceCode, "App", spec));
      actual = row.actual = await trace(compiled.code, compiled.functionName, spec, "vapor", true);
      const differs = changedFields(actual, reference);
      assert.deepEqual(differs, verdict.differs ?? [], `${spec.id}: Vapor/vue-jsx-vapor fields`);
      if (differs.length === 0) assert.deepEqual(actual, reference, spec.id);
      else assertBehavior(spec, reference, "Reference");
      assertBehavior(spec, actual);
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
      const vdomDiffers = changedFields(vdom, babel);
      assert.deepEqual(vdomDiffers, verdict.vdomDiffers ?? [], `${spec.id}: VDOM/Babel fields`);
      if (vdomDiffers.length === 0) assert.deepEqual(vdom, babel, `${spec.id}: VDOM/Babel`);
      else assertBehavior(spec, babel, "Babel");
      // Source maps route VDOM through the Relief codegen; both must agree.
      assert.deepEqual(Object.keys(compiled.vdomSourceMapped), ["code", "functionName"]);
      row.vdomSourceMapped = await trace(
        compiled.vdomSourceMapped.code,
        compiled.vdomSourceMapped.functionName,
        spec,
        "vdom",
      );
      assert.deepEqual(row.vdomSourceMapped, vdom, `${spec.id}: source-mapped VDOM`);
      const conflicts = verdict.referenceConflicts ?? [];
      if (actual && verdict.verdict === "equivalent") {
        assert.deepEqual(
          vdom.flatMap((phase, index) => (isDeepStrictEqual(phase, actual[index]) ? [] : [index])),
          conflicts,
          `${spec.id}: VDOM/Vapor outside recorded reference conflicts`,
        );
      }
      for (const phase of conflicts) {
        assert.notDeepEqual(
          babel[phase],
          row.reference[phase],
          `${spec.id}: reference conflict ${phase}`,
        );
      }
      assertBehavior(spec, vdom, "Vdom");
      receipt[vdomDiffers.length ? "babelDivergent" : "babelEquivalent"] += 1;
    }
  }
  await window.happyDOM.close();
  console.log(JSON.stringify(receipt));
}
