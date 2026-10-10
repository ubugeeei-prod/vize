import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import {
  buildCaptureIdentity,
  normalizeSnapshotIdentity,
  resolveSnapshotIdentity,
  validateCaptureIdentity,
} from "./snapshot-identity.ts";

void test("identities preserve literal portable names and reject ambiguous path forms", () => {
  for (const value of ["src/Button.art.vue", "日本語/ボタン.art.vue", "a%2Fb/ Button.art.vue"])
    assert.equal(normalizeSnapshotIdentity(value), value);
  for (const value of [
    "",
    "/Button.art.vue",
    "C:/Button.art.vue",
    "C:Button.art.vue",
    "a\\Button.art.vue",
    "./Button.art.vue",
    "../Button.art.vue",
    "a/../Button.art.vue",
    "a/./b",
    "a//b",
    "a/",
    "a\0b",
  ])
    assert.throws(() => normalizeSnapshotIdentity(value), /Invalid project-relative/);
});

void test("project-relative identity survives relocation and refuses outside-root sources", () => {
  for (const root of [path.resolve("/machine-a/project"), path.resolve("/machine-b/project")]) {
    assert.equal(
      resolveSnapshotIdentity(path.join(root, "src", "Button.art.vue"), root),
      "src/Button.art.vue",
    );
    assert.equal(resolveSnapshotIdentity("src/Button.art.vue", root), "src/Button.art.vue");
    assert.throws(
      () => resolveSnapshotIdentity(path.join(root, "..", "Outside.art.vue"), root),
      /Configure projectRoot/,
    );
  }
});

void test("hosted identities require exact own keys and ignore the local checkout", () => {
  const identities = { "/build-machine/src/Button.art.vue": "src/Button.art.vue" };
  assert.equal(
    resolveSnapshotIdentity("/build-machine/src/Button.art.vue", "/unrelated", identities),
    "src/Button.art.vue",
  );
  assert.throws(
    () => resolveSnapshotIdentity("/missing", "/unrelated", identities),
    /identity missing/,
  );
  assert.throws(() => resolveSnapshotIdentity("toString", "/unrelated", {}), /identity missing/);
  assert.throws(
    () => resolveSnapshotIdentity("/art", "/unrelated", { "/art": "../escape" }),
    /Invalid project-relative/,
  );
});

void test("capture identity includes the full viewport and roundtrips canonically", () => {
  const job = {
    art: { path: "src/Button.art.vue" },
    variantName: "Default",
    viewport: { width: 200, height: 100, name: "small" },
  };
  const expected = '[1,"src/Button.art.vue","Default","small",200,100,1]';
  assert.equal(buildCaptureIdentity(job), expected);
  assert.doesNotThrow(() => validateCaptureIdentity(expected));
  assert.notEqual(
    buildCaptureIdentity({ ...job, viewport: { ...job.viewport, width: 400 } }),
    expected,
  );
  assert.notEqual(
    buildCaptureIdentity({ ...job, viewport: { ...job.viewport, deviceScaleFactor: 2 } }),
    expected,
  );
  assert.notEqual(
    buildCaptureIdentity({ ...job, variantName: "" }),
    buildCaptureIdentity({ ...job, variantName: "unnamed" }),
  );
  for (const viewport of [
    { width: 0, height: 100 },
    { width: 200.5, height: 100 },
    { width: 200, height: 100, deviceScaleFactor: Infinity },
  ])
    assert.throws(() => buildCaptureIdentity({ ...job, viewport }), /Invalid snapshot capture/);
  for (const value of [
    "[]",
    expected.replace("[1,", "[2,"),
    ` ${expected}`,
    expected.replace(",200,", ",2e2,"),
    "not json",
  ])
    assert.throws(() => validateCaptureIdentity(value), /snapshot ownership/);
});
