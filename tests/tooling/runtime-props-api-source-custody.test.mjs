import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import {
  authenticateRuntimeApiSourceFamily,
  assertReservedApiOrderSource,
} from "./support/component-value-retained-guards.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (name) => fs.readFileSync(path.join(root, name));
const directory = "tests/_fixtures/differential/lint/runtime-prop-defaults-8564/source-custody";

await test("one complete runtime source family preserves original order and typed authorities", () => {
  const manifest = authenticateRuntimeApiSourceFamily(root);
  assert.equal(manifest.sources.length, 12);
  for (const source of manifest.changedCarriers) {
    assert.equal(assertReservedApiOrderSource(root, source), "runtime-props-successor");
    const before = manifest.sources.find(({ path }) => path === source.path).before;
    const historical = (name) =>
      name === source.path ? read(`${directory}/${before.archive}`) : read(name);
    assert.equal(assertReservedApiOrderSource(root, source, historical), "original");
    assert.throws(() => assertReservedApiOrderSource(root, { ...source, bytes: source.bytes + 1 }));
    assert.throws(() =>
      assertReservedApiOrderSource(root, { ...source, path: source.path + ".foreign" }),
    );
  }
});

await test("every whole source, archive and authority mutation refuses runtime admission", () => {
  const manifest = authenticateRuntimeApiSourceFamily(root);
  const names = [
    `${directory}/successor.json`,
    manifest.originalCustody.path,
    `${directory}/${manifest.originalLaw.path}`,
    ...manifest.sources.flatMap((source) => [
      source.path,
      `${directory}/${source.after.archive}`,
      ...(source.before.absent ? [] : [`${directory}/${source.before.archive}`]),
    ]),
  ];
  assert.equal(new Set(names).size, names.length);
  for (const name of names) {
    const corrupt = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateRuntimeApiSourceFamily(root, corrupt), name);
  }
  for (const source of manifest.sources) {
    const mixed = (name) =>
      name === source.path
        ? source.before.absent
          ? Buffer.alloc(0)
          : read(`${directory}/${source.before.archive}`)
        : read(name);
    if (source.changed)
      assert.throws(() => authenticateRuntimeApiSourceFamily(root, mixed), source.path);
  }
});

await test("initial full CLI packets and both source refusals remain immutable evidence", () => {
  const bytes = read(`${directory}/initial-qualification.json`);
  assert.equal(sha256(bytes), "60ba41dd815edac8d1e0103762741a8c028ba65dd121f2661f9dc7c5e307363c");
  const initial = JSON.parse(bytes);
  assert.equal(initial.head, "029b7facb57d56e0c94780a4bc1f0820ceef996b");
  assert.equal(initial.run, 38183136962);
  assert.equal(initial.packets, 79);
  const packets = [];
  for (const identity of initial.artifacts) {
    const packed = read(`${directory}/${identity.path}`);
    assert.equal(packed.length, identity.bytes);
    assert.equal(sha256(packed), identity.sha256);
    const raw = gunzipSync(packed);
    assert.equal(raw.length, identity.rawBytes);
    assert.equal(sha256(raw), identity.rawSha256);
    packets.push(raw);
  }
  const observed = JSON.parse(packets[0]);
  assert.equal(observed.build.expected.sourceRevision, initial.compiledSource);
  assert.equal(
    observed.build.expected.binarySha256,
    "4e83d4ac1889f5840aa6010ed793bcffc8f579fd11dffff7ebd8efaa889acaea",
  );
  assert.equal(
    sha256(packets[0]),
    "aaf279c2d3c3295915f6f2a747cfa7bf56b785240348955c2c7839a40f493dce",
  );
  assert.equal(observed.runs.length, 79);
  const suites = {};
  for (const run of observed.runs) {
    suites[run.suite] = (suites[run.suite] ?? 0) + 1;
    assert.equal(sha256(Buffer.from(run.source)), run.sourceSha256);
    assert.equal(run.status, run.expectedStatus);
    assert.equal(run.signal, null);
    assert.equal(run.error, null);
    assert.equal(run.stderr, "");
    assert.deepEqual(JSON.parse(run.stdout), run.actual);
    assert.deepEqual(run.actual, run.expected);
  }
  assert.deepEqual(suites, {
    "original-scoped": 18,
    "ownership-scoped": 12,
    "minimal-full-route": 49,
  });
  assert.match(
    packets[1].toString(),
    /AssertionError.*crates\/vize_patina\/src\/linter\/script_rules\.rs/,
  );
  assert.match(
    packets[2].toString(),
    /Generated rule reference is stale: .*script-no-required-prop-with-default\.md/,
  );
});
