import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import type Module from "node:module";
import { hash, integer, record, string } from "./model.ts";
const config = record(JSON.parse(string(process.env.VIZE_TYPE_ADMISSION_OBSERVER)));
const workspace = fs.realpathSync(string(config.workspace));
const native = record(config.native),
  target = string(native.path);
assert.equal(fs.realpathSync(target), target);
assert.ok(target.startsWith(workspace + path.sep));
assert.equal(fs.statSync(target).size, integer(native.bytes));
const prefix = string(config.prefix);
assert.ok(path.isAbsolute(prefix));
const journal = fs.openSync(prefix + "-" + process.pid + ".txt", "wx");
const event = (value: Record<string, unknown>) =>
  fs.writeSync(journal, JSON.stringify({ pid: process.pid, ...value }) + "\n");
event({ event: "initialized", workspace, node: process.execPath, argv: process.argv });
process.on("exit", (code) => event({ event: "exit", code }));
const require = createRequire(import.meta.url);
const original = require.extensions[".node"];
assert.ok(original);
require.extensions[".node"] = function (this: unknown, mod: Module, filename: string): unknown {
  const actual = fs.realpathSync(filename),
    expected = actual === target;
  const actualHash = hash(fs.readFileSync(actual));
  event({
    event: "attempt",
    path: actual,
    sha256: actualHash,
    expected,
    moduleFilename: mod.filename,
  });
  if (
    (/(?:vize[-_](?:native|vitrine)|@vizejs[/\\]native)/u.test(actual) && !expected) ||
    (expected && actualHash !== string(native.sha256))
  ) {
    event({ event: "rejected", path: actual });
    throw new Error("native load escaped the verified SDK");
  }
  let result: unknown;
  try {
    result = Reflect.apply(original, this, arguments);
  } catch (error) {
    event({ event: "failed", path: actual, error: String(error) });
    throw error;
  }
  const after = hash(fs.readFileSync(actual));
  assert.equal(after, actualHash, "loaded native image changed");
  event({
    event: "returned",
    path: actual,
    sha256: after,
    expected,
    nativeCachePaths: Object.keys(require.cache)
      .filter((name) => name.endsWith(".node"))
      .map((name) => fs.realpathSync(name)),
  });
  return result;
};
