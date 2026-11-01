import assert from "node:assert/strict";
import { test } from "node:test";
import {
  baselineObserver,
  verifyPair,
} from "../../tools/benchmarks/scripts/package-shadow-work-admission.ts";

const original = [
  "mod install_roots;",
  "fn walk() {",
  "        let mut pending = keys.into_iter().collect::<Vec<_>>();",
  "        while let Some(key) = pending.pop() {",
  "            if !visited.insert(key.clone()) { continue; }",
  "        }",
  "}",
].join("\n");

test("baseline instrumentation has only three invertible test-only insertions", () => {
  const { observed, edits } = baselineObserver(original);
  assert.equal(edits.length, 3);
  let inverse = observed;
  for (const [before, after] of [...edits].reverse()) inverse = inverse.replace(after, before);
  assert.equal(inverse, original);
  assert.equal(
    observed,
    [
      "#[cfg(test)]",
      "mod dirty_propagation_tests;",
      "mod install_roots;",
      "fn walk() {",
      "        #[cfg(test)]",
      "        dirty_propagation_tests::record_call(!self.package_shadows_initialized);",
      "        let mut pending = keys.into_iter().collect::<Vec<_>>();",
      "        while let Some(key) = pending.pop() {",
      "            #[cfg(test)]",
      "            dirty_propagation_tests::record_pop();",
      "            if !visited.insert(key.clone()) { continue; }",
      "        }",
      "}",
    ].join("\n"),
  );
});

test("missing, duplicate and pre-observed source anchors refuse before a build", () => {
  for (const [before] of baselineObserver(original).edits) {
    assert.throws(() => baselineObserver(original.replace(before, "changed")));
    assert.throws(() => baselineObserver(original + "\n" + before));
  }
  assert.throws(() => baselineObserver(baselineObserver(original).observed));
});

const root = "/tmp/controlled-package-work/runtime/fixture";
function packet() {
  return {
    schema: "vize.controlled-package-shadow-state",
    version: 1,
    state: "cold",
    fixtureRoot: root,
    virtualRoot: "/tmp/controlled-package-work/runtime/cache/vize/canon/projects/exact",
    nativeCheckerExecuted: false,
    reporterProjectReproduced: false,
    snapshot: Object.fromEntries(
      [
        "routes",
        "files",
        "originals",
        "shadows",
        "manifests",
        "fileOwners",
        "manifestOwners",
        "sourceIndex",
        "sourceOwners",
        "sourcePaths",
        "rootOwners",
        "importerOwners",
        "manifestRoutes",
        "shadowSources",
        "shadowScopes",
        "artifacts",
        "reverse",
        "aliases",
      ].map((field) => [field, `complete ${field} body 雪🍣\r\n`]),
    ),
  };
}
const bytes = (value: unknown) => Buffer.from(JSON.stringify(value));

test("complete equal state bytes pass with exact root and closed field inventory", () => {
  const body = bytes(packet());
  verifyPair(body, body, "cold", root);
});

test("any changed code, map, owner, alias or opaque source bytes refuse", () => {
  const before = bytes(packet());
  for (const field of Object.keys(packet().snapshot)) {
    const changed = packet();
    changed.snapshot[field] += " one changed byte";
    assert.throws(() => verifyPair(before, bytes(changed), "cold", root), field);
  }
  assert.throws(() =>
    verifyPair(before, Buffer.from(before.toString().replace("\\r\\n", "\\n")), "cold", root),
  );
});

test("equal malformed or unknown packet inventories do not become accepted baselines", () => {
  for (const value of [
    { ...packet(), unknown: "new metadata" },
    { ...packet(), snapshot: { ...packet().snapshot, unknown: "new metadata" } },
    { ...packet(), snapshot: {} },
    { ...packet(), snapshot: { ...packet().snapshot, files: [] } },
    { ...packet(), nativeCheckerExecuted: true },
    { ...packet(), reporterProjectReproduced: true },
    { ...packet(), virtualRoot: "/tmp/unowned/shared-cache" },
  ]) {
    const body = bytes(value);
    assert.throws(() => verifyPair(body, body, "cold", root));
  }
  assert.throws(() => verifyPair(bytes(packet()), bytes(packet()), "warm-noop", root));
  assert.throws(() => verifyPair(bytes(packet()), bytes(packet()), "cold", root + "-different"));
  const invalid = Buffer.from("not JSON");
  assert.throws(() => verifyPair(invalid, invalid, "cold", root));
});
