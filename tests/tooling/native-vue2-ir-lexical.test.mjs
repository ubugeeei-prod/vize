import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { test } from "node:test";
import { compileFunction } from "node:vm";
import { vnode } from "./native-vue2-ir-vnode.mjs";

const capture = process.env.VIZE_VUE2_IR_CAPTURE;
if (process.env.VIZE_VUE2_IR_REQUIRED === "1") assert.ok(capture);
const packet = capture ? JSON.parse(readFileSync(capture, "utf8")) : null;
const baseline = process.env.VIZE_VUE2_IR_BASELINE === "1";
const results = [];
const values = {
  a: 6,
  b: 3,
  日本: 4,
  _vm: 101,
  _vm1: 401,
  render: 202,
  render1: 402,
  staticRenderFns: 303,
  staticRenderFns1: 403,
};

function errorRecord(error) {
  return Object.fromEntries(Reflect.ownKeys(error).map((key) => [key, String(error[key])]));
}

function host(Vue, row, template, events) {
  const state = { ...values };
  if (row.constructor)
    state._vm = function FixtureConstructor(input) {
      events.push(`construct:${input}`);
      this.input = input;
    };
  const component = new Vue({ ...template });
  const scope = {};
  for (const { name, role } of row.references) {
    if (role === "context") {
      Object.defineProperty(component, name, {
        configurable: true,
        get() {
          events.push(`context:${name}`);
          return state[name];
        },
      });
      Object.defineProperty(scope, name, {
        get() {
          return component[name];
        },
      });
    } else if (role === "global") {
      Object.defineProperty(scope, name, {
        get() {
          events.push(`global:${name}`);
          return state[name];
        },
      });
    } else {
      Object.defineProperty(scope, name, {
        get() {
          return state[name];
        },
      });
    }
  }
  return { component, scope, state, events };
}

// The oracle evaluates the authored expression in the same JS realm as Vue.
// Foreign-realm objects change Vue 2's _s/isPlainObject behavior. This fixture
// evaluation is never used by the production parser, resolver or emitter.
function oracle(row) {
  const evaluate = compileFunction(`with (scope) { return (${row.expression}); }`, ["scope"]);
  let instance;
  function render() {
    return this._c("div", undefined, [this._v(this._s(evaluate(instance.scope)))]);
  }
  render._withStripped = true;
  return {
    template: { render, staticRenderFns: [] },
    bind(value) {
      instance = value;
    },
  };
}

async function observe(Vue, renderer, row, native, linkedCode, hostName, index, actualEvents) {
  const referenceEvents = [];
  const reference = oracle(row);
  const expected = host(Vue, row, reference.template, referenceEvents);
  reference.bind(expected);
  const actual = host(Vue, row, native, actualEvents);
  const saved = new Map();
  const oldErrorHandler = Vue.config.errorHandler;
  const renderErrors = [];
  Vue.config.errorHandler = (error, _instance, info) =>
    renderErrors.push({ info, error: errorRecord(error) });
  try {
    for (const { name, role } of row.references)
      if (role === "global") {
        saved.set(name, Object.getOwnPropertyDescriptor(globalThis, name));
        Object.defineProperty(globalThis, name, {
          configurable: true,
          get() {
            actualEvents.push(`global:${name}`);
            return actual.state[name];
          },
        });
      }
    const phases = [];
    for (const phase of ["first", "updated", "ssr"]) {
      if (phase === "updated") actual.state.a = expected.state.a = 7;
      actualEvents.length = referenceEvents.length = renderErrors.length = 0;
      let actualValue, referenceValue;
      if (phase === "ssr") {
        actualValue = await renderer.renderToString(actual.component);
        referenceValue = await renderer.renderToString(expected.component);
      } else {
        actualValue = vnode(actual.component._render(), actual.component);
        referenceValue = vnode(expected.component._render(), expected.component);
      }
      phases.push({
        phase,
        native: actualValue,
        reference: referenceValue,
        nativeEvents: actualEvents.slice(),
        referenceEvents: referenceEvents.slice(),
        renderErrors: renderErrors.slice(),
      });
    }
    return {
      host: hostName,
      index,
      role: row.role,
      expression: row.expression,
      linkedCode,
      phases,
    };
  } finally {
    for (const [name, descriptor] of saved) {
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
      else delete globalThis[name];
    }
    Vue.config.errorHandler = oldErrorHandler;
  }
}

for (const [name, version] of Object.entries({ vue26: "2.6.14", vue27: "2.7.16" })) {
  await test(
    `actual Vue ${version} preserves caller-owned lexical and global identities`,
    { skip: !capture },
    async () => {
      assert.equal(packet.bindingRows.length, 44);
      const require = createRequire(
        join(process.env.VIZE_VUE2_IR_ORACLES_ROOT, name, "package.json"),
      );
      const Vue = require("vue");
      const renderer = require("vue-server-renderer").createRenderer();
      assert.equal(Vue.version, version);
      const observations = [];
      for (const [index, row] of packet.bindingRows.entries()) {
        assert.equal(row.originalAstIdentitiesPreserved, true);
        assert.deepEqual(JSON.parse(row.map).sourcesContent, [row.source]);
        const actualEvents = [];
        const local = Object.fromEntries(
          row.references
            .filter((ref) => ref.role === "local")
            .map((ref) => [ref.name, values[ref.name]]),
        );
        if (row.constructor && Object.hasOwn(local, "_vm"))
          local._vm = function FixtureConstructor(input) {
            actualEvents.push(`construct:${input}`);
            this.input = input;
          };
        // Genuine caller declarations are linked before the exact unchanged
        // emitted module. Global cases use actual globalThis properties below.
        globalThis.__vizeVue2LocalValues = local;
        const prelude = Object.keys(local)
          .map(
            (key) => `const ${key} = globalThis.__vizeVue2LocalValues[${JSON.stringify(key)}];\n`,
          )
          .join("");
        const linkedCode = prelude + row.code;
        let result;
        try {
          const native = await import(
            `data:text/javascript;base64,${Buffer.from(linkedCode).toString("base64")}#${name}-${index}`
          );
          assert.deepEqual(Object.keys(native).sort(), ["render", "staticRenderFns"]);
          assert.equal(typeof native.render, "function");
          assert.deepEqual(native.staticRenderFns, []);
          result = await observe(Vue, renderer, row, native, linkedCode, name, index, actualEvents);
        } catch (error) {
          result = {
            host: name,
            index,
            role: row.role,
            expression: row.expression,
            linkedCode,
            error: errorRecord(error),
          };
        } finally {
          delete globalThis.__vizeVue2LocalValues;
        }
        observations.push(result);
        results.push(result);
      }
      // Preserve all 44 complete observations even when the first case fails.
      writeFileSync(
        process.env.VIZE_VUE2_IR_LEXICAL_RESULTS,
        JSON.stringify(results, null, 2) + "\n",
      );
      const failing = observations.filter(
        (row) =>
          row.error ||
          row.phases.some(
            (phase) =>
              phase.renderErrors.length ||
              !isDeepStrictEqual(phase.native, phase.reference) ||
              !isDeepStrictEqual(phase.nativeEvents, phase.referenceEvents),
          ),
      );
      if (baseline) {
        const expectedFailures = ["global", "local", "mixed"].flatMap((role) =>
          packet.bindingRows
            .filter((row) => row.role === role)
            .slice(0, 9)
            .map((row) => `${role}:${row.expression}`),
        );
        assert.deepEqual(
          failing.map((row) => `${row.role}:${row.expression}`),
          expectedFailures,
          "baseline controls must pass and only the known admitted lexical captures may fail",
        );
        assert.ok(
          failing.some((row) => row.role === "global" && row.expression === "_vm"),
          "baseline must reproduce genuine global receiver capture",
        );
        assert.ok(
          failing.some((row) => row.role === "local" && row.expression === "render" && row.error),
          "baseline must reproduce caller declaration collision",
        );
        assert.ok(
          failing.some((row) => row.role === "global" && row.expression === "staticRenderFns"),
          "baseline must reproduce module declaration capture",
        );
      } else {
        assert.deepEqual(failing, [], `${name}: full lexical/runtime observations differ`);
      }
    },
  );
}
