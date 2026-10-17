import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./vue-config-fixture.ts";

const RULES = ["script/return-in-computed-property", "script/return-in-emits-validator"];
const OLD_NEEDLES = ["=> {", "=>{", "function", "get("];
const FILE_COUNT = 64;
const MEMBERS_PER_RULE = 64;
const WARMUPS = 2;
const SAMPLES = 6;
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const orderPaths = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

function workloadSource(variant) {
  const member = (name, parameter, value) => {
    if (variant === 0) return `    ${name}(${parameter}) { return ${value} }`;
    if (variant === 1) return `    ${name} /* own body */ (${parameter}) { return ${value} }`;
    const trivia = variant === 2 ? "\n" : " /* own body */\n";
    return `    ${name}: (${parameter}) =>${trivia}    { return ${value} }`;
  };
  const computed = [];
  const emits = [];
  for (let index = 0; index < MEMBERS_PER_RULE; index++) {
    computed.push(member(`metric${index}`, "", String(index)));
    emits.push(member(`event${index}`, "payload: number", "payload > 0"));
  }
  return (
    '<script lang="ts">\nexport default {\n  computed: {\n' +
    computed.join(",\n") +
    "\n  },\n  emits: {\n" +
    emits.join(",\n") +
    "\n  }\n}\n</script>\n"
  );
}

function binaryIdentity(binary) {
  const bytes = fs.readFileSync(binary);
  return { realpath: fs.realpathSync(binary), bytes: bytes.length, sha256: sha256(bytes) };
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

/**
 * The caller validates the source-build receipt before passing this executable.
 * This measures two configurations of that same executable, not base/head code.
 * Config loading and entry selection differ; elapsed time includes both of them.
 */
export function observeReturnPrefilterCost({ binary, directory, persist }) {
  assert.equal(typeof persist, "function");
  const executable = path.resolve(binary);
  const workspace = fs.mkdtempSync(path.join(directory, "return-prefilter-cost-"));
  const configFile = "vite.config.mjs";
  const configSource =
    JSON.stringify({ linter: { rules: Object.fromEntries(RULES.map((rule) => [rule, "off"])) } }) +
    "\n";
  const configCarrierSource = vueConfigFixtureSource(configSource);
  const packet = {
    schema: "vize.lint.return-prefilter-controlled-cost",
    version: 1,
    status: "RUNNING",
    scope: "Controlled marginal producer-scan measurement of one verified source CLI executable.",
    limits: [
      "Default dispatch versus disabling only the two return rules; entry-selection and config-loading costs differ.",
      "Whole-process elapsed time includes startup, input IO, shared parsing, all other rules, and JSON output.",
      "Warmups warm filesystem/runtime caches; every trial starts a fresh CLI process.",
      "This is not a base/head source regression gate, an isolated per-rule timer, a speedup claim, or a budget waiver.",
      "Existing protected instruction/performance gates remain required; no instruction or allocation counts are measured here.",
    ],
    binary: { path: executable, before: binaryIdentity(executable), after: null },
    workspace,
    runner: { platform: process.platform, arch: process.arch, nodeVersion: process.version },
    environment: { RAYON_NUM_THREADS: "1", NO_COLOR: "1", FORCE_COLOR: "0" },
    timing: {
      clock:
        "performance.now around spawnSync only; generation, hashes, checks and persistence are outside the timer",
      serial: true,
      order:
        "Each paired round alternates default/disabled order; two modes never run concurrently",
      warmupsPerMode: WARMUPS,
      samplesPerMode: SAMPLES,
    },
    workload: {
      fileCount: FILE_COUNT,
      computedGettersPerFile: MEMBERS_PER_RULE,
      emitsValidatorsPerFile: MEMBERS_PER_RULE,
      totalOwnValueReturns: FILE_COUNT * MEMBERS_PER_RULE * 2,
      variants: ["method", "method-trivia", "arrow-newline", "arrow-comment-newline"],
      excludedOldNeedles: OLD_NEEDLES,
      files: [],
    },
    configurations: {
      default: { argv: ["--no-config"], source: null, sha256: null },
      disabled: {
        argv: ["-c", configFile],
        file: configFile,
        source: configCarrierSource,
        bytes: Buffer.byteLength(configCarrierSource),
        sha256: sha256(configCarrierSource),
        nativeSource: configSource,
        nativeBytes: Buffer.byteLength(configSource),
        nativeSha256: sha256(configSource),
        disabledRules: RULES,
      },
    },
    expected: [],
    trials: [],
  };
  persist(packet);
  try {
    assert.equal(writeVueConfigFixture(workspace, configSource), path.join(workspace, configFile));
    for (let index = 0; index < FILE_COUNT; index++) {
      const file = `ReturnScan${String(index).padStart(2, "0")}.vue`;
      const variant = index % packet.workload.variants.length;
      const source = workloadSource(variant);
      for (const needle of OLD_NEEDLES) assert.ok(!source.includes(needle), `${file}: ${needle}`);
      assert.ok(source.includes("computed") && source.includes("emits") && source.includes("{"));
      fs.writeFileSync(path.join(workspace, file), source, { flag: "wx" });
      packet.workload.files.push({
        file,
        variant: packet.workload.variants[variant],
        source,
        bytes: Buffer.byteLength(source),
        sha256: sha256(source),
      });
      packet.expected.push({ file, messages: [], errorCount: 0, warningCount: 0 });
    }
    const files = packet.workload.files.map((entry) => entry.file);
    packet.workload.manifestSha256 = sha256(
      JSON.stringify(
        packet.workload.files.map(({ file, bytes, sha256 }) => ({ file, bytes, sha256 })),
      ),
    );
    persist(packet);

    const assertInputs = () => {
      assert.deepEqual(
        fs.readdirSync(workspace).sort(orderPaths),
        [...files, configFile].sort(orderPaths),
      );
      for (const entry of packet.workload.files) {
        const bytes = fs.readFileSync(path.join(workspace, entry.file));
        assert.equal(bytes.length, entry.bytes, entry.file);
        assert.equal(sha256(bytes), entry.sha256, entry.file);
        assert.equal(bytes.toString("utf8"), entry.source, entry.file);
      }
      assert.deepEqual(
        fs.readFileSync(path.join(workspace, configFile)),
        Buffer.from(configCarrierSource),
      );
    };
    let output;
    const trial = (phase, round, mode) => {
      assertInputs();
      const argv = [
        "lint",
        ...packet.configurations[mode].argv,
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "none",
        ...files,
      ];
      const started = performance.now();
      const run = spawnSync(executable, argv, {
        cwd: workspace,
        env: { ...process.env, ...packet.environment },
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const elapsedMs = performance.now() - started;
      const stdout = run.stdout ?? Buffer.alloc(0);
      const stderr = run.stderr ?? Buffer.alloc(0);
      const observation = {
        sequence: packet.trials.length,
        phase,
        round,
        mode,
        argv,
        inputManifestSha256: packet.workload.manifestSha256,
        configSha256: packet.configurations[mode].sha256,
        elapsedMs,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: stdout.toString("utf8"),
        stdoutBase64: stdout.toString("base64"),
        stderr: stderr.toString("utf8"),
        stderrBase64: stderr.toString("base64"),
        actual: null,
      };
      packet.trials.push(observation);
      persist(packet);
      assert.equal(observation.error, null);
      assert.equal(observation.signal, null);
      assert.equal(observation.status, 0);
      assert.equal(stderr.length, 0);
      assert.ok(Number.isFinite(elapsedMs) && elapsedMs > 0);
      assert.deepEqual(Buffer.from(observation.stdout), stdout, "CLI stdout must be valid UTF-8");
      observation.actual = JSON.parse(observation.stdout);
      persist(packet);
      assert.deepEqual(observation.actual, packet.expected, `${phase}/${round}/${mode}`);
      if (output === undefined) output = observation.stdout;
      assert.equal(
        observation.stdout,
        output,
        "Every mode/trial must retain identical complete JSON bytes",
      );
      assertInputs();
    };

    for (const [phase, rounds] of [
      ["warmup", WARMUPS],
      ["sample", SAMPLES],
    ]) {
      for (let round = 0; round < rounds; round++) {
        const modes = round % 2 ? ["disabled", "default"] : ["default", "disabled"];
        for (const mode of modes) trial(phase, round, mode);
      }
    }
    assert.equal(packet.trials.length, (WARMUPS + SAMPLES) * 2);
    const samples = packet.trials.filter((entry) => entry.phase === "sample");
    const enabled = samples.filter((entry) => entry.mode === "default");
    const disabled = samples.filter((entry) => entry.mode === "disabled");
    const pairedDeltaMs = enabled.map(
      (entry, index) => entry.elapsedMs - disabled[index].elapsedMs,
    );
    packet.summary = {
      defaultSamplesMs: enabled.map((entry) => entry.elapsedMs),
      disabledSamplesMs: disabled.map((entry) => entry.elapsedMs),
      defaultMedianMs: median(enabled.map((entry) => entry.elapsedMs)),
      disabledMedianMs: median(disabled.map((entry) => entry.elapsedMs)),
      pairedDefaultMinusDisabledMs: pairedDeltaMs,
      pairedDefaultMinusDisabledMedianMs: median(pairedDeltaMs),
      interpretation:
        "Observed configuration contrast, including different entry-selection/config-loading costs; no isolated producer cost or source-regression conclusion",
    };
    packet.status = "OBSERVED";
    return packet;
  } catch (error) {
    packet.status = "FAILED";
    packet.failure = { name: error.name, message: error.message };
    throw error;
  } finally {
    packet.binary.after = binaryIdentity(executable);
    if (JSON.stringify(packet.binary.after) !== JSON.stringify(packet.binary.before)) {
      packet.status = "FAILED";
      packet.binaryChanged = true;
    }
    persist(packet);
    assert.deepEqual(
      packet.binary.after,
      packet.binary.before,
      "source CLI binary must stay byte-identical",
    );
  }
}
