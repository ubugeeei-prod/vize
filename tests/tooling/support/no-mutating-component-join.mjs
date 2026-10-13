import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { qualifyDefaultMigrationHost } from "../../performance/support/warm-type-backed-default-migration-host.ts";
import {
  gitReaderSetupChanges,
  gitReaderSetupPairs,
  qualifyGitReaderSetupHarness,
} from "../../performance/support/warm-type-backed-git-reader-setup.ts";
import { rawNoMutatingCallerDigest } from "./no-mutating-props-current-caller-frame.mjs";
import { authenticateJoinLiteralFrame } from "./no-mutating-component-join-frame.mjs";
import { executeJoinLiteralCaller } from "./no-mutating-component-join-runner.mjs";
import {
  assertJoinCommitted,
  joinPhysical,
  joinObjectID,
  joinSHA,
} from "./no-mutating-component-join-physical.mjs";

export const componentJoinDirectory =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/component-main-join";
export const componentJoinManifestSHA256 =
  "34687146620db1a72515c4efa4330744dc120a578c469aa9c18aa0da81dce8a5";
const expectedSources = [
  "49d95b8e93a9f5bcb73e0c1c307815d4b3a9d570",
  "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
];
const callerNames = [
  "tests/tooling/reserved-suppression-default-routes.test.mjs",
  "tests/tooling/setup-export-default-reference.test.mjs",
  "tests/tooling/warm-type-backed-default-migration-host.test.ts",
  "tests/tooling/warm-type-backed-git-reader-nodeep.test.ts",
  "tests/tooling/no-mutating-props-source-custody.test.mjs",
  "tests/tooling/no-mutating-props-test-correction-source-custody.test.mjs",
  "tests/tooling/no-mutating-props-module-layout-source-custody.test.mjs",
  "tests/tooling/no-mutating-props-current-callers-source.test.mjs",
  "tests/tooling/no-mutating-props-current-callers-ancestry.test.mjs",
  "tests/tooling/bare-reserved-default-route-reference.test.mjs",
  "tests/tooling/props-deep-import-source-custody.test.mjs",
  "tests/tooling/props-deep-trait-scope-source-custody.test.mjs",
  "tests/tooling/props-deep-destructure-reference.test.mjs",
  "tests/tooling/runtime-props-api-source-custody.test.mjs",
  "tests/tooling/reserved-prefix-api-order.test.mjs",
  "tests/tooling/component-value-retained-guards.test.ts",
  "tests/tooling/warm-type-backed-component-value-composition.test.ts",
  "tests/tooling/warm-type-backed-component-value-host.test.ts",
  "tests/tooling/warm-type-backed-ascii-symbol-host.test.ts",
];
export function composeNoMutatingComponentJoinCaller(law) {
  assert(callerNames.includes(law), "only a declared complete caller");
  return Buffer.from(
    'import path from "node:path";\nimport { fileURLToPath } from "node:url";\n' +
      'import { test } from "node:test";\n' +
      'import { runNoMutatingComponentJoinCaller } from "./support/no-mutating-component-join.mjs";\n\n' +
      'const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");\n' +
      'void test("complete merged source retains both literal NoMutating and component caller authorities", async () => {\n' +
      (law === "tests/tooling/reserved-prefix-api-order.test.mjs"
        ? "  await runNoMutatingComponentJoinCaller(root, " + JSON.stringify(law) + ");\n"
        : "  await runNoMutatingComponentJoinCaller(\n    root,\n    " +
          JSON.stringify(law) +
          ",\n  );\n") +
      "});\n",
  );
}

/** Every actual merged field is authenticated before either literal historical context. */
export function authenticateNoMutatingComponentJoinPhysical(root) {
  const filename = componentJoinDirectory + "/successor.json";
  const stat = fs.lstatSync(path.join(root, filename));
  assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644);
  assert.equal(
    fs.realpathSync(path.join(root, filename)),
    path.join(fs.realpathSync(root), filename),
  );
  const bytes = fs.readFileSync(path.join(root, filename));
  assert.equal(joinSHA(bytes), componentJoinManifestSHA256, "whole finite source join");
  const manifest = JSON.parse(bytes);
  assert.equal(manifest.schema, "vize.no-mutating-component.complete-source-join");
  assert.equal(manifest.version, 1);
  assert.deepEqual(manifest.sources, expectedSources);
  assert.deepEqual(manifest.laws, callerNames);
  assert.equal(manifest.sourceRegistry.length, 20);
  assert.equal(new Set(manifest.sourceRegistry.map((row) => row.path)).size, 20);
  assert.equal(
    manifest.sourceRegistry.filter((row) => row.kind === "historical-source-law").length,
    19,
  );
  assert.deepEqual(
    manifest.sourceRegistry
      .filter((row) => row.kind === "live-native-consumer")
      .map((row) => row.path),
    ["tests/tooling/lint-props-deep-destructure.test.mjs"],
  );
  assert.equal(manifest.completePureHistoricalCases, 126);
  const counts = [
    ["noMutating23", 23],
    ["component81", 81],
    ["retained13", 13],
    ["currentReader111", 111],
  ];
  for (const [name, size] of counts) {
    assert.equal(manifest.families[name].length, size);
    assert.equal(new Set(manifest.families[name]).size, size);
  }
  const union = new Set(Object.values(manifest.families).flat());
  assert.equal(union.size, 206);
  assert.equal(manifest.completeUnionSize, 206);
  assert.equal(
    manifest.externalMaintenanceLaw.path,
    "tests/tooling/no-mutating-component-join-maintenance.test.mjs",
  );
  joinPhysical(root, manifest.externalMaintenanceLaw);
  const rows = manifest.current;
  assert.equal(new Set(rows.map((row) => row.path)).size, rows.length);
  assert.deepEqual(
    rows.map((row) => row.path),
    rows.map((row) => row.path).toSorted(),
  );
  const bodies = new Map(rows.map((row) => [row.path, joinPhysical(root, row)]));
  for (const name of union) assert(bodies.has(name), "complete actual source union: " + name);
  for (const law of callerNames)
    assert.deepEqual(bodies.get(law), composeNoMutatingComponentJoinCaller(law));
  for (const row of manifest.immutableOriginalSourcePins)
    assert.deepEqual(joinPhysical(root, { ...row, mode: "100644" }), bodies.get(row.path));
  assert.equal(manifest.completeOriginalNoMutatingCases, 20);
  assert.equal(manifest.completeIncomingCallerCases, 20);
  assert.deepEqual(manifest.credit, {
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
    oldMigrationMeasurement: null,
    oldSetupMeasurement: null,
    currentOld111Receipt: null,
  });
  assert.deepEqual(
    manifest.literalFrames.map((frame) => frame.source),
    expectedSources,
  );
  for (const frame of manifest.literalFrames) joinPhysical(root, frame);
  return {
    manifest,
    bodies,
    completeCurrentUnionBodies: 206,
    completeCurrentClosedBodies: rows.length,
    completeAncillarySourceControlBodies: 1,
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
    currentOld111Receipt: null,
    oldMigrationMeasurement: null,
    oldSetupMeasurement: null,
  };
}

/** The current join is new source-only authority, separately from both preserved old models. */
export function authenticateNoMutatingComponentJoin(root) {
  const current = authenticateNoMutatingComponentJoinPhysical(root);
  const frames = current.manifest.literalFrames.map((row) =>
    authenticateJoinLiteralFrame(root, row),
  );
  const migrationName =
    "tests/performance/support/warm-type-backed-default-migration-manifest.json";
  const migration = JSON.parse(current.bodies.get(migrationName));
  const migrationFiles = migration.files.map(([name]) => name);
  const measured = (side, name) => {
    assert(side === "before" || side === "after");
    const row = migration.files.find(([file]) => file === name);
    const preserved = migration.preservedBodies.find(([file]) => file === name);
    assert(row || preserved, "unknown old migration source");
    if (side === "before") return row ? row[1] : preserved[1];
    assert(current.bodies.has(name), "complete actual migration counterpart");
    return joinSHA(current.bodies.get(name));
  };
  assert.equal(
    qualifyDefaultMigrationHost(migrationFiles, new Set(), measured),
    null,
    "current joint must not inherit the old400 measurement",
  );
  const setup = (side, name) => {
    assert(side === "before" || side === "after");
    assert(
      gitReaderSetupPairs.some(([file]) => file === name),
      "unknown old setup source",
    );
    if (side === "before")
      return rawNoMutatingCallerDigest(
        frames[0].root,
        "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f",
        name,
      );
    assert(current.bodies.has(name), "complete actual setup counterpart");
    return joinSHA(current.bodies.get(name));
  };
  assert.equal(
    qualifyGitReaderSetupHarness(gitReaderSetupChanges, new Set(), setup),
    null,
    "current joint must not inherit the old setup measurement",
  );
  return { ...current, frames };
}

export function authenticateNoMutatingComponentJoinCommitted(root) {
  const current = authenticateNoMutatingComponentJoinPhysical(root);
  const committedRows = [
    ...current.manifest.current,
    ...current.manifest.literalFrames,
    current.manifest.externalMaintenanceLaw,
  ];
  for (const name of [
    componentJoinDirectory + "/successor.json",
    "tests/tooling/support/no-mutating-component-join.mjs",
    "tests/tooling/no-mutating-component-join-source.test.mjs",
  ]) {
    const body = fs.readFileSync(path.join(root, name));
    const row = {
      path: name,
      mode: "100644",
      bytes: body.length,
      sha256: joinSHA(body),
      gitBlob: joinObjectID("blob", body),
    };
    joinPhysical(root, row);
    committedRows.push(row);
  }
  committedRows.sort((left, right) =>
    left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
  );
  return assertJoinCommitted(root, committedRows);
}

export function runNoMutatingComponentJoinCaller(root, law) {
  assert(callerNames.includes(law), "unknown source-only joint caller");
  const current = authenticateNoMutatingComponentJoin(root);
  const committedHead = authenticateNoMutatingComponentJoinCommitted(root);
  const registry = current.manifest.sourceRegistry.find((row) => row.path === law);
  assert(registry && registry.kind === "historical-source-law");
  const selected = current.frames.filter((frame) =>
    registry.declaredSources.includes(frame.frame.source),
  );
  assert.equal(selected.length, registry.declaredSources.length);
  for (const frame of selected) assert(frame.frame.laws.some((row) => row.path === law));
  const historical = selected.map((frame) => executeJoinLiteralCaller(root, frame, law));
  return Promise.resolve({
    committedHead,
    currentUnion: current.completeCurrentUnionBodies,
    currentClosedBodies: current.completeCurrentClosedBodies,
    historical,
    historicalProjection: true,
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
    oldMigrationMeasurement: null,
    oldSetupMeasurement: null,
    currentOld111Receipt: null,
  });
}
