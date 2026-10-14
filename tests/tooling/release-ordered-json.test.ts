import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { parseJson, stringifyJson } from "../../tools/support/release/ordered_json.ts";
import { runReleaseCommand } from "./support/release-command.ts";

const roundTrip = (text: string, indent = 0) => {
  const value = parseJson(text);
  assert.notEqual(value, undefined, text);
  return stringifyJson(value!, indent);
};

test("objects keep source member order, including integer-like keys", () => {
  assert.equal(roundTrip('{"b":1,"2":2,"1":3,"b":4}'), '{"b":4,"2":2,"1":3}');
  assert.equal(roundTrip("{}", 2), "{}");
  assert.equal(roundTrip("[]", 2), "[]");
  assert.equal(
    roundTrip('{"a":[{"b":null}]}', 2),
    '{\n  "a": [\n    {\n      "b": null\n    }\n  ]\n}',
  );
});

test("numbers use shortest doubles but keep unsafe integers and overflow literally", () => {
  assert.equal(
    roundTrip("[1.0, 1.50, 1E2, -0, -0.0, 1e21, 1e20, 0.000001, 1e-7, 123e-20]"),
    "[1,1.5,100,0,0,1e+21,100000000000000000000,0.000001,1e-7,1.23e-18]",
  );
  assert.equal(
    roundTrip("[9007199254740991, 9007199254740993, -9007199254740993, 1e400, -1e400]"),
    "[9007199254740991,9007199254740993,-9007199254740993,1e400,-1e400]",
  );
});

test("strings escape only quotes, backslashes and control characters", () => {
  assert.equal(
    roundTrip('"\\u0001\\u001f \\/ \\u007f \\b\\f\\n\\r\\t \\"\\\\ é \\ud83d\\ude00"'),
    '"\\u0001\\u001f / \u007f \\b\\f\\n\\r\\t \\"\\\\ é 😀"',
  );
});

test("parsing is strict JSON with a nesting limit of 1024", () => {
  for (const invalid of [
    "﻿{}",
    "[1,]",
    '{"a":1,}',
    "01",
    "NaN",
    '"a\tb"',
    '"\\x"',
    "-",
    "1.",
    "{}\u0000",
  ]) {
    assert.equal(parseJson(invalid), undefined, JSON.stringify(invalid));
  }
  assert.equal(roundTrip(" \n{}\r\t"), "{}");
  assert.notEqual(parseJson("[".repeat(1024) + "]".repeat(1024)), undefined);
  assert.equal(parseJson("[".repeat(1025) + "]".repeat(1025)), undefined);
});

test("prepared manifests are written in the original byte format", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-ordered-json-"));
  try {
    fs.mkdirSync(path.join(root, "npm/native"), { recursive: true });
    fs.mkdirSync(path.join(root, "npm/pkg"));
    fs.writeFileSync(
      path.join(root, "pnpm-workspace.yaml"),
      'catalogs:\n  tools:\n    tinyglobby: "0.2.16"\n',
    );
    fs.writeFileSync(
      path.join(root, "npm/native/package.json"),
      '{"name":"@vizejs/native","version":"0.57.0"}',
    );
    const manifest = path.join(root, "npm/pkg/package.json");
    fs.writeFileSync(
      manifest,
      '{"name": "x", "version": "1.0.0", "2": "two", "1": "one",\n' +
        ' "numbers": [1.0, 1.50, 1E2, -0, 1e21, 0.000001, 1e-7, 9007199254740993, 1e400],\n' +
        ' "text": "\\u0001 \\/ \\u007f é 😀 \\"q\\" \\\\ \\t\\n\\r\\b\\f",\n' +
        ' "empty": {}, "list": [], "nested": {"a": [{"b": null, "c": true}]},\n' +
        ' "dependencies": {"@vizejs/native": "workspace:^", "tinyglobby": "catalog:tools", "dup": "1", "dup": "2"}}\n',
    );
    const result = runReleaseCommand("prepare_npm_publish_manifest", [path.dirname(manifest)]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, `Prepared npm publish manifest at ${manifest}\n`);
    assert.equal(
      fs.readFileSync(manifest, "utf8"),
      '{\n  "name": "x",\n  "version": "1.0.0",\n  "2": "two",\n  "1": "one",\n  "numbers": [\n    1,\n    1.5,\n    100,\n    0,\n    1e+21,\n    0.000001,\n    1e-7,\n    9007199254740993,\n    1e400\n  ],\n  "text": "\\u0001 / \u007f é 😀 \\"q\\" \\\\ \\t\\n\\r\\b\\f",\n  "empty": {},\n  "list": [],\n  "nested": {\n    "a": [\n      {\n        "b": null,\n        "c": true\n      }\n    ]\n  },\n  "dependencies": {\n    "@vizejs/native": "^0.57.0",\n    "tinyglobby": "0.2.16",\n    "dup": "2"\n  }\n}\n',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
