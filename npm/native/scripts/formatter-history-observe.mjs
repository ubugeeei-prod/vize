import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { nativePreparationIsActive } from "./test-preparation.mjs";
import {
  hash,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
  nativeHistoryBuildEnvironment,
} from "./formatter-history-build.mjs";

const nativeDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
assert(
  nativePreparationIsActive(nativeDir),
  "public formatter requires the live preparation owner",
);
const buildBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
const build = validateNativeHistoryBuild(nativeDir, JSON.parse(buildBytes));
const preparation = JSON.parse(
  fs.readFileSync(path.join(nativeDir, ".artifacts/native/js-test-preparation.json")),
);
assert.equal(preparation.sha256, build.frozen.sha256);
for (const key of ["head", "tree", "workingDiff"])
  assert.equal(preparation[key], build.source[key]);
const inputBytes = fs.readFileSync(0);
const input = JSON.parse(inputBytes);
assert.equal(typeof input.source, "string");
assert(input.options && typeof input.options === "object");
const native = createRequire(import.meta.url)(build.frozen.path);
assert.equal(typeof native.formatSfc, "function");
// This declared observer policy makes the whole genuine Error object stable.
// No caught stream, property or stack is stripped or rewritten.
Error.stackTraceLimit = 0;
let result = null;
let error = null;
try {
  result = native.formatSfc(input.source, input.options);
} catch (actual) {
  error = {
    name: actual.name,
    constructor: actual.constructor.name,
    ownProperties: Object.fromEntries(
      Object.getOwnPropertyNames(actual)
        .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))
        .map((key) => [key, actual[key] === undefined ? { type: "undefined" } : actual[key]]),
    ),
  };
}
process.stdout.write(
  `${JSON.stringify({
    schema: "vize.public-native-formatter-observation",
    version: 1,
    input,
    inputSha256: hash(inputBytes),
    preparation,
    buildReceiptSha256: hash(buildBytes),
    artifactSha256: build.frozen.sha256,
    observerSha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))),
    runtime: {
      node: process.version,
      nodeOptions: process.env.NODE_OPTIONS ?? null,
      cargoEnvironment: nativeHistoryBuildEnvironment(),
      errorStackTraceLimit: Error.stackTraceLimit,
    },
    result,
    error,
  })}\n`,
);
