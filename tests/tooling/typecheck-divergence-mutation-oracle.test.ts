import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { before, test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  script,
  setup,
  sha256,
  unusableFailure,
  updateJson,
  updateVizeOutput,
} from "./_helpers/typecheck-divergence-report-fixture.ts";
import { buildRustScript } from "./support/rust-script-build.ts";

// Build outside the timed window: a cold rust-script build exceeded the bound.
before(() => buildRustScript(script, ["--rust-script-build-check"]));

const outerHarnessTimeoutMs = 20_000;

function artifactPath(fixture: ReturnType<typeof setup>, extension: string) {
  return path.join(fixture.reportDir, `fixture-typecheck-divergence.${extension}`);
}

test("seeded mutation oracle accepts a shared probe with shifted compiler coordinates", () => {
  const fixture = setup({
    baselineOutput: "",
    baselineFiles: ["src/App.vue"],
    baselineMutation: "shifted",
    vizeDiagnostics: [],
    vizeMutation: "shifted",
  });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const oracle = readJson(artifactPath(fixture, "json")).mutationOracle;
    assert.equal(oracle.passed, true);
    assert.equal(oracle.expectedDiagnosticMatched, true);
    assert.equal(oracle.file, "src/App.vue");
    assert.deepEqual(oracle.span, { line: 3, column: 1 });
    assert.equal(oracle.states.length, 3);
    const [cleanState, brokenState, repairedState] = oracle.states;
    assert.equal(cleanState.sharedCount, 0);
    assert.equal(cleanState.messageMismatchCount, 0);
    assert.equal(brokenState.sharedCount, 1);
    assert.equal(brokenState.messageMismatchCount, 0);
    assert.equal(repairedState.sharedCount, 0);
    assert.equal(repairedState.messageMismatchCount, 0);
    assert.notEqual(brokenState.sourceSha256, cleanState.sourceSha256);
    assert.equal(repairedState.sourceSha256, cleanState.sourceSha256);
  } finally {
    cleanup(fixture);
  }
});

test("seeded mutation oracle narrows checker reruns to the selected probe file", () => {
  const fixture = setup({
    baselineOutput: "",
    baselineFiles: ["src/App.vue", "src/Other.vue"],
    vizeDiagnostics: [],
  });
  try {
    fs.writeFileSync(path.join(fixture.fixtureRoot, "src", "Other.vue"), "<template />\n");
    updateVizeOutput(fixture, (parsed) => {
      parsed.fileCount = 2;
      parsed.files = [
        { file: "src/App.vue", diagnostics: [] },
        { file: "src/Other.vue", diagnostics: [] },
      ];
    });

    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(artifactPath(fixture, "json"));
    const oracle = artifact.mutationOracle;
    assert.equal(oracle.passed, true);

    const fullBaselineConfig = readJson(
      path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json"),
    );
    assert.deepEqual(fullBaselineConfig.files, ["../src/App.vue", "../src/Other.vue"]);

    const mutationConfig = readJson(
      path.join(fixture.fixtureRoot, ".vize-baseline", "fixture-mutation-vue-tsc.tsconfig.json"),
    );
    assert.deepEqual(mutationConfig.files, [`../${oracle.file}`]);

    for (const state of oracle.states) {
      assert.match(state.vize.command, new RegExp(` check ${oracle.file.replace("/", "\\/")} `));
      assert.doesNotMatch(state.vize.command, /src\/\*\*\/\*\.vue/u);
      assert.match(state.baseline.command, /fixture-mutation-vue-tsc\.tsconfig\.json/u);
    }
  } finally {
    cleanup(fixture);
  }
});

test("seeded mutation oracle bounds a Vize mutation process that ignores SIGTERM", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  const sourcePath = path.join(fixture.fixtureRoot, "src/App.vue");
  const originalSource = fs.readFileSync(sourcePath, "utf8");
  const readyPath = path.join(fixture.fakeDir, "mutation-child-ready.json");
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.hangTimeoutMs = 2_000),
      // The vue-tsc baseline before the hanging mutation run must finish in it.
    );
    fs.writeFileSync(
      fixture.vize,
      `#!/usr/bin/env node
import { spawn } from "node:child_process";
if (process.argv.includes("--version")) { console.log("vize 0.0.0-test"); process.exit(0); }
process.on("SIGTERM", () => {});
spawn(process.execPath, ["-e", ${JSON.stringify(
        String.raw`const fs = require("node:fs");
process.on("SIGTERM", () => {});
const stat = fs.readFileSync("/proc/self/stat", "utf8");
const fields = stat.slice(stat.lastIndexOf(")") + 1).trim().split(/\s+/u);
fs.writeFileSync(${JSON.stringify(readyPath)}, JSON.stringify({ pid: process.pid, parentPid: process.ppid, group: Number(fields[2]), session: Number(fields[3]) }));
process.stdout.write("mutation-child-started\n");
process.stderr.write("mutation-child-evidence\n");
setInterval(() => {}, 1000);`,
      )}], { stdio: "inherit" });
setInterval(() => {}, 1000);
`,
    );
    fs.chmodSync(fixture.vize, 0o755);

    const startedAt = Date.now();
    const result = run(fixture, {}, [], { timeoutMs: outerHarnessTimeoutMs, retainEvidence: true });
    assert.equal(result.error, undefined, result.stderr);
    assert.equal(result.status, 1, result.stderr);
    assert.ok(
      Date.now() - startedAt < outerHarnessTimeoutMs,
      "timeout handling must not wait for a Vize mutation process tree that ignores SIGTERM",
    );
    const deadlines = path.join(fixture.reportDir, "typecheck-deadlines");
    const roots = fs.readdirSync(deadlines);
    assert.equal(roots.length, 1);
    const directory = path.join(deadlines, roots[0]!);
    const supervisor = readJson(path.join(directory, "supervisor.json"));
    assert.equal(supervisor.budgetMs, 2_000);
    assert.ok(supervisor.elapsedMs < outerHarnessTimeoutMs);
    const deadlinePath = path.join(directory, "deadline-result.json");
    if (supervisor.timedOut === true) {
      assert.equal(result.stderr, `${unusableFailure("whole-project deadline exhausted")}\n`);
      const deadline = readJson(deadlinePath);
      assert.deepEqual(
        {
          schema: deadline.schema,
          version: deadline.version,
          project: deadline.project,
          completed: deadline.completed,
          instrumentFailure: deadline.instrumentFailure,
          budgetMs: deadline.budgetMs,
        },
        {
          schema: "vize.fixtureTypecheckProjectDeadline",
          version: 1,
          project: "fixture",
          completed: false,
          instrumentFailure: true,
          budgetMs: 2_000,
        },
      );
      assert.ok(deadline.elapsedMs >= 2_000 && deadline.elapsedMs < outerHarnessTimeoutMs);
    } else {
      // The worker's floored remaining budget can expire before the outer poll.
      // Its next phase then rejects the incomplete project, without a final report.
      assert.equal(supervisor.timedOut, false);
      assert.deepEqual(readJson(path.join(directory, "worker-result.json")), {
        ok: false,
        error: "whole-project typecheck deadline exhausted",
      });
      assert.equal(result.stderr, "whole-project typecheck deadline exhausted\n");
      assert.equal(fs.existsSync(deadlinePath), false);
    }
    assert.equal(readJson(path.join(directory, "phase-current.json")).phase, "Vize mutation run");
    const commands = fs
      .readdirSync(directory)
      .filter((name) => /^command-\d+\.json$/u.test(name))
      .filter((name) => readJson(path.join(directory, name)).label === "Vize mutation run");
    assert.equal(commands.length, 1);
    const command = commands[0]!.replace(/\.json$/u, "");
    assert.equal(
      fs.readFileSync(path.join(directory, `${command}-stdout.bin`), "utf8"),
      "mutation-child-started\n",
    );
    assert.equal(
      fs.readFileSync(path.join(directory, `${command}-stderr.bin`), "utf8"),
      "mutation-child-evidence\n",
    );
    const ready = readJson(readyPath);
    assert.equal(ready.session, supervisor.session);
    assert.equal(ready.group, ready.parentPid);
    assert.notEqual(ready.pid, ready.parentPid);
    assert.equal(supervisor.cleanup.allWritersStopped, true);
    assert.deepEqual(Object.keys(supervisor.mutationRecovery), ["restored"]);
    assert.equal(typeof supervisor.mutationRecovery.restored, "boolean");
    for (const pid of [ready.pid, ready.parentPid]) {
      assert.ok(Number.isSafeInteger(pid) && pid > 0);
      const stat = `/proc/${pid}/stat`;
      try {
        assert.match(fs.readFileSync(stat, "utf8"), /\) [ZX] /u);
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
      }
    }
    assert.deepEqual(readJson(path.join(directory, "mutation-recovery.json")), {
      source: sourcePath,
      originalSha256: sha256(originalSource),
      restored: true,
    });
    assert.equal(fs.existsSync(path.join(directory, "active-mutation.json")), false);
    assert.equal(
      fs.existsSync(artifactPath(fixture, "json")),
      false,
      "an incomplete project must retain deadline evidence instead of a complete report",
    );
    assert.equal(fs.readFileSync(sourcePath, "utf8"), originalSource);
  } finally {
    cleanup(fixture);
  }
});
