import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  setup,
  updateJson,
  unusableFailure,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

function artifactPath(fixture: ReturnType<typeof setup>) {
  return path.join(fixture.reportDir, "fixture-typecheck-divergence.json");
}

test("typecheck divergence report bounds a vue-tsc baseline that ignores SIGTERM", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.hangTimeoutMs = 80),
    );
    writeVueTsc(
      fixture.vueTsc,
      `import { spawn } from "node:child_process";
process.on("SIGTERM", () => {});
spawn(process.execPath, ["-e", ${JSON.stringify(
        "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);",
      )}], { stdio: "inherit" });
setInterval(() => {}, 1000);`,
      fixture.invocationPath,
    );

    const startedAt = Date.now();
    const result = run(fixture, {}, [], { timeoutMs: 8_000 });
    const reason = "vue-tsc baseline failed to run: spawn timed out after 80ms";
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "timeout handling must not wait for a vue-tsc process tree that ignores SIGTERM",
    );
    assert.equal(result.stderr, `${unusableFailure(reason)}\n`);
    const artifact = readJson(artifactPath(fixture));
    assert.equal(artifact.baseline.exitCode, null);
    assert.equal(artifact.baseline.runError, reason);
    assert.equal(
      artifact.baseline.coverageRunError,
      `vue-tsc coverage baseline skipped because ${reason}`,
    );
    assert.equal(artifact.baseline.configuration.unusableReason, reason);
    assert.equal(artifact.baseline.coverage.unusableReason, reason);
    assert.equal(artifact.budget.verdict, "unusable");
    assert.equal(artifact.budget.passed, false);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report records a vue-tsc coverage timeout as an unusable baseline", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.hangTimeoutMs = 80),
    );
    writeVueTsc(
      fixture.vueTsc,
      `if (process.argv.includes("--listFilesOnly")) {
  setInterval(() => {}, 1000);
} else {
  process.stdout.write("src/App.vue(1,1): error TS2322: shared\\n");
  process.exit(2);
}`,
      fixture.invocationPath,
    );

    const startedAt = Date.now();
    const result = run(fixture, {}, [], { timeoutMs: 8_000 });
    const reason = "vue-tsc coverage baseline failed to run: spawn timed out after 80ms";
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "coverage timeout handling must not wait for the outer test timeout",
    );
    assert.equal(result.stderr, `${unusableFailure(reason)}\n`);
    const artifact = readJson(artifactPath(fixture));
    assert.equal(artifact.baseline.exitCode, 2);
    assert.equal(artifact.baseline.runError, null);
    assert.equal(artifact.baseline.coverageExitCode, null);
    assert.equal(artifact.baseline.coverageRunError, reason);
    assert.equal(artifact.baseline.configuration.verdict, "usable");
    assert.equal(artifact.baseline.coverage.unusableReason, reason);
    assert.equal(artifact.mutationOracle.unusableReason, reason);
    assert.equal(artifact.budget.verdict, "unusable");
  } finally {
    cleanup(fixture);
  }
});

test("record-only mode still warns and uploads baseline timeout evidence", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  try {
    updateJson(
      fixture.registryPath,
      (registry) => (registry.projects[0].typecheckPerformance.hangTimeoutMs = 80),
    );
    writeVueTsc(
      fixture.vueTsc,
      `process.on("SIGTERM", () => {});
setInterval(() => {}, 1000);`,
      fixture.invocationPath,
    );

    const result = run(fixture, {}, ["--budget-mode", "record-only"], { timeoutMs: 8_000 });
    const reason = "vue-tsc baseline failed to run: spawn timed out after 80ms";
    assert.equal(result.status, 0, result.stderr);
    assert.ok(
      result.stdout.includes(
        `::warning title=Typecheck divergence budget not enforced::${unusableFailure(reason)}`,
      ),
    );
    assert.equal(readJson(artifactPath(fixture)).budget.passed, false);
  } finally {
    cleanup(fixture);
  }
});
