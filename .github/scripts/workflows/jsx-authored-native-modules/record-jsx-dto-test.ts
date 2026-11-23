import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileSha256 } from "../shared/files.ts";

import { readdirSync } from "node:fs";
const root = join(requiredEnv("RUNNER_TEMP"), "jsx-authored-native");
const stdout = textFile(join(root, "napi-dto.run.stdout"));
const passed = /test result: ok[.] 1 passed; 0 failed; 0 ignored;/u.test(stdout);
const names = textFile(join(root, "napi-dto.list.stdout"))
  .split(/\r?\n/u)
  .filter((name) => name.endsWith(": test"));
const filesSha256 = Object.fromEntries(
  readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.startsWith("napi-dto."))
    .map(({ name }) => [name, fileSha256(join(root, name))]),
);
writeJson(join(root, "napi-dto.receipt.json"), {
  sourceHead: textFile(join(root, "source.txt")).split("\n")[0],
  selected: names,
  selectedCount: names.length,
  exactOnePassed: passed,
  scope: "pure Rust adapter DTO and compile implementation; no Node ABI execution credit",
  filesSha256,
});
assert.ok(passed, stdout);
