import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  artifactPath,
  cleanup,
  commit,
  commitSha,
  expectedInvocationArgs,
  expectedInvocationCommand,
  expectedInvocationEnv,
  git,
  packageManagers,
  root,
  run,
  script,
  setup,
  successBody,
  writeJson,
  writeManager,
} from "./support/typecheck-dependency-prepare-fixture.ts";

test("dependency prepare uses each pinned manager's immutable install command", () => {
  for (const packageManager of packageManagers) {
    const fixture = setup(packageManager);
    try {
      const lockfile = fs.readFileSync(path.join(fixture.fixtureRoot, packageManager.lockfile));
      const result = run(fixture);
      assert.equal(result.status, 0, result.stderr);
      const artifact = JSON.parse(fs.readFileSync(artifactPath(fixture), "utf8"));
      assert.deepEqual(Object.keys(artifact).sort(), [
        "baselinePrepare",
        "evidence",
        "install",
        "lockfile",
        "packageManager",
        "project",
        "revision",
        "schema",
        "version",
      ]);
      assert.equal(artifact.schema, "vize.fixtureTypecheckDependencyInstall");
      assert.equal(artifact.version, 2);
      assert.equal(artifact.baselinePrepare, null);
      assert.equal(artifact.evidence.commitSha, commitSha);
      assert.deepEqual(artifact.packageManager, {
        name: packageManager.name,
        version: packageManager.version,
      });
      assert.deepEqual(artifact.lockfile, {
        path: packageManager.lockfile,
        sizeBytes: lockfile.byteLength,
        sha256: createHash("sha256").update(lockfile).digest("hex"),
      });
      assert.deepEqual(artifact.install.command, [
        packageManager.name,
        ...packageManager.installArgs,
      ]);
      assert.equal(artifact.install.exitCode, 0);
      assert.match(artifact.install.stdoutSha256, /^[0-9a-f]{64}$/);
      assert.match(artifact.install.stderrSha256, /^[0-9a-f]{64}$/);
      assert.deepEqual(JSON.parse(fs.readFileSync(fixture.invocationPath, "utf8")), {
        cwd: fixture.fixtureRoot,
        command: expectedInvocationCommand(packageManager),
        args: expectedInvocationArgs(packageManager),
        managerArgs: packageManager.installArgs,
        env: expectedInvocationEnv(packageManager),
      });
    } finally {
      cleanup(fixture);
    }
  }
});

test("dependency prepare keeps pnpm fixtures out of parent workspaces", () => {
  const packageManager = packageManagers.find((candidate) => candidate.name === "pnpm");
  assert.ok(packageManager);
  const fixture = setup(packageManager);
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const invocation = JSON.parse(fs.readFileSync(fixture.invocationPath, "utf8"));
    assert.equal(invocation.cwd, fixture.fixtureRoot);
    assert.equal(invocation.env.corepackProjectSpec, "0");
    assert.deepEqual(invocation.managerArgs, [
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
      "--ignore-workspace",
    ]);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare preserves a pnpm fixture's own workspace manifest", () => {
  const packageManager = packageManagers.find((candidate) => candidate.name === "pnpm");
  assert.ok(packageManager);
  const fixture = setup(packageManager);
  try {
    fs.writeFileSync(path.join(fixture.fixtureRoot, "pnpm-workspace.yaml"), "packages: []\n");
    git(fixture.fixtureRoot, ["add", "pnpm-workspace.yaml"]);
    commit(fixture.fixtureRoot, "add fixture workspace");
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(fs.readFileSync(artifactPath(fixture), "utf8"));
    assert.deepEqual(artifact.install.command, [
      "pnpm",
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
    ]);
    const invocation = JSON.parse(fs.readFileSync(fixture.invocationPath, "utf8"));
    assert.deepEqual(invocation.managerArgs, [
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
    ]);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare rejects a mismatched detected package manager version", () => {
  const fixture = setup();
  try {
    writeManager(fixture.runner, fixture.invocationPath, "9.0.0", successBody, {
      spec: "pnpm@10.0.0",
    });
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Detected pnpm version 9.0.0 does not match 10.0.0/);
    assert.equal(fs.existsSync(fixture.invocationPath), false);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare materializes and records an explicit generated baseline config", () => {
  const fixture = setup();
  try {
    const project = {
      ...fixture.project,
      typecheckPerformance: {
        ...fixture.project.typecheckPerformance,
        baseline: {
          tsconfig: ".generated/tsconfig.json",
          prepare: ["pnpm", "exec", "fixture", "prepare"],
        },
      },
    };
    writeJson(fixture.registryPath, { projects: [project] });
    git(fixture.fixtureRoot, ["add", "registry.json"]);
    commit(fixture.fixtureRoot, "configure generated baseline");
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      `if (process.argv[2] === "install") { ${successBody} } else process.exit(9);`,
      { spec: "pnpm@10.0.0" },
    );
    const failed = run(fixture);
    assert.equal(failed.status, 1);
    assert.match(failed.stderr, /baseline prepare exited with status 9/);
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      `if (process.argv[2] === "install") { ${successBody} } else { fs.mkdirSync(".generated", { recursive: true }); fs.writeFileSync(".generated/tsconfig.json", "{}\\n"); process.stdout.write("prepared"); }`,
      { spec: "pnpm@10.0.0" },
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(fs.readFileSync(artifactPath(fixture), "utf8"));
    assert.deepEqual(artifact.baselinePrepare.command, ["pnpm", "exec", "fixture", "prepare"]);
    assert.equal(artifact.baselinePrepare.exitCode, 0);
    assert.match(artifact.baselinePrepare.stdoutSha256, /^[0-9a-f]{64}$/);
    assert.equal(fs.existsSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")), true);
    const invocation = JSON.parse(fs.readFileSync(fixture.invocationPath, "utf8"));
    assert.equal(invocation.command, "corepack");
    assert.deepEqual(invocation.args, ["pnpm@10.0.0", "exec", "fixture", "prepare"]);
    assert.deepEqual(invocation.managerArgs, ["exec", "fixture", "prepare"]);
    assert.equal(invocation.env.corepackProjectSpec, "0");
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare rejects an unavailable pinned package manager", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(fixture.runner, "#!/bin/sh\nexit 12\n");
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /pnpm@10\.0\.0 is not runnable/);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare rejects lockfile and tracked-source mutations", () => {
  for (const [body, message] of [
    [`fs.appendFileSync("pnpm-lock.yaml", "changed");`, /modified frozen lockfile/],
    [`fs.appendFileSync("package.json", " ");`, /tracked source changed/],
  ] as const) {
    const fixture = setup();
    try {
      writeManager(fixture.runner, fixture.invocationPath, "10.0.0", body, {
        spec: "pnpm@10.0.0",
      });
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(fs.existsSync(artifactPath(fixture)), false);
    } finally {
      cleanup(fixture);
    }
  }
});

test("dependency prepare rejects pre-existing tracked-source changes", () => {
  const fixture = setup();
  try {
    fs.appendFileSync(path.join(fixture.fixtureRoot, "package.json"), " ");
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /tracked source changed before dependency installation/);
    assert.equal(fs.existsSync(fixture.invocationPath), false);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare rejects failed and timed-out installs", () => {
  for (const [body, args, message] of [
    ["process.exit(9);", [], /exited with status 9/],
    [
      "Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 500);",
      ["--timeout-ms", "50"],
      /failed to run/,
    ],
  ] as const) {
    const fixture = setup();
    try {
      writeManager(fixture.runner, fixture.invocationPath, "10.0.0", body, {
        spec: "pnpm@10.0.0",
      });
      const result = run(fixture, [...args]);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(fs.existsSync(artifactPath(fixture)), false);
    } finally {
      cleanup(fixture);
    }
  }
});

test("dependency prepare skips empty typecheck shards", () => {
  const fixture = setup();
  try {
    writeJson(fixture.registryPath, {
      projects: [{ ...fixture.project, typecheckPerformance: { enabled: false } }],
    });
    const empty = run(fixture);
    assert.equal(empty.status, 0, empty.stderr);
    assert.match(empty.stdout, /No typecheck performance projects selected/);
    assert.equal(fs.existsSync(fixture.invocationPath), false);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare shards the full fixture registry before filtering typecheck targets", () => {
  const fixture = setup();
  try {
    writeJson(fixture.registryPath, {
      projects: [
        { ...fixture.project, id: "padding", typecheckPerformance: { enabled: false } },
        fixture.project,
      ],
    });
    git(fixture.fixtureRoot, ["add", "registry.json"]);
    commit(fixture.fixtureRoot, "align typecheck shard selection");

    const empty = run(fixture, ["--shard-index", "0", "--shard-count", "2"]);
    assert.equal(empty.status, 0, empty.stderr);
    assert.match(empty.stdout, /No typecheck performance projects selected/);
    assert.equal(fs.existsSync(fixture.invocationPath), false);
    assert.equal(fs.existsSync(artifactPath(fixture)), false);

    const selected = run(fixture, ["--shard-index", "1", "--shard-count", "2"]);
    assert.equal(selected.status, 0, selected.stderr);
    assert.equal(fs.existsSync(artifactPath(fixture)), true);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare prepares every selected typecheck target", () => {
  const selectedFixture = setup();
  try {
    writeJson(selectedFixture.registryPath, {
      projects: [selectedFixture.project, { ...selectedFixture.project, id: "second" }],
    });
    git(selectedFixture.fixtureRoot, ["add", "registry.json"]);
    commit(selectedFixture.fixtureRoot, "select second target");
    const selected = run(selectedFixture);
    assert.equal(selected.status, 0, selected.stderr);
    assert.equal(fs.existsSync(artifactPath(selectedFixture)), true);
    assert.equal(
      fs.existsSync(path.join(selectedFixture.outputDir, "second-typecheck-dependencies.json")),
      true,
    );
  } finally {
    cleanup(selectedFixture);
  }
});

test("dependency prepare requires exact SHA evidence for selected targets", () => {
  const fixture = setup();
  try {
    writeJson(fixture.registryPath, { projects: [fixture.project] });
    const malformedSha = spawnSync(
      "rust-script",
      [script, "--registry", fixture.registryPath, "--output-dir", fixture.outputDir],
      { cwd: root, encoding: "utf8", env: { ...process.env, GITHUB_SHA: "main" } },
    );
    assert.equal(malformedSha.status, 1);
    assert.match(malformedSha.stderr, /GITHUB_SHA must be a full lowercase commit SHA/);
  } finally {
    cleanup(fixture);
  }
});
