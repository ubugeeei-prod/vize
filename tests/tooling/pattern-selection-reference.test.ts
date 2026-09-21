// The selection semantics of the patterned-template reference implementation
// (vuejs/core#15531, `packages/compiler-core/__tests__/patterns.spec.ts`), run
// through Vize's compiled render function instead of the reference's bare
// selector: every case is an arm whose index and bindings a rendered
// interpolation reports back.
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import {
  buildPatternComponents,
  installDom,
  type PatternBuild,
} from "./support/upstream/pattern-runtime.ts";

await installDom();
const { createApp } = await import("vue");

/** `[pattern, subject expression in state.mjs, matches]` */
const selection: Array<[string, string, boolean]> = [
  ["'ok'", "'ok'", true],
  ["'ok'", "'other'", false],
  ["42", "'42'", false],
  ["-0", "0", true],
  ["NaN", "NaN", true],
  ["NaN", "0", false],
  ["null", "undefined", false],
  ["undefined", "undefined", true],
  ["{ x: _ }", "{}", false],
  ["{ x: _ }", "{ x: undefined }", true],
  ["{}", "null", false],
  ["{ x: 1 }", "{ x: 1, y: 2 }", true],
  ["[]", "[]", true],
  ["[]", "[1]", false],
  ["[1]", "{ 0: 1, length: 1 }", false],
  ["[1]", "new Set([1])", false],
  ["[1, ...]", "[1]", true],
  ["[1, ...]", "[1, 2]", true],
  ["[1, ...]", "[]", false],
  ["[1 | 2, true]", "[2, true]", true],
  ["[1 | 2, true]", "[3, true]", false],
  ["{ x: { y: 2 } }", "{ x: null }", false],
];

/** Named scenarios: arms in order, each reporting `capture(name, index, ...bindings)`. */
const scenarios: Record<string, { subject: string; arms: Array<[string, string[]]> }> = {
  order: {
    subject: "get()",
    arms: [
      ["values.first", []],
      ["{ kind: 'ok', const data } if (guard(data))", ["data"]],
      ["{ const data }", ["data"]],
      ["values.never", []],
    ],
  },
  objectRest: {
    subject: "subjects.objectRest",
    arms: [["{ kind: 'ok', value: _, ...const rest } as whole", ["rest", "whole"]]],
  },
  unboundRest: { subject: "subjects.unboundRest", arms: [["{ kind: 'ok', ... }", []]] },
  arrayRest: {
    subject: "subjects.frozen",
    arms: [["[const head, ...const tail]", ["head", "tail"]]],
  },
  arrayRestOne: { subject: "[1]", arms: [["[const head, ...const tail]", ["head", "tail"]]] },
  arrayRestEmpty: { subject: "[]", arms: [["[...const tail]", ["tail"]]] },
  arrayRestCopy: { subject: "subjects.frozen", arms: [["[...const tail]", ["tail"]]] },
  guardedArrayRest: {
    subject: "[1, 2]",
    arms: [
      ["[_, ...const tail] if (tail.length > 1)", ["tail"]],
      ["[const first, ...]", ["first"]],
      ["_", []],
    ],
  },
  guardedObjectRest: {
    subject: "{ x: 1, y: 2 }",
    arms: [
      ["{ x: _, ...const rest } if (rest.y === 2)", ["rest"]],
      ["_", []],
    ],
  },
  inherited: { subject: "subjects.inherited", arms: [["{ x: const value }", ["value"]]] },
  enclosingValue: {
    subject: "{ a: 1, b: 2 }",
    arms: [["{ a: outer, b: const outer }", ["outer"]]],
  },
  getterOnce: { subject: "subjects.getter", arms: [["{ kind: 'ok', ...const rest }", ["rest"]]] },
  protoKey: { subject: "subjects.proto", arms: [["{ value: _, ...const rest }", ["rest"]]] },
};

const attribute = (pattern: string) => pattern.replaceAll("&", "&amp;").replaceAll('"', "&quot;");

const template = [
  ...selection.map(
    ([pattern], index) =>
      `<template v-match="table[${index}]"><i v-when="${attribute(pattern)}">{{ capture('table${index}', 0) }}</i><i v-when="_">{{ capture('table${index}', -1) }}</i></template>`,
  ),
  ...Object.entries(scenarios).map(
    ([name, { subject, arms }]) =>
      `<template v-match="${attribute(subject)}">${arms
        .map(
          ([pattern, bindings], index) =>
            `<i v-when="${attribute(pattern)}">{{ capture('${name}', ${[index, ...bindings].join(", ")}) }}</i>`,
        )
        .join("")}</template>`,
  ),
].join("\n");

const state = `
export const table = [${selection.map(([, subject]) => subject).join(", ")}];
export const captured = new Map();
export const capture = (name, ...result) => (captured.set(name, result), "");
export const events = [];
const okay = { kind: "ok", data: 42 };
export const get = () => (events.push("subject"), okay);
export const values = {
  get first() { events.push("value"); return "other"; },
  get never() { throw new Error("unreachable"); },
};
export const guard = (data) => (events.push("guard:" + data), false);
export const outer = 1;
export const symbol = Symbol("metadata");
export const nested = { value: 1 };
const objectRest = Object.assign(Object.create({ inherited: 1 }), { kind: "ok", value: 2, nested, [symbol]: 3 });
Object.defineProperty(objectRest, "hidden", { value: 4 });
export const reads = { kind: 0 };
export const subjects = {
  objectRest,
  unboundRest: { kind: "ok", get unread() { throw new Error("unexpected read"); } },
  frozen: Object.freeze([1, 2, 3]),
  inherited: Object.create({ x: 1 }),
  getter: { get kind() { reads.kind++; return "ok"; }, value: 42 },
  proto: JSON.parse('{"__proto__":{"changed":true},"value":1}'),
};
`;

let build: PatternBuild;
let shared: Record<string, any>;

before(async () => {
  build = buildPatternComponents(
    {
      "Selection.vue": `<script setup>\nimport { table, capture, get, values, guard, outer, subjects } from "./state.mjs";\n</script>\n\n<template><div>${template}</div></template>\n`,
    },
    { "state.mjs": state },
    { backends: ["dom"] },
  );
  const { default: Selection } = await build.load<{ default: object }>("dom", "Selection.js");
  shared = await build.load("dom", "state.mjs");
  createApp(Selection).mount(document.createElement("div"));
});
after(() => build.dispose());

test("literal, value, object and array patterns select like the reference", () => {
  assert.deepEqual(
    selection.map((_, index) => shared.captured.get(`table${index}`)),
    selection.map(([, , matches]) => [matches ? 0 : -1]),
  );
});

test("the subject is read once, patterns lazily, and a guard after its bindings", () => {
  assert.deepEqual(shared.captured.get("order"), [2, 42]);
  assert.deepEqual(shared.events, ["subject", "value", "guard:42"]);
});

test("object rest copies own enumerable strings and symbols and excludes listed keys", () => {
  const [index, rest, whole] = shared.captured.get("objectRest");
  assert.equal(index, 0);
  assert.deepEqual(rest, { nested: shared.nested, [shared.symbol]: 3 });
  assert.equal(whole, shared.subjects.objectRest);
  assert.notEqual(rest, whole);
  assert.equal(rest.nested, shared.nested);
  assert.equal(Object.getPrototypeOf(rest), Object.prototype);
  assert.equal(whole.value, 2);
});

test("an unbound rest never reads the properties it omits", () => {
  assert.deepEqual(shared.captured.get("unboundRest"), [0]);
});

test("array rest is a fresh ordinary array, including an empty remainder", () => {
  assert.deepEqual(shared.captured.get("arrayRest"), [0, 1, [2, 3]]);
  assert.deepEqual(shared.captured.get("arrayRestOne"), [0, 1, []]);
  assert.deepEqual(shared.captured.get("arrayRestEmpty"), [0, []]);
  const [, copy] = shared.captured.get("arrayRestCopy");
  assert.deepEqual(copy, [1, 2, 3]);
  assert.notEqual(copy, shared.subjects.frozen);
  assert.equal(Object.getPrototypeOf(copy), Array.prototype);
});

test("rest bindings reach the guard, and a failed guard continues with the next arm", () => {
  assert.deepEqual(shared.captured.get("guardedArrayRest"), [1, 1]);
  assert.deepEqual(shared.captured.get("guardedObjectRest"), [0, { y: 2 }]);
});

test("property presence follows `in`, so inherited properties match", () => {
  assert.deepEqual(shared.captured.get("inherited"), [0, 1]);
});

test("a value pattern reads the enclosing scope before an arm binding shadows it", () => {
  assert.deepEqual(shared.captured.get("enclosingValue"), [0, 2]);
});

test("rest is copied after the match without reading an excluded getter again", () => {
  assert.deepEqual(shared.captured.get("getterOnce"), [0, { value: 42 }]);
  assert.equal(shared.reads.kind, 1);
});

test("copying a __proto__ key defines an own property and keeps the prototype", () => {
  const [, rest] = shared.captured.get("protoKey");
  assert.equal(Object.getPrototypeOf(rest), Object.prototype);
  assert.equal(Object.prototype.hasOwnProperty.call(rest, "__proto__"), true);
  assert.equal(rest.changed, undefined);
});
