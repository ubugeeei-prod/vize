import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { joinSHA } from "./no-mutating-component-join-physical.mjs";

export function executeJoinLiteralCaller(currentRoot, context, lawPath) {
  const declared = context.frame.laws.find((row) => row.path === lawPath);
  assert(declared, "only a complete declared original caller");
  const body = fs.readFileSync(path.join(context.root, lawPath));
  assert.equal(joinSHA(body), declared.sha256);
  const env = { ...process.env };
  for (const name of Object.keys(env))
    if (name.startsWith("GIT_") || name === "NODE_TEST_CONTEXT") delete env[name];
  const argv = ["--test", "--test-reporter=tap", lawPath];
  const result = spawnSync(process.execPath, argv, {
    cwd: context.root,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const directory = path.join(currentRoot, "target/differential/no-mutating-component-join");
  fs.mkdirSync(directory, { recursive: true });
  const packet = path.join(
    directory,
    context.frame.source + "-" + path.basename(lawPath) + ".json",
  );
  let retainedHistoricalLaw = null;
  const record = {
    retainedHistoricalLaw,
    literalSource: context.frame.source,
    literalTree: context.frame.tree,
    originalWholeCallerSHA256: declared.sha256,
    completePhysicalSourceFrameRows: context.frame.files.length,
    nodeExecutable: process.execPath,
    nodeVersion: process.version,
    argv,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout,
    stderr: result.stderr,
    stdoutSHA256: joinSHA(result.stdout ?? ""),
    stderrSHA256: joinSHA(result.stderr ?? ""),
    historicalCallerCases: declared.cases,
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
    currentOldMigrationMeasurement: null,
    currentOldSetupMeasurement: null,
  };
  fs.writeFileSync(packet, JSON.stringify(record, null, 2) + "\n");
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  for (const line of [
    "# tests " + declared.cases,
    "# pass " + declared.cases,
    "# fail 0",
    "# cancelled 0",
    "# skipped 0",
    "# todo 0",
  ])
    assert(result.stdout.split("\n").includes(line), result.stdout);
  if (
    context.frame.source === "49d95b8e93a9f5bcb73e0c1c307815d4b3a9d570" &&
    [
      "reserved-suppression-default-routes.test.mjs",
      "setup-export-default-reference.test.mjs",
      "warm-type-backed-default-migration-host.test.ts",
      "warm-type-backed-git-reader-nodeep.test.ts",
    ].includes(path.basename(lawPath))
  ) {
    const originalPath = path.join(
      context.root,
      "target/differential/no-mutating-current-callers",
      path.basename(lawPath) + ".json",
    );
    const originalBytes = fs.readFileSync(originalPath);
    const original = JSON.parse(originalBytes);
    const cases = {
      "reserved-suppression-default-routes.test.mjs": 1,
      "setup-export-default-reference.test.mjs": 7,
      "warm-type-backed-default-migration-host.test.ts": 7,
      "warm-type-backed-git-reader-nodeep.test.ts": 5,
    }[path.basename(lawPath)];
    assert.equal(original.source, "2489128d5f8fab1b7442a1e5c09b4e1701cbf185");
    assert.equal(original.completeSourceFrameRows, 425);
    assert.equal(original.status, 0);
    assert.equal(original.signal, null);
    assert.equal(original.error, null);
    assert.equal(original.current23, 23);
    assert.equal(original.currentOldMeasurement, null);
    assert.equal(original.currentOldSetup, null);
    for (const line of ["# tests " + cases, "# pass " + cases, "# fail 0", "# skipped 0"])
      assert(original.stdout.split("\n").includes(line), original.stdout);
    const retainedPacket = path.join(directory, "original20-" + path.basename(lawPath) + ".json");
    fs.writeFileSync(retainedPacket, originalBytes);
    retainedHistoricalLaw = {
      packet: retainedPacket,
      bytes: originalBytes.length,
      sha256: joinSHA(originalBytes),
      cases,
      source: original.source,
      sourceOnly: true,
    };
  }
  record.retainedHistoricalLaw = retainedHistoricalLaw;
  fs.writeFileSync(packet, JSON.stringify(record, null, 2) + "\n");
  return { ...record, packet, historicalProjection: true };
}
