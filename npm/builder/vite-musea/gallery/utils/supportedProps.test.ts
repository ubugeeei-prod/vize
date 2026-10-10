import assert from "node:assert/strict";
import test from "node:test";
import { supportedProps, unsupportedPropEdit } from "./supportedProps.ts";

const retained = JSON.parse(
  '{"label":"Initial","__proto__":"Saved raw value","constructor":"Constructor","hasOwnProperty":"Method"}',
) as Record<string, unknown>;
const unsupported = new Set(["__proto__"]);

void test("component capability excludes only unsupported applied keys and retains raw ownership", () => {
  assert.deepEqual(supportedProps(retained, unsupported), {
    label: "Initial",
    constructor: "Constructor",
    hasOwnProperty: "Method",
  });
  assert.deepEqual(Object.keys(retained), ["label", "__proto__", "constructor", "hasOwnProperty"]);
  assert.equal(Object.hasOwn(retained, "__proto__"), true);
  assert.equal(retained.__proto__, "Saved raw value");
  assert.equal(Object.getPrototypeOf(retained), Object.prototype);
  assert.deepEqual(supportedProps(retained, new Set()), retained);
  assert.equal(Object.hasOwn(supportedProps(retained, new Set()), "__proto__"), true);
});

void test("ordinary code edits retain unsupported raw values without blocking supported fields", () => {
  const edited = { ...retained, label: "Edited", ["scope.name"]: "Custom" };
  assert.equal(unsupportedPropEdit(retained, edited, unsupported), undefined);
  const removed = Object.fromEntries(
    Object.entries(edited).filter(([name]) => name !== "__proto__"),
  );
  assert.equal(unsupportedPropEdit(retained, removed, unsupported), undefined);
  assert.equal(
    unsupportedPropEdit(retained, { ...edited, ["__proto__"]: "Changed" }, new Set()),
    undefined,
  );
});

void test("unsupported additions and edits fail explicitly without changing either dictionary", () => {
  const attempted = { ...retained, label: "Attempt", ["__proto__"]: "Changed" };
  assert.equal(
    unsupportedPropEdit(retained, attempted, unsupported),
    "__proto__ cannot be applied by Vue. Keep its retained value or remove it.",
  );
  assert.equal(
    unsupportedPropEdit({}, attempted, unsupported),
    "__proto__ cannot be applied by Vue. Keep its retained value or remove it.",
  );
  assert.equal(retained.__proto__, "Saved raw value");
  assert.equal(attempted.__proto__, "Changed");
  assert.equal(Object.getPrototypeOf(attempted), Object.prototype);
});
