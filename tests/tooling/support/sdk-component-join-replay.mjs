import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { admittedSdkHistoricalView, authenticateSdkComponentJoin } from "./sdk-component-join.mjs";
import { joinGit, joinSHA } from "./no-mutating-component-join-physical.mjs";
import { ensureHostedLiteralSource } from "../../performance/support/warm-type-backed-git-reader-literal-source.ts";

/** Provision only through the existing owned-Actions literal-object gate, before private replay. */
function provision(root, model, bodies) {
  const literals = new Set([model.beforeSource, "3dda8f9d71fbd7e701f0805187daff6191865190"]);
  const metadata = [];
  for (const role of ["sdk", "component"]) {
    const original = model.original[role];
    const name = original.module.replace(/\.ts$/u, ".json");
    const data = JSON.parse(bodies.get(name));
    metadata.push(data);
    for (const source of [
      original.source,
      data.originalSource,
      data.retainedSource,
      ...data.rawIntegrationPath.map(({ source }) => source),
    ])
      literals.add(source);
  }
  for (const source of literals) ensureHostedLiteralSource(root, source);
  const source = metadata[0];
  let current = source.retainedSource;
  for (let count = 0; count < 64; count++) {
    ensureHostedLiteralSource(root, current);
    if (current === source.originalSource) return;
    const body = joinGit(root, ["cat-file", "commit", current]);
    const parent = /^parent ([a-f0-9]{40})$/mu.exec(
      body.subarray(0, body.indexOf("\n\n")).toString(),
    );
    assert(parent, "complete retained raw first parent");
    current = parent[1];
  }
  assert.fail("retained first-parent provisioning exceeds unchanged64 limit");
}
/** The complete old233-row law runs only in an explicitly historical private SDK source view. */
export function runSdkJoinedHistoricalReaderLaws(root) {
  const receipt = authenticateSdkComponentJoin(root);
  const frame = admittedSdkHistoricalView(root, receipt);
  provision(root, frame.model, frame.oldBodies);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sdk-joined-historical-source-"));
  try {
    joinGit(directory, ["init", "--quiet"]);
    const common = joinGit(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(directory, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    for (const row of frame.oldRows) {
      const filename = path.join(directory, row.path);
      fs.mkdirSync(path.dirname(filename), { recursive: true });
      fs.writeFileSync(filename, frame.oldBodies.get(row.path), { mode: 0o644, flag: "wx" });
    }
    joinGit(directory, ["update-ref", "refs/heads/historical-sdk-source", frame.sdkSource]);
    joinGit(directory, ["symbolic-ref", "HEAD", "refs/heads/historical-sdk-source"]);
    const env = { ...process.env };
    for (const key of Object.keys(env))
      if (key.startsWith("GIT_") || key === "NODE_TEST_CONTEXT") delete env[key];
    const result = spawnSync(process.execPath, ["--test", "--test-reporter=tap", frame.driver], {
      cwd: directory,
      env,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      timeout: 240_000,
    });
    const packet = {
      currentSource: receipt.head,
      historicalSource: frame.sdkSource,
      historicalSourceBodies: frame.oldRows.length,
      historicalDriverRows: 233,
      driverSHA256: joinSHA(frame.oldBodies.get(frame.driver)),
      argv: [process.execPath, "--test", "--test-reporter=tap", frame.driver],
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      currentReceipt: receipt,
      sourceOnly: true,
      historicalProjection: true,
      runtimeCredit: false,
      nativeCredit: false,
      performanceCredit: false,
      currentOld111Receipt: null,
    };
    const capture = path.join(root, "target/differential/sdk-component-source-join");
    fs.mkdirSync(capture, { recursive: true });
    fs.writeFileSync(
      path.join(capture, "historical-reader-laws.json"),
      JSON.stringify(packet, null, 2) + "\n",
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, packet.stdout + packet.stderr);
    assert.equal(result.stderr, "");
    for (const line of ["# tests 5", "# pass 5", "# fail 0", "# skipped 0"])
      assert(packet.stdout.split("\n").includes(line), packet.stdout);
    const current = authenticateSdkComponentJoin(root);
    assert.equal(current.head, receipt.head);
    return { ...packet, passed: 5, skipped: 0 };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
