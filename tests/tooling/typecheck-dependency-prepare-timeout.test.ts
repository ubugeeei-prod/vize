import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  artifactPath,
  cleanup,
  commit,
  git,
  run,
  setup,
  successBody,
  writeJson,
  writeManager,
} from "./support/typecheck-dependency-prepare-fixture.ts";

test("dependency prepare bounds an install process that ignores SIGTERM", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  try {
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      `const { spawn } = await import("node:child_process");
process.on("SIGTERM", () => {});
spawn(process.execPath, ["-e", ${JSON.stringify(
        "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);",
      )}], { stdio: "inherit" });
setInterval(() => {}, 1000);`,
      { spec: "pnpm@10.0.0" },
    );

    const startedAt = Date.now();
    const result = run(fixture, ["--timeout-ms", "80"], { timeoutMs: 8_000 });
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "timeout handling must not wait for an install process tree that ignores SIGTERM",
    );
    assert.match(result.stderr, /pnpm@10\.0\.0 install failed to run: spawn timed out after 80ms/);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare bounds a baseline prepare process that ignores SIGTERM", () => {
  if (process.platform === "win32") return;

  const fixture = setup();
  try {
    writeJson(fixture.registryPath, {
      projects: [
        {
          ...fixture.project,
          typecheckPerformance: {
            ...fixture.project.typecheckPerformance,
            baseline: {
              tsconfig: ".generated/tsconfig.json",
              prepare: ["pnpm", "exec", "fixture", "prepare"],
            },
          },
        },
      ],
    });
    git(fixture.fixtureRoot, ["add", "registry.json"]);
    commit(fixture.fixtureRoot, "configure baseline prepare");
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      `if (managerArgs[0] === "install") {
  ${successBody}
} else {
  const { spawn } = await import("node:child_process");
  fs.mkdirSync(".generated", { recursive: true });
  fs.writeFileSync(".generated/tsconfig.json", "{}\\n");
  process.on("SIGTERM", () => {});
  spawn(process.execPath, ["-e", ${JSON.stringify(
    "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);",
  )}], { stdio: "inherit" });
  setInterval(() => {}, 1000);
}`,
      { spec: "pnpm@10.0.0" },
    );

    const startedAt = Date.now();
    const result = run(fixture, ["--timeout-ms", "80"], { timeoutMs: 8_000 });
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "timeout handling must not wait for a baseline prepare process tree that ignores SIGTERM",
    );
    assert.match(result.stderr, /baseline prepare failed to run: spawn timed out after 80ms/);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});
