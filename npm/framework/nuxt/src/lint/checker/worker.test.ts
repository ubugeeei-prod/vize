import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  NuxtLintCheckerWorker,
  runNuxtLintCheckerTask,
  type NuxtLintCheckerTask,
} from "./worker.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const fakeOxlint = path.join(here, "fixtures", "fake-oxlint.mjs");

function task(overrides: Partial<NuxtLintCheckerTask> = {}): NuxtLintCheckerTask {
  return {
    cwd: here,
    configFile: "/project/.nuxt/oxlint.config.json",
    exclude: ["generated/**", "**/node_modules/**"],
    fix: true,
    formatter: "json",
    oxlintEntrypoint: fakeOxlint,
    targets: ["/project/app/pages/error.vue"],
    emitError: true,
    emitWarning: true,
    ...overrides,
  };
}

void test("worker execution reports the complete filtered diagnostic payload", async () => {
  const result = await runNuxtLintCheckerTask(task());
  assert.equal(result.hasErrors, true);
  assert.equal(result.hasWarnings, true);
  assert.equal(result.diagnosticCount, 2);
  const payload = JSON.parse(result.output) as {
    diagnostics: Array<{ message: string; severity: string }>;
  };
  assert.deepEqual(
    payload.diagnostics.map(({ severity }) => severity),
    ["error", "warning"],
  );
  assert.match(payload.diagnostics[0].message, /--config/u);
  assert.match(payload.diagnostics[0].message, /--fix/u);
  assert.match(payload.diagnostics[0].message, /--ignore-pattern generated\/\*\*/u);
});

void test("worker rebases project-absolute globs for oxlint", async () => {
  const result = await runNuxtLintCheckerTask(
    task({
      exclude: [path.join(here, "generated/**")],
      targets: [path.join(here, "app/**/*.{ts,vue}")],
    }),
  );
  const message = (JSON.parse(result.output) as { diagnostics: Array<{ message: string }> })
    .diagnostics[0].message;
  assert.match(message, /--ignore-pattern generated\/\*\*/u);
  assert.match(message, /app\/\*\*\/\*\.ts/u);
  assert.match(message, /app\/\*\*\/\*\.vue/u);
  assert.match(message, /app\/\*\.ts/u);
  assert.match(message, /app\/\*\.vue/u);
});

void test("worker execution honours emitError and emitWarning independently", async () => {
  const warnings = await runNuxtLintCheckerTask(task({ emitError: false }));
  assert.equal(warnings.hasErrors, false);
  assert.equal(warnings.hasWarnings, true);
  assert.equal(warnings.diagnosticCount, 1);
  assert.deepEqual(
    (JSON.parse(warnings.output) as { diagnostics: Array<{ severity: string }> }).diagnostics.map(
      ({ severity }) => severity,
    ),
    ["warning"],
  );

  const errors = await runNuxtLintCheckerTask(task({ emitWarning: false }));
  assert.equal(errors.hasErrors, true);
  assert.equal(errors.hasWarnings, false);
  assert.equal(errors.diagnosticCount, 1);

  assert.deepEqual(await runNuxtLintCheckerTask(task({ emitError: false, emitWarning: false })), {
    diagnosticCount: 0,
    hasErrors: false,
    hasWarnings: false,
    output: "",
  });
  await assert.rejects(
    runNuxtLintCheckerTask(
      task({
        emitError: false,
        emitWarning: false,
        fix: true,
        oxlintEntrypoint: "/missing/oxlint",
      }),
    ),
    /Failed to start oxlint/u,
  );
});

void test("long-lived worker runs outside the main thread and closes cleanly", async (t) => {
  const worker = new NuxtLintCheckerWorker();
  t.after(() => worker.close());

  const result = await worker.run(task({ fix: false, targets: ["/project/app/pages/warn.vue"] }));
  assert.equal(result.hasErrors, true);
  assert.equal(result.hasWarnings, true);
  assert.equal(result.diagnosticCount, 2);
});

void test("worker failures carry actionable subprocess context", async () => {
  await assert.rejects(
    runNuxtLintCheckerTask(task({ oxlintEntrypoint: "/missing/oxlint" })),
    /Failed to start oxlint/u,
  );
});
