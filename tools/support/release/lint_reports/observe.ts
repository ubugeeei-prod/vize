import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { assertLoadedBinary } from "../typed_defaults/distribution.mjs";
import { sha256 } from "../typed_defaults/custody.mjs";
import { inputObservation } from "../typed_defaults/packet.mjs";
import { serializeFailure } from "../typed_defaults/observation.ts";
import type {
  BinaryMember,
  Distribution,
  InputObservation,
  Observation,
  PacketResult,
  SerializedFailure,
} from "../typed_defaults/observation.ts";
import { vueConfigFixtureSource } from "../../../../tests/tooling/support/vue-config-fixture.ts";

interface ReportScenarioSource {
  id: string;
  filename: string;
  source: string;
  config: string;
  tsconfig: string;
  [field: string]: unknown;
}

export interface TemplateScenario extends ReportScenarioSource {
  suite: "template";
  afterStatus: number;
  afterCli?: unknown;
  afterFormatter?: unknown;
}

export interface ComputedScenario extends ReportScenarioSource {
  suite: "computed";
  expectedCliErrors?: unknown[];
  expectedSnapshotAndSpreadWarnings: number;
}

export type ReportScenario = TemplateScenario | ComputedScenario;

export interface ReportVerificationPacket {
  error: SerializedFailure | null;
  signal: NodeJS.Signals | null;
  status: number | null;
  stderrBase64: string;
  stdoutBase64: string;
  inputs: InputObservation[];
}

interface ReportPacket extends ReportVerificationPacket, PacketResult {
  index: number;
  scenario: ReportScenario;
  argv: string[];
  cwd: string;
  binary: BinaryMember;
  stdoutSha256: string;
  stderrSha256: string;
}

interface ReportDiagnostic {
  severity: number;
  message: string;
  ruleId?: unknown;
  [field: string]: unknown;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isDiagnostic(value: unknown): value is ReportDiagnostic {
  return isRecord(value) && typeof value.severity === "number" && typeof value.message === "string";
}

export function verifyReportPacket(
  scenario: ReportScenario,
  packet: ReportVerificationPacket,
): unknown {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.stderrBase64, "");
  const actual: unknown = JSON.parse(
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
    assert(Array.isArray(actual), "whole CLI report must be an array");
    assert.equal(actual.length, 1);
    const report: unknown = actual[0];
    assert(isRecord(report), "whole CLI report must contain an object");
    assert.equal(report.file, scenario.filename);
    assert(Array.isArray(report.messages), "whole CLI report must retain its messages");
    const messages: unknown[] = report.messages;
    assert(messages.every(isDiagnostic), "whole CLI report contains an invalid diagnostic");
    assert.deepEqual(
      messages.filter((m) => m.severity === 2),
      errors,
    );
    assert.equal(report.errorCount, errors.length);
    assert.equal(report.warningCount, messages.filter((m) => m.severity === 1).length);
    const findings = messages.filter((m) => m.ruleId === "type/no-reactivity-loss");
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

export function observeReports(
  fixtures: { vectors: ReportScenario[] },
  distribution: Distribution,
  consumer: string,
  output: string,
  assertDependencies: () => void,
): Observation[] {
  const observations: Observation[] = [];
  const packets = path.join(output, "packets");
  fs.mkdirSync(packets);
  for (const [index, scenario] of fixtures.vectors.entries()) {
    const directory = path.join(consumer, "cases", String(index).padStart(3, "0"));
    fs.mkdirSync(directory, { recursive: true });
    // Each private project resolves the genuine consumer node_modules naturally.
    // No override or source-built provider is passed to the actual public CLI.
    fs.writeFileSync(path.join(directory, "package.json"), '{"private":true}\n');
    const inputs: [string, string][] = [
      [scenario.filename, scenario.source],
      // Preserve the frozen native document as evidence; execute its Vue carrier.
      ["vize.config.json.fixture", scenario.config],
      ["vite.config.mjs", vueConfigFixtureSource(scenario.config)],
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
    const packet: ReportPacket = {
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
      packet.failure = serializeFailure(error);
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
      passed: packet.passed === true,
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
