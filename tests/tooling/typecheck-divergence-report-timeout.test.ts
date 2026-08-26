import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  run,
  setup,
  updateJson,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

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
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "timeout handling must not wait for a vue-tsc process tree that ignores SIGTERM",
    );
    assert.match(result.stderr, /vue-tsc baseline failed to run: spawn timed out after 80ms/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});
