import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { joinProjectJson } from "./project-json.ts";

void test("diagnostic array boundary whitespace stays byte-exact, including empty arrays", () => {
  const original = '{"diagnostics":[ \n {"message":"core"} \n ]}';
  const bridge = '{"diagnostics":[\t{"message":"template"}\t]}';
  const joined = joinProjectJson(original, bridge, {});
  assert.equal(joined, '{"diagnostics":[ \n {"message":"core"} \n ,\t{"message":"template"}\t]}');
  assert.deepEqual(JSON.parse(joined).diagnostics, [{ message: "core" }, { message: "template" }]);
  assert.equal(
    joinProjectJson('{"diagnostics":[ \n ]}', '{"diagnostics":[\t]}', {}),
    '{"diagnostics":[ \n \t]}',
  );
});

void test("the native reporter text remains exact when original diagnostics are empty", () => {
  const bridge = fs.readFileSync(
    new URL("../__snapshots__/json-scriptless-workaround-output.txt", import.meta.url),
    "utf8",
  );
  const start = bridge.indexOf("[{"),
    end = bridge.indexOf("],") + 1;
  assert.ok(start >= 0 && end > start);
  const source = bridge.slice(0, start) + "[]" + bridge.slice(end);
  assert.deepEqual(JSON.parse(source).diagnostics, []);
  assert.equal(
    joinProjectJson(source, bridge, {
      number_of_rules: 88,
      threads_count: 0,
      start_time: 0,
    }),
    bridge,
    "restore diagnostic JSON without changing any original report formatting",
  );
});

void test("escaped strings, nested duplicate names and opaque fields retain original text", () => {
  const diagnostic =
    '{"message": "escaped \\\"diagnostics\\\": [ ]", "labels": [], "opaque": {"diagnostics": [1, {"diagnostics": "]"}]}}';
  const original = `{\n  "diagnostics": [${diagnostic},${diagnostic}],\n  "number_of_rules": 2,\n  "opaque": { "number_of_rules": 999, "diagnostics": ["keep bytes"] }\n}\n`;
  const template = `{ "diagnostics": [{"message": "template", "labels": []}] }`;
  const joined = joinProjectJson(original, template, { number_of_rules: 3 });
  assert.deepEqual(JSON.parse(joined).diagnostics, [
    JSON.parse(diagnostic),
    JSON.parse(diagnostic),
    { message: "template", labels: [] },
  ]);
  assert.ok(joined.includes(`\n  "diagnostics": [${diagnostic},${diagnostic},`));
  assert.ok(joined.includes('"opaque": { "number_of_rules": 999, "diagnostics": ["keep bytes"] }'));
  assert.equal(joined.endsWith("\n}\n"), true);
  assert.throws(
    () => joinProjectJson('{"diagnostics":[],"diagnostics":[]}', template, {}),
    /Duplicate/u,
  );
  assert.throws(
    () => joinProjectJson(original, template, { foreign_count: 1 }),
    /Missing original/u,
  );
  assert.throws(
    () => joinProjectJson(original, template, { number_of_rules: Infinity }),
    /Invalid/u,
  );
});
