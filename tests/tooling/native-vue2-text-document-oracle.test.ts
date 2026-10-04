import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { compiler, plain } from "./support/vue2-pinned-oracle.ts";
import { loadRuntime } from "./support/vue2-pinned-runtime.ts";

type Call = { name: string; args: unknown[] };
type Case = {
  id: string;
  source: string;
  width: number;
  lineEnding: string;
  expected: string;
  beforeRender: string;
  afterRender: string;
  staticRenderFns: string[];
  errors: unknown[];
  vnode: { tag: string; children: Array<{ text: string }> };
  calls: Call[];
  lookups: string[];
};
const packet = JSON.parse(
  readFileSync(
    new URL(
      "../../crates/vize_glyph/tests/fixtures/native-vue2-text-doc-2.7.16.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  schema: string;
  version: number;
  vueVersion: string;
  nativeCapture: null;
  cases: Case[];
};
const compile = (source: string) =>
  compiler.compile(`<div>${source}</div>`) as ReturnType<typeof compiler.compile> & {
    staticRenderFns: string[];
  };

const execute = (source: string, environment: "test" | "production") => {
  const calls: Call[] = [];
  const lookups: string[] = [];
  const events: string[] = [];
  const implementations: Record<string, (...args: unknown[]) => unknown> = {
    add: (value, amount) => Number(value) + Number(amount),
    upper: (value) => `U:${String(value)}`,
    "upper ": (value) => `SP:${String(value)}`,
  };
  const Vue = loadRuntime(environment);
  const component = new Vue({
    data: () => ({ a: 6, b: 3, 日本: 4 }),
    filters: implementations,
    ...compiler.compileToFunctions(`<div>${source}</div>`),
  }) as InstanceType<typeof Vue> & { $options: { filters: Record<string, unknown> } };
  // Install observation only after Vue's real options merge; resolver and _f
  // remain the original runtime, including exact own/camel/Pascal lookup.
  for (const [name, implementation] of Object.entries(implementations)) {
    Object.defineProperty(component.$options.filters, name, {
      configurable: true,
      get() {
        lookups.push(name);
        events.push(`lookup:${name}`);
        return (...args: unknown[]) => {
          calls.push({ name, args: plain(args) as unknown[] });
          events.push(`call:${name}`);
          return implementation(...args);
        };
      },
    });
  }
  for (const [name, value] of Object.entries({ a: 6, b: 3, 日本: 4 })) {
    Object.defineProperty(component, name, {
      configurable: true,
      get() {
        events.push(`read:${name}`);
        return value;
      },
    });
  }
  const vnode = component._render();
  return {
    vnode: {
      tag: vnode.tag,
      children: Array.from(vnode.children, (child) => ({ text: child.text })),
    },
    calls,
    lookups,
    events,
  };
};
const expectedEvents = (entry: Case) => {
  const family = entry.id.slice(0, entry.id.indexOf("-"));
  const reads =
    family === "literal"
      ? []
      : family === "unicode"
        ? ["read:日本"]
        : ["bare", "zero", "blank", "spaced"].includes(family)
          ? ["read:a"]
          : ["read:a", "read:b"];
  return [
    ...entry.lookups.map((name) => `lookup:${name}`),
    ...reads,
    ...entry.calls.map(({ name }) => `call:${name}`),
  ];
};

test("whole independently authored Vue2 compiler goldens preserve exact historical ABI", () => {
  assert.equal(packet.schema, "vize.native-vue2-text-document-reference");
  assert.equal(packet.version, 1);
  assert.equal(packet.vueVersion, "2.7.16");
  assert.equal(packet.nativeCapture, null, "source plan has no fabricated execution receipt");
  assert.equal(packet.cases.length, 48);
  assert.equal(new Set(packet.cases.map((entry) => entry.id)).size, 48);
  for (const entry of packet.cases) {
    for (const [source, render] of [
      [entry.source, entry.beforeRender],
      [entry.expected, entry.afterRender],
    ]) {
      const actual = compile(source);
      assert.deepEqual(
        {
          render: actual.render,
          staticRenderFns: plain(actual.staticRenderFns),
          errors: plain(actual.errors),
        },
        { render, staticRenderFns: entry.staticRenderFns, errors: entry.errors },
        entry.id,
      );
    }
    for (const environment of ["test", "production"] as const) {
      const expected = {
        vnode: entry.vnode,
        calls: entry.calls,
        lookups: entry.lookups,
        events: expectedEvents(entry),
      };
      assert.deepEqual(
        execute(entry.source, environment),
        expected,
        `${entry.id} original ${environment}`,
      );
      assert.deepEqual(
        execute(entry.expected, environment),
        expected,
        `${entry.id} golden ${environment}`,
      );
    }
  }
});

const path = process.env.VIZE_GLYPH_VUE2_TEXT_CAPTURE;
test(
  "actual original TextView Docs join whole compiler/runtime goldens",
  { skip: path ? false : "dedicated source/merge hook requires a fresh Rust capture" },
  () => {
    assert.ok(path);
    const actual = JSON.parse(readFileSync(path, "utf8")) as {
      schema: string;
      version: number;
      sourceHead: string | null;
      executionCommit: string | null;
      workflowRun: string | null;
      cases: Array<{
        id: string;
        source: string;
        width: number;
        lineEnding: string;
        printed: string;
      }>;
    };
    assert.equal(actual.schema, "vize.native-vue2-text-document-capture");
    assert.equal(actual.version, 1);
    assert.equal(actual.sourceHead, process.env.VIZE_GLYPH_VUE2_TEXT_SOURCE_HEAD ?? null);
    assert.equal(actual.executionCommit, process.env.GITHUB_SHA ?? null);
    assert.equal(actual.workflowRun, process.env.GITHUB_RUN_ID ?? null);
    assert.deepEqual(
      actual.cases.map((entry) => entry.id),
      packet.cases.map((entry) => entry.id),
    );
    for (const [index, native] of actual.cases.entries()) {
      const entry = packet.cases[index];
      assert.ok(entry);
      assert.deepEqual(native, {
        id: entry.id,
        source: entry.source,
        width: entry.width,
        lineEnding: entry.lineEnding,
        printed: entry.expected,
      });
      const compiled = compile(native.printed);
      assert.deepEqual(
        {
          render: compiled.render,
          staticRenderFns: plain(compiled.staticRenderFns),
          errors: plain(compiled.errors),
        },
        { render: entry.afterRender, staticRenderFns: entry.staticRenderFns, errors: entry.errors },
      );
      for (const environment of ["test", "production"] as const) {
        assert.deepEqual(execute(native.printed, environment), {
          vnode: entry.vnode,
          calls: entry.calls,
          lookups: entry.lookups,
          events: expectedEvents(entry),
        });
      }
    }
  },
);
