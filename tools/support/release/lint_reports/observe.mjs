import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { assertLoadedBinary } from "../typed_defaults/distribution.mjs";
import { sha256 } from "../typed_defaults/custody.mjs";
import { inputObservation } from "../typed_defaults/packet.mjs";

export function verifyReportPacket(scenario, packet) {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.stderrBase64, "");
  const actual = JSON.parse(
    new TextDecoder("utf-8", { fatal: true }).decode(Buffer.from(packet.stdoutBase64, "base64")),
  );
  if (scenario.suite === "template") {
    assert.equal(packet.status, scenario.afterStatus);
    assert.deepEqual(
      actual,
      scenario.afterCli ?? scenario.afterFormatter,
      "whole frozen template formatter packet",
    );
  } else {
    // Preserve the original source campaign's finite count/error contract.
    // Full raw diagnostics are retained; this grants no complete warning oracle.
    const errors = scenario.expectedCliErrors ?? [];
    assert.equal(packet.status, errors.length ? 1 : 0);
    assert.equal(actual.length, 1);
    assert.equal(actual[0].file, scenario.filename);
    assert.deepEqual(
      actual[0].messages.filter((m) => m.severity === 2),
      errors,
    );
    assert.equal(actual[0].errorCount, errors.length);
    assert.equal(actual[0].warningCount, actual[0].messages.filter((m) => m.severity === 1).length);
    const findings = actual[0].messages.filter((m) => m.ruleId === "type/no-reactivity-loss");
    const snapshots = findings.filter((m) =>
      /Assigning 'held\.value'|Spreading 'h\.shape'/.test(m.message),
    );
    assert.equal(snapshots.length, scenario.expectedSnapshotAndSpreadWarnings);
    assert.equal(
      findings.length,
      scenario.expectedSnapshotAndSpreadWarnings +
        (scenario.id === "captured-outside-alias" ? 1 : 0),
    );
  }
  for (const input of packet.inputs) {
    assert.equal(input.readError, null);
    assert.equal(input.afterText, input.text);
    assert.equal(input.afterSha256, input.sha256);
  }
  return actual;
}

export function observeReports(fixtures, distribution, consumer, output, assertDependencies) {
  const observations = [];
  const packets = path.join(output, "packets");
  fs.mkdirSync(packets);
  for (const [index, scenario] of fixtures.vectors.entries()) {
    const directory = path.join(consumer, "cases", String(index).padStart(3, "0"));
    fs.mkdirSync(directory, { recursive: true });
    // Each private project resolves the genuine consumer node_modules naturally.
    // No override or source-built provider is passed to the actual public CLI.
    fs.writeFileSync(path.join(directory, "package.json"), '{"private":true}\n');
    const inputs = [
      [scenario.filename, scenario.source],
      ["vize.config.json", scenario.config],
      ["tsconfig.json", scenario.tsconfig],
    ];
    for (const [name, bytes] of inputs)
      fs.writeFileSync(path.join(directory, name), bytes, { flag: "wx" });
    const argv = [
      "lint",
      scenario.filename,
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      scenario.suite === "template" ? "full" : "none",
    ];
    assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
    assertDependencies();
    const run = spawnSync(distribution.member.binary, argv, {
      cwd: directory,
      env: { ...process.env, NO_COLOR: "1" },
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const stdout = run.stdout ?? Buffer.alloc(0),
      stderr = run.stderr ?? Buffer.alloc(0);
    const packet = {
      index,
      scenario,
      argv,
      cwd: directory,
      binary: distribution.member,
      inputs: inputs.map(([name, bytes]) => inputObservation(path.join(directory, name), bytes)),
      status: run.status,
      signal: run.signal,
      error: run.error ? { name: run.error.name, message: run.error.message } : null,
      stdoutBase64: stdout.toString("base64"),
      stderrBase64: stderr.toString("base64"),
      stdoutSha256: sha256(stdout),
      stderrSha256: sha256(stderr),
    };
    try {
      packet.actual = verifyReportPacket(scenario, packet);
      assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
      assertDependencies();
      packet.passed = true;
    } catch (error) {
      packet.passed = false;
      packet.failure = { name: error.name, message: error.message };
    }
    const name = `${String(index).padStart(3, "0")}-${scenario.suite}-${scenario.id}.json`;
    const bytes = Buffer.from(JSON.stringify(packet, null, 2) + "\n");
    fs.writeFileSync(path.join(packets, name), bytes, { flag: "wx" });
    observations.push({
      index,
      suite: scenario.suite,
      id: scenario.id,
      packet: `packets/${name}`,
      sha256: sha256(bytes),
      passed: packet.passed,
    });
    fs.writeFileSync(
      path.join(output, "observations.json"),
      JSON.stringify(observations, null, 2) + "\n",
    );
  }
  assert.equal(observations.length, 65);
  assert(
    observations.every((o) => o.passed),
    "installed report packets failed; inspect all retained packets",
  );
  return observations;
}
