import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { installToolingProgram, toolingProgram } from "./support/program-path.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const helpers = path.join(root, ".github/actions/test-js-packages-with-history");
const helper = pathToFileURL(path.join(helpers, "native-fault-evidence.ts")).href;
const commands = [
  ["vp", "run", "--filter", "./npm/native", "build:ci"],
  ["vp", "run", "--filter", "./npm/ui", "check"],
];

// The executable below is a synthetic fixture. No package manager, product,
// native addon or compiler is executed by these command-boundary controls.
function fixture(first: string, second = "0", setup = "", observed = true) {
  const directory = fs.mkdtempSync(path.join(tmpdir(), "vize-ui-phases-"));
  const outcome = path.join(directory, "js-native-fault-evidence-phases-1-ui/outcome.json");
  installToolingProgram(path.join(directory, "vp"), "workflow/mock-ui-task.cjs");

  const env = {
    PATH: directory,
    UI_BUILD_OUTCOME: first,
    UI_CHECK_OUTCOME: second,
    GITHUB_ACTIONS: String(observed),
    GITHUB_RUN_ID: "phases",
    GITHUB_RUN_ATTEMPT: "1",
    GITHUB_JOB: "ui",
    RUNNER_TEMP: directory,
    PHASE_OUTCOME: outcome,
    VIZE_PHASE_NONCE: "space and 'quote'\nnewline",
  };
  const result = spawnSync(
    process.execPath,
    [toolingProgram("workflow/observe-ui-phases.js"), setup],
    {
      cwd: directory,
      env,
      encoding: "utf8",
    },
  );
  // Compare the whole effective environment with an independent launch under
  // the same finite input. macOS adds its own text-encoding variable at launch.
  const baseline = spawnSync(
    process.execPath,
    [toolingProgram("process/print-process-environment.js")],
    {
      env,
      encoding: "utf8",
    },
  );
  assert.equal(baseline.status, 0, baseline.stderr);
  const calls = fs.existsSync(path.join(directory, "calls.jsonl"))
    ? fs
        .readFileSync(path.join(directory, "calls.jsonl"), "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line))
    : [];
  return { directory, outcome, result, calls, env: JSON.parse(baseline.stdout) };
}

test("closed UI mode executes both original argv in one source/session/shared128 observation", () => {
  const value = fixture("0");
  try {
    assert.equal(value.result.status, 0, value.result.stderr);
    assert.equal(value.result.signal, null);
    assert.equal(value.result.stdout, "build stdout\nui stdout\n");
    assert.equal(value.result.stderr, "build stderr\nui stderr\n");
    assert.deepEqual(
      value.calls.map((row) => row.argv),
      commands.map((row) => row.slice(1)),
    );
    for (const call of value.calls) {
      assert.equal(call.cwd, fs.realpathSync(value.directory));
      assert.deepEqual(call.env, value.env);
    }
    const receipt = JSON.parse(fs.readFileSync(value.outcome, "utf8"));
    assert.equal(receipt.mode, "ui-conformance");
    assert.deepEqual(
      receipt.declaredPhases,
      commands.map((command, index) => ({ name: index ? "ui-check" : "native-build", command })),
    );
    assert.equal(receipt.helpers.length, 5);
    assert.equal(receipt.commandReturnCode, 0);
    assert.equal(receipt.phases.length, 2);
    assert.equal(value.calls[1].snapshot.phases[0].commandReturnCode, 0);
    assert.equal(value.calls[1].snapshot.activePhase, "ui-check");
    assert.deepEqual(
      receipt.phases.map((phase: Record<string, unknown>) => [
        phase.commandExitCode,
        phase.commandSignal,
        phase.commandReturnCode,
      ]),
      [
        [0, null, 0],
        [0, null, 0],
      ],
    );
    assert.equal(receipt.phases[0].sampleSequenceStart, 0);
    assert.equal(receipt.phases[0].sampleSequenceEnd, receipt.phases[1].sampleSequenceStart);
    const retained = receipt.processRetention;
    assert.equal(retained.packetLimit, 128);
    assert.equal(retained.retainedPackets, 128);
    assert.equal(retained.observedPackets, receipt.phases[1].sampleSequenceEnd + 80);
    assert.equal(retained.droppedPackets, retained.observedPackets - 128);
    assert.equal(retained.retained[0].sequence, 0);
    assert.equal(retained.retained.at(-1).phase, "terminal");
    assert.equal(retained.terminalObservation.packetWrites, 80);
    assert.equal(fs.readdirSync(path.join(path.dirname(value.outcome), "processes")).length, 128);
    assert.deepEqual(
      JSON.parse(
        fs.readFileSync(path.join(path.dirname(value.outcome), "source-before.json"), "utf8"),
      ),
      { synthetic: true },
    );
    assert.deepEqual(
      JSON.parse(
        fs.readFileSync(path.join(path.dirname(value.outcome), "source-after.json"), "utf8"),
      ),
      { synthetic: true },
    );
  } finally {
    fs.rmSync(value.directory, { recursive: true, force: true });
  }
});

for (const [first, second, status, count] of [
  ["23", "0", 23, 1],
  ["0", "7", 7, 2],
  ["SIGTERM", "0", 143, 1],
  ["0", "SIGTERM", 143, 2],
] as const) {
  test(`UI && contract preserves ${first}/${second} and skips only the unstarted phase`, () => {
    const value = fixture(first, second);
    try {
      assert.equal(value.result.status, status, value.result.stderr);
      assert.equal(value.result.signal, null);
      assert.equal(value.calls.length, count);
      const receipt = JSON.parse(fs.readFileSync(value.outcome, "utf8"));
      assert.equal(receipt.phases.length, count);
      assert.equal(receipt.commandReturnCode, status);
      const phase = receipt.phases.at(-1);
      assert.equal(phase.commandSignal, status === 143 ? "SIGTERM" : null);
      assert.equal(phase.commandExitCode, status === 143 ? null : status);
      assert.equal(phase.commandReturnCode, status === 143 ? -15 : status);
      assert.equal(phase.shellReturnCode, status);
      assert.ok(receipt.processRetention.terminalObservation.finishedAt);
    } finally {
      fs.rmSync(value.directory, { recursive: true, force: true });
    }
  });
}

for (const [code, status, setup] of [
  ["ENOENT", 127, "missing-command"],
  ["EACCES", 126, "non-executable-command"],
] as const) {
  test(`UI shell launch refusal ${code} preserves ${status} and never starts phase two`, () => {
    const value = fixture("0", "0", setup);
    try {
      assert.equal(value.result.status, status, value.result.stderr);
      assert.equal(value.result.signal, null);
      assert.deepEqual(value.calls, []);
      const receipt = JSON.parse(fs.readFileSync(value.outcome, "utf8"));
      assert.equal(receipt.commandReturnCode, status);
      assert.equal(receipt.phases.length, 1);
      const phase = receipt.phases[0];
      assert.deepEqual(phase.command, commands[0]);
      assert.equal(phase.commandExitCode, null);
      assert.equal(phase.commandSignal, null);
      assert.equal(phase.commandReturnCode, 1);
      assert.equal(phase.shellReturnCode, status);
      assert.deepEqual(phase.commandSpawnError, { name: "Error", code });
      assert.equal(phase.sampleSequenceStart, 0);
      assert.equal(phase.sampleSequenceEnd, 0);
      assert.equal(receipt.processRetention.observedPackets, 0);
    } finally {
      fs.rmSync(value.directory, { recursive: true, force: true });
    }
  });
}

test("default single-command launch refusal retains its original exit1 behavior", () => {
  const result = spawnSync(
    process.execPath,
    [toolingProgram("workflow/observe-missing-ui-command.js"), helper],
    {
      env: { PATH: "/nonexistent-vize-pure-fixture-path" },
      encoding: "utf8",
    },
  );
  assert.equal(result.status, 1);
  assert.match(result.stderr, /ENOENT/);
});

test("phase outcome I/O and setup refusal preserve the first product failure and execute it once", () => {
  for (const setup of ["unwritable-outcome", "occupied-directory"]) {
    const value = fixture("29", "0", setup);
    try {
      assert.equal(value.result.status, 29, value.result.stderr);
      assert.equal(value.calls.length, 1);
      assert.equal(value.result.stdout, "build stdout\n");
      assert.equal(value.result.stderr, "build stderr\n");
    } finally {
      fs.rmSync(value.directory, { recursive: true, force: true });
    }
  }
});

test("non-Actions pass-through still executes the same closed chain and shell status", () => {
  const value = fixture("0", "7", "", false);
  try {
    assert.equal(value.result.status, 7, value.result.stderr);
    assert.deepEqual(
      value.calls.map((row) => row.argv),
      commands.map((row) => row.slice(1)),
    );
    assert.equal(fs.existsSync(value.outcome), false);
  } finally {
    fs.rmSync(value.directory, { recursive: true, force: true });
  }
});

test("closed UI mode refuses extra options instead of accepting an arbitrary command plan", () => {
  for (const argv of [
    ["--ui-conformance", "extra"],
    ["--ui-conformance", "--", "true"],
  ]) {
    const result = spawnSync(
      process.execPath,
      [
        toolingProgram("workflow/observe-ui-arguments.js"),
        JSON.stringify({ observerModule: helper, args: argv }),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Expected -- followed by the original command/);
  }
});
