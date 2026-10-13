import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { ownedCorsaNativeReleaseInputs } from "./support/owned-corsa-native-release-inputs.mjs";

const pins = JSON.parse(
  fs.readFileSync(new URL("./fixtures/typechecker-native-release-inputs.json", import.meta.url)),
);
const workflow = parse(
  fs.readFileSync(
    new URL("../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
    "utf8",
  ),
);

void test("the exact ten package additions preserve every original ordered trigger and exclusion", () => {
  const actual = workflow.on.pull_request.paths;
  const positive = ownedCorsaNativeReleaseInputs(pins.positivePaths);
  assert.deepEqual(
    actual.filter((file) => !file.startsWith("!")),
    positive,
  );
  assert.equal(positive.length, 44);
  assert.deepEqual(
    actual.filter((file) => file.startsWith("!")),
    pins.excludedRoots.map((file) => "!" + file),
  );
  assert.deepEqual([...positive.slice(0, 31), ...positive.slice(-3)], pins.positivePaths);
  positive[0] = "foreign source";
  assert.equal(ownedCorsaNativeReleaseInputs(pins.positivePaths)[0], pins.positivePaths[0]);
});

void test("missing, reordered or unknown historical trigger cells refuse", () => {
  for (const foreign of [
    pins.positivePaths.slice(1),
    [...pins.positivePaths].reverse(),
    [...pins.positivePaths, "vendor/foreign/**"],
    pins.positivePaths.map((file, index) => (index === 0 ? "foreign/**" : file)),
  ])
    assert.throws(() => ownedCorsaNativeReleaseInputs(foreign));
});
