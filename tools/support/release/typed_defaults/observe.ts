import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { assertLoadedBinary } from "./distribution.mjs";
import { sha256 } from "./custody.mjs";
import { inputObservation, verifyPacket } from "./packet.mjs";
import { serializeFailure } from "./observation.ts";
import type {
  BinaryMember,
  Distribution,
  InputObservation,
  Observation,
  PacketResult,
  SerializedFailure,
} from "./observation.ts";
import {
  vueConfigFixtureSource,
  writeVueConfigFixture,
} from "../../../../tests/tooling/support/vue-config-fixture.ts";

export interface TypedEntry {
  id: string;
  filename: string;
  source: string;
  [field: string]: unknown;
}

export interface TypedFinding {
  severity: string;
  [field: string]: unknown;
}

export interface TypedScenario {
  entry: TypedEntry;
  preset: string | null;
  config: string | null;
  findings: TypedFinding[];
  suite: string;
}

export interface TypedFixtures {
  vectors: TypedScenario[];
  wholeJson(entry: TypedEntry, findings: TypedFinding[]): unknown;
}

interface TypedPacket extends PacketResult {
  index: number;
  suite: string;
  id: string;
  argv: string[];
  cwd: string;
  preset: string | null;
  config: string | null;
  binary: BinaryMember;
  input: InputObservation;
  configuration: InputObservation | null;
  vueConfiguration: InputObservation | null;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: SerializedFailure | null;
  stdout: string;
  stderr: string;
  raw: {
    stdoutBase64: string;
    stdoutSha256: string;
    stderrBase64: string;
    stderrSha256: string;
  };
  expected: unknown;
  expectedStatus: number;
}

export function observePackets(
  fixtures: TypedFixtures,
  distribution: Distribution,
  consumer: string,
  output: string,
): Observation[] {
  const observations: Observation[] = [];
  const packets = path.join(output, "packets");
  fs.mkdirSync(packets);
  for (const [index, scenario] of fixtures.vectors.entries()) {
    const { entry, preset, config, findings, suite } = scenario;
    const directory = path.join(consumer, "cases", String(index).padStart(3, "0"));
    fs.mkdirSync(directory, { recursive: true });
    const filename = path.join(directory, entry.filename);
    const configFile = path.join(directory, "vize.config.json.fixture");
    const vueConfig = config ? vueConfigFixtureSource(config) : null;
    fs.writeFileSync(filename, entry.source, { flag: "wx" });
    if (config) {
      // Retain the original bytes and hash separately from the active carrier.
      fs.writeFileSync(configFile, config, { flag: "wx" });
      writeVueConfigFixture(directory, config);
    }
    const argv = [
      "lint",
      ...(config ? ["-c", "vite.config.mjs"] : ["--no-config"]),
      ...(preset ? ["--preset", preset] : []),
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      "none",
      entry.filename,
    ];
    assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
    const run = spawnSync(distribution.member.binary, argv, {
      cwd: directory,
      env: process.env,
      timeout: 60_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    const stdout = run.stdout ?? Buffer.alloc(0),
      stderr = run.stderr ?? Buffer.alloc(0);
    const packet: TypedPacket = {
      index,
      suite,
      id: entry.id,
      argv,
      cwd: directory,
      preset,
      config,
      binary: distribution.member,
      input: inputObservation(filename, entry.source),
      configuration: config ? inputObservation(configFile, config) : null,
      vueConfiguration:
        vueConfig !== null
          ? inputObservation(path.join(directory, "vite.config.mjs"), vueConfig)
          : null,
      status: run.status,
      signal: run.signal,
      error: run.error ? { name: run.error.name, message: run.error.message } : null,
      stdout: stdout.toString("utf8"),
      stderr: stderr.toString("utf8"),
      raw: {
        stdoutBase64: stdout.toString("base64"),
        stdoutSha256: sha256(stdout),
        stderrBase64: stderr.toString("base64"),
        stderrSha256: sha256(stderr),
      },
      expected: fixtures.wholeJson(entry, findings),
      expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
    };
    try {
      verifyPacket(packet, stdout, stderr);
      if (packet.vueConfiguration) {
        assert.equal(packet.vueConfiguration.readError, null, "Vue carrier disappeared");
        assert.equal(packet.vueConfiguration.afterText, packet.vueConfiguration.text);
        assert.equal(packet.vueConfiguration.afterSha256, packet.vueConfiguration.sha256);
      }
      assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
      packet.passed = true;
    } catch (error) {
      packet.passed = false;
      packet.failure = serializeFailure(error);
    }
    const name = `${String(index).padStart(3, "0")}-${suite}-${entry.id}.json`;
    const bytes = Buffer.from(`${JSON.stringify(packet, null, 2)}\n`);
    fs.writeFileSync(path.join(packets, name), bytes, { flag: "wx" });
    observations.push({
      index,
      suite,
      id: entry.id,
      packet: `packets/${name}`,
      sha256: sha256(bytes),
      passed: packet.passed === true,
    });
    fs.writeFileSync(
      path.join(output, "observations.json"),
      `${JSON.stringify(observations, null, 2)}\n`,
    );
  }
  assertLoadedBinary(distribution.member.binary, distribution.member, consumer);
  assert(
    observations.every((entry) => entry.passed),
    "original whole public CLI expectations failed; inspect every retained packet",
  );
  return observations;
}

/** Original 392-packet contract remains mandatory for the existing workflow. */
export function observe(
  fixtures: TypedFixtures,
  distribution: Distribution,
  consumer: string,
  output: string,
): Observation[] {
  const observations = observePackets(fixtures, distribution, consumer, output);
  assert.equal(observations.length, 392);
  assert.equal(observations.filter((entry) => entry.suite === "typed").length, 56);
  assert.equal(observations.filter((entry) => entry.suite === "first").length, 80);
  assert.equal(observations.filter((entry) => entry.suite === "second").length, 144);
  assert.equal(observations.filter((entry) => entry.suite === "returns").length, 112);
  return observations;
}
