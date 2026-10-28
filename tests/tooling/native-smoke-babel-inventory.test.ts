import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import {
  admitBabelSmokeInventory,
  verifyBabelSmokeInventoryPair,
} from "./support/native-smoke-babel-inventory.ts";

const directory = new URL(
  "../_fixtures/differential/workflows/native-smoke-babel-inventory-10702/",
  import.meta.url,
);
const read = (name: string) => fs.readFileSync(new URL(name, directory), "utf8");
const before = read("before.yml"),
  after = read("after.yml");
const admission = (): unknown => JSON.parse(read("admission.json"));

test("both complete documents restore the same original provenance input", () => {
  verifyBabelSmokeInventoryPair(before, after, admission());
  assert.equal(admitBabelSmokeInventory(before), before);
  assert.equal(admitBabelSmokeInventory(after), before);
});

test("every unrelated byte remains protected on both sides", () => {
  for (const source of [before, after]) {
    for (const anchor of [
      "runs-on:",
      "timeout-minutes:",
      "node-version:",
      "workflow_dispatch:",
      "cargo build --profile ci -p vize",
      "--max 2.36",
      "--runtime-checks",
      "node --test tests/tooling/native-build-token.test.ts",
    ]) {
      assert.ok(source.includes(anchor), anchor);
      assert.throws(() => admitBabelSmokeInventory(source.replace(anchor, "counterfeit")));
    }
  }
});

test("an omitted, reordered or duplicated inventory entry cannot alter the registered pair", () => {
  const line = before.split("\n").find((row) => row.startsWith("  RELEASE_JS_PACKAGES: "));
  assert.ok(line);
  const packages = line.slice("  RELEASE_JS_PACKAGES: ".length).split(" ");
  assert.equal(packages.length, 15);
  for (const name of packages) {
    assert.throws(() =>
      verifyBabelSmokeInventoryPair(before, after.replace(" " + name, ""), admission()),
    );
  }
  assert.throws(() => verifyBabelSmokeInventoryPair(before, before, admission()));
  assert.throws(() =>
    verifyBabelSmokeInventoryPair(
      before,
      after.replace(" npm/babel", " npm/babel npm/babel"),
      admission(),
    ),
  );
  const first = packages[0],
    second = packages[1];
  assert.ok(first && second);
  assert.throws(() =>
    verifyBabelSmokeInventoryPair(
      before,
      after.replace(first + " " + second, second + " " + first),
      admission(),
    ),
  );
});

test("source provenance, original digest and archive membership cannot be omitted or forged", () => {
  const metadata: unknown = admission();
  assert.ok(metadata && typeof metadata === "object" && !Array.isArray(metadata));
  for (const key of [
    "beforeSource",
    "afterSource",
    "originalProvenanceSha256",
    "rawFiles",
    "qualification",
  ]) {
    const omitted = { ...metadata };
    Reflect.deleteProperty(omitted, key);
    assert.throws(() => verifyBabelSmokeInventoryPair(before, after, omitted));
    const forged = { ...metadata, [key]: "counterfeit" };
    assert.throws(() => verifyBabelSmokeInventoryPair(before, after, forged));
  }
});

function assertLiveInventory(workflow: string, release: string): void {
  assert.equal(workflow, after, "the active whole workflow is the registered successor");
  const expected = after.split("\n").find((row) => row.startsWith("  RELEASE_JS_PACKAGES: "));
  assert.ok(expected);
  assert.deepEqual(
    release.split("\n").filter((row) => row.startsWith("  RELEASE_JS_PACKAGES: ")),
    [expected],
    "the active release inventory retains exactly the same ordered sixteen packages",
  );
}

test("actual native-smoke and release files bind the registered successor", () => {
  assertLiveInventory(
    fs.readFileSync(new URL("../../.github/workflows/native-smoke.yml", import.meta.url), "utf8"),
    fs.readFileSync(new URL("../../.github/workflows/release.yml", import.meta.url), "utf8"),
  );
});

test("rolling back either or both active inventories cannot reuse historical admission", () => {
  const release = fs.readFileSync(
    new URL("../../.github/workflows/release.yml", import.meta.url),
    "utf8",
  );
  const oldLine = before.split("\n").find((row) => row.startsWith("  RELEASE_JS_PACKAGES: "));
  const newLine = after.split("\n").find((row) => row.startsWith("  RELEASE_JS_PACKAGES: "));
  assert.ok(oldLine && newLine);
  const rolledBackRelease = release.replace(newLine, oldLine);
  assert.throws(() => assertLiveInventory(before, rolledBackRelease));
  assert.throws(() => assertLiveInventory(after, rolledBackRelease));
});
