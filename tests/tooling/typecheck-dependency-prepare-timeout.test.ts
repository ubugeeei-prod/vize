import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { before, test } from "node:test";

import {
  artifactPath,
  cleanup,
  commit,
  git,
  run,
  script,
  setup,
  successBody,
  writeJson,
  writeManager,
} from "./support/typecheck-dependency-prepare-fixture.ts";
import { buildRustScript } from "./support/rust-script-build.ts";

// The install before the hanging prepare must succeed inside the same limit,
// so it gets more than one slow Node startup.
const succeedingStepTimeoutMs = 2_000;

// Build outside the timed window: a cold rust-script build exceeded the bound.
before(() => buildRustScript(script, ["--rust-script-build-check"]));

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
    const result = run(fixture, ["--timeout-ms", String(succeedingStepTimeoutMs)], {
      timeoutMs: 8_000,
    });
    assert.equal(result.status, 1);
    assert.ok(
      Date.now() - startedAt < 8_000,
      "timeout handling must not wait for a baseline prepare process tree that ignores SIGTERM",
    );
    assert.match(result.stderr, /baseline prepare failed to run: spawn timed out after 2000ms/);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("Misskey declaration preparation rejects failed, missing and empty emitter output", () => {
  const prepare = [
    "pnpm",
    "--filter",
    "misskey-js",
    "--filter",
    "misskey-reversi",
    "--filter",
    "misskey-bubble-game",
    "--filter",
    "i18n",
    "exec",
    "tsgo",
    "--project",
    "tsconfig.json",
    "--outDir",
    "built",
    "--declaration",
    "true",
    "--emitDeclarationOnly",
    "true",
  ];
  for (const [emitted, exitCode, message] of [
    ["export {};", 9, /baseline prepare exited with status 9/],
    [null, 0, /misskey-js declaration output is missing/],
    ["", 0, /declaration output must be a nonempty regular file/],
    ["export {};", 0, null],
  ] as const) {
    const fixture = setup();
    try {
      writeJson(fixture.registryPath, {
        projects: [
          {
            ...fixture.project,
            id: "misskey",
            typecheckPerformance: {
              ...fixture.project.typecheckPerformance,
              baseline: { tsconfig: fixture.project.tsconfig, prepare },
            },
          },
        ],
      });
      git(fixture.fixtureRoot, ["add", "registry.json"]);
      commit(fixture.fixtureRoot, "configure Misskey declaration preparation");
      writeManager(
        fixture.runner,
        fixture.invocationPath,
        "10.0.0",
        `if (managerArgs[0] === "install") { ${successBody} } else {
          const emitted = ${JSON.stringify(emitted)};
          if (emitted !== null) {
            for (const name of ["misskey-js", "misskey-reversi", "misskey-bubble-game", "i18n"]) {
              fs.mkdirSync("packages/" + name + "/built", { recursive: true });
              fs.writeFileSync("packages/" + name + "/built/index.d.ts", emitted);
            }
          }
          process.exit(${exitCode});
        }`,
        { spec: "pnpm@10.0.0" },
      );
      const result = run(fixture);
      const artifact = path.join(fixture.outputDir, "misskey-typecheck-dependencies.json");
      const invocation = JSON.parse(fs.readFileSync(fixture.invocationPath, "utf8"));
      assert.equal(invocation.cwd, fixture.fixtureRoot);
      assert.deepEqual(invocation.managerArgs, prepare.slice(1));
      if (message !== null) {
        assert.equal(result.status, 1);
        assert.match(result.stderr, message);
        assert.equal(fs.existsSync(artifact), false);
      } else {
        assert.equal(result.status, 0, result.stderr);
        const receipt = JSON.parse(fs.readFileSync(artifact, "utf8"));
        assert.deepEqual(receipt.baselinePrepare.command, prepare);
        assert.equal(receipt.baselinePrepare.exitCode, 0);
      }
    } finally {
      cleanup(fixture);
    }
  }
});
