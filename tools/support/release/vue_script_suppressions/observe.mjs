import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "../typed_defaults/custody.mjs";
import { inputObservation } from "../typed_defaults/packet.mjs";

export const errorRecord = (error) =>
  error
    ? {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      }
    : null;

export function verifyPacket(sourceCase, packet) {
  assert.equal(packet.error, null);
  assert.equal(packet.rawCaptureComplete, true, "truncated or interrupted capture cannot qualify");
  assert.equal(packet.signal, null);
  assert.equal(packet.stderrBase64, "");
  const actual = JSON.parse(
    new TextDecoder("utf-8", { fatal: true }).decode(Buffer.from(packet.stdoutBase64, "base64")),
  );
  assert.equal(actual.length, 1);
  assert.equal(actual[0].file, sourceCase.filename);
  // This named derived law is exactly the frozen real-Linter contract.
  // Keep the entire ordered native packet; do not filter foreign findings or
  // grant a diagnostic-message/range oracle that those tests never established.
  assert.deepEqual(
    actual[0].messages.map(({ ruleId }) => ruleId).sort((a, b) => a.localeCompare(b)),
    sourceCase.expectedRuleIds,
    sourceCase.id + ": every finding, including every unrelated enabled rule",
  );
  assert.ok(actual[0].messages.every(({ severity }) => severity === 2));
  assert.equal(actual[0].errorCount, sourceCase.expectedRuleIds.length);
  assert.equal(actual[0].warningCount, 0);
  assert.equal(packet.status, sourceCase.expectedRuleIds.length ? 1 : 0);
  for (const input of packet.inputs) {
    assert.equal(input.readError, null, input.name);
    assert.equal(input.sha256, input.afterSha256, input.name);
    assert.equal(input.text, input.afterText, input.name);
  }
  return actual;
}

export function observeAll(fixture, identity, consumer, output, assertInstalled) {
  const observations = [];
  fs.mkdirSync(path.join(output, "packets"));
  for (const sourceCase of fixture.cases) {
    const cwd = path.join(consumer, "cases", sourceCase.id);
    fs.mkdirSync(cwd, { recursive: true });
    const inputs = [
      [sourceCase.filename, sourceCase.source],
      ["vize.config.json", sourceCase.config],
    ];
    for (const [name, text] of inputs) fs.writeFileSync(path.join(cwd, name), text, { flag: "wx" });
    const argv = [
      identity.launcher,
      "lint",
      sourceCase.filename,
      "--config",
      "vize.config.json",
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      "none",
    ];
    assertInstalled();
    const run = spawnSync(process.execPath, argv, {
      cwd,
      env: { ...process.env, NO_COLOR: "1" },
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const stdout = run.stdout ?? Buffer.alloc(0),
      stderr = run.stderr ?? Buffer.alloc(0);
    const packet = {
      id: sourceCase.id,
      sourceTest: sourceCase.sourceTest,
      installed: identity,
      executable: process.execPath,
      argv,
      cwd,
      inputs: inputs.map(([name, text]) => ({
        name,
        ...inputObservation(path.join(cwd, name), text),
      })),
      status: run.status,
      signal: run.signal,
      error: errorRecord(run.error),
      captureLimitBytes: 8 * 1024 * 1024,
      rawCaptureComplete: run.error === undefined,
      stdoutBase64: stdout.toString("base64"),
      stderrBase64: stderr.toString("base64"),
      stdoutSha256: sha256(stdout),
      stderrSha256: sha256(stderr),
    };
    // Save whole process evidence even when parsing or a contract fails.
    const processFile = path.join(output, "packets", sourceCase.id + ".process.json");
    fs.writeFileSync(processFile, JSON.stringify(packet, null, 2) + "\n", { flag: "wx" });
    const observation = {
      process: path.basename(processFile),
      processSha256: sha256(fs.readFileSync(processFile)),
    };
    try {
      observation.actual = verifyPacket(sourceCase, packet);
      assertInstalled();
      observation.passed = true;
    } catch (error) {
      observation.passed = false;
      observation.failure = errorRecord(error);
    }
    const filename = path.join(output, "packets", sourceCase.id + ".observation.json");
    fs.writeFileSync(filename, JSON.stringify(observation, null, 2) + "\n", { flag: "wx" });
    observations.push({
      id: sourceCase.id,
      packet: "packets/" + path.basename(filename),
      passed: observation.passed,
      sha256: sha256(fs.readFileSync(filename)),
    });
  }
  fs.writeFileSync(
    path.join(output, "observations.json"),
    JSON.stringify(observations, null, 2) + "\n",
    { flag: "wx" },
  );
  return observations;
}
