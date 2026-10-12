import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  retainedHostBeforeComponentValues,
  retainedPhysicalHostBeforeComponentValues,
  selectedComponentValueBodies,
} from "./support/component-value-host-custody.ts";
const root = path.resolve(import.meta.dirname, "../..");
const capsule = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/linter/component-import-value-host-before/host-composition.json",
    ),
    "utf8",
  ),
);
const files = [
  "tests/performance/support/warm-type-backed-host-qualifications.ts",
  "tests/performance/support/warm-type-backed-component-value-host.ts",
  "tests/performance/support/warm-type-backed-component-value-manifest.json",
];
const bodies = new Map(files.map((file) => [file, fs.readFileSync(path.join(root, file))]));
const read = (file: string) => bodies.get(file) ?? fs.readFileSync(path.join(root, file));
void test("the complete NoUnused source successor inverts to the retained original ASCII host without runtime credit", () => {
  assert.deepEqual(
    retainedPhysicalHostBeforeComponentValues(root),
    Buffer.from(capsule.rawBase64, "base64"),
  );
});
void test("unknown, missing, partial and historical mixed helper or whole source bodies refuse", () => {
  for (const file of files)
    for (const body of [
      null,
      Buffer.from("foreign source"),
      Buffer.concat([read(file)!, Buffer.from(" ")]),
    ]) {
      assert.throws(() =>
        retainedHostBeforeComponentValues((name) => (name === file ? body : read(name)), capsule),
      );
    }
  assert.throws(() =>
    retainedHostBeforeComponentValues(
      (file) => (file === files[0] ? Buffer.from(capsule.rawBase64, "base64") : read(file)),
      capsule,
    ),
  );
  for (const clause of [
    'import { qualifyComponentValueHost } from "./warm-type-backed-component-value-host.ts";\n',
    '  const componentValueHost =\n    enabled &&\n    digest("before", "tests/performance/support/warm-type-backed-component-value-host.ts") ===\n      null &&\n    digest("after", "tests/performance/support/warm-type-backed-component-value-host.ts") ===\n      "5e2f68b285f7a2de18f7d45f0a78fed29331c54cb761d3f59133d90a3d6306cf" &&\n    digest("before", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===\n      null &&\n    digest("after", "tests/performance/support/warm-type-backed-component-value-manifest.json") ===\n      "50259b48cc321fb1114816ae53ff50003aaeee5fd6f36c9b19ef4a0bc5d8768e"\n      ? qualifyComponentValueHost(production, alreadyQualified, digest)\n      : null;\n',
    "    ...(componentValueHost ? { componentValueHost } : {}),\n",
  ]) {
    assert.equal(read(files[0])!.toString("utf8").split(clause).length, 2);
    const partial = read(files[0])!.toString("utf8").replace(clause, "");
    assert.throws(() =>
      retainedHostBeforeComponentValues(
        (file) => (file === files[0] ? Buffer.from(partial) : read(file)),
        capsule,
      ),
    );
  }
});
void test("missing, foreign, changed and noncanonical complete original source metadata refuses", () => {
  for (const [key, value] of [
    ["source", "f".repeat(40)],
    ["mode", "100755"],
    ["gitBlob", "0".repeat(40)],
    ["byteLength", capsule.byteLength + 1],
    ["sha256", "f".repeat(64)],
    ["rawBase64", capsule.rawBase64 + "\n"],
    ["foreign", true],
  ]) {
    assert.throws(() =>
      retainedHostBeforeComponentValues(read, { ...capsule, [key as string]: value }),
    );
  }
  for (const key of Object.keys(capsule)) {
    const changed = { ...capsule };
    delete changed[key];
    assert.throws(() => retainedHostBeforeComponentValues(read, changed));
  }
});

void test("the actual host composer requires complete source pins and preserves disabled/null receipts", async () => {
  const { createHash } = await import("node:crypto");
  const { qualifyHostMoves } =
    await import("../performance/support/warm-type-backed-host-qualifications.ts");
  const { production, digest: selectedDigest } = selectedComponentValueBodies(read);
  const pins = new Map(
    files
      .slice(1)
      .map((file) => [
        file,
        { before: null, after: createHash("sha256").update(read(file)!).digest("hex") },
      ]),
  );
  const digest = (side: "before" | "after", file: string) =>
    pins.get(file)?.[side] ?? selectedDigest(side, file);
  const oldNull = { pathHostMove: null, timingHostMove: null, defaultMigrationHost: null };
  const result = qualifyHostMoves(production, new Set(), true, digest);
  assert.deepEqual(Object.keys(result), [...Object.keys(oldNull), "componentValueHost"]);
  assert.equal(result.componentValueHost?.completeBodyPairs, 15);
  assert.equal(result.componentValueHost?.supplementalBodyPairs, 52);
  assert.equal(result.componentValueHost?.preservedBodyPairs, 13);
  assert.equal(result.componentValueHost?.runtimeCredit, false);
  assert.equal(result.componentValueHost?.performanceCredit, false);
  assert.deepEqual(
    qualifyHostMoves(production, new Set(), false, () => {
      throw Error("disabled digest called");
    }),
    oldNull,
  );
  for (const file of files.slice(1))
    for (const side of ["before", "after"] as const)
      for (const changed of [null, "f".repeat(64)]) {
        if (changed === digest(side, file)) continue;
        assert.deepEqual(
          qualifyHostMoves(production, new Set(), true, (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          oldNull,
        );
      }
});

void test("every actual selected Rust/corpus/preserved body is required before historical projection", () => {
  const manifest = JSON.parse(read(files[2])!.toString("utf8")) as {
    files: Array<[string, unknown, unknown]>;
    supplementalBodies: Array<[string, unknown, unknown]>;
    preservedBodies: Array<[string, unknown]>;
  };
  const selected = [
    ...manifest.files,
    ...manifest.supplementalBodies,
    ...manifest.preservedBodies,
  ].map(([file]) => file);
  assert.equal(selected.length, 80);
  for (const target of selected)
    for (const changed of [null, Buffer.concat([read(target)!, Buffer.from(" ")])]) {
      assert.throws(
        () =>
          retainedHostBeforeComponentValues(
            (file) => (file === target ? changed : read(file)),
            capsule,
          ),
        "selected full source: " + target,
      );
    }
  for (const target of [
    "tests/_fixtures/differential/linter/component-import-value-host-before/entries.json",
    "tests/_fixtures/differential/linter/component-import-value-host-before/host-supplement-before.json",
  ]) {
    assert.throws(() =>
      retainedHostBeforeComponentValues((file) => (file === target ? null : read(file)), capsule),
    );
    assert.throws(() =>
      retainedHostBeforeComponentValues(
        (file) => (file === target ? Buffer.from("{}") : read(file)),
        capsule,
      ),
    );
  }
});
