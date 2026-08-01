import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const script = path.join(root, "tools", "fixtures", "typecheck-dependency-prepare.mjs");
const commitSha = "a".repeat(40);
const packageManagers = [
  {
    name: "npm",
    version: "11.9.0",
    lockfile: "package-lock.json",
    lockfileContents: '{"lockfileVersion":3}\n',
    installArgs: ["ci", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund"],
  },
  {
    name: "pnpm",
    version: "10.0.0",
    lockfile: "pnpm-lock.yaml",
    lockfileContents: "lockfileVersion: '9.0'\n",
    installArgs: ["install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
  },
  {
    name: "yarn",
    version: "4.9.2",
    lockfile: "yarn.lock",
    lockfileContents: "__metadata:\n  version: 8\n",
    installArgs: ["install", "--immutable", "--mode=skip-build"],
  },
] as const;

function setup(packageManager: (typeof packageManagers)[number] = packageManagers[1]) {
  const fixtureRoot = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "typecheck-dependencies-"),
  );
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-dependencies-out-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-dependencies-manager-"));
  const fixturePath = path.relative(root, fixtureRoot);
  const project = {
    id: "fixture",
    fixturePath,
    revision: "b".repeat(40),
    vueGlobs: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    typecheckPerformance: {
      enabled: true,
      compareTo: "vue-tsc",
      packageManager: packageManager.name,
      packageManagerVersion: packageManager.version,
      lockfile: packageManager.lockfile,
      hangTimeoutMs: 5_000,
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
    },
  };
  fs.mkdirSync(path.join(fixtureRoot, "src"));
  fs.writeFileSync(path.join(fixtureRoot, "src", "App.vue"), "<template />\n");
  fs.writeFileSync(path.join(fixtureRoot, "tsconfig.json"), "{}\n");
  fs.writeFileSync(path.join(fixtureRoot, "package.json"), '{"name":"fixture"}\n');
  fs.writeFileSync(
    path.join(fixtureRoot, packageManager.lockfile),
    packageManager.lockfileContents,
  );
  const registryPath = path.join(fixtureRoot, "registry.json");
  writeJson(registryPath, { projects: [project] });
  git(fixtureRoot, ["init", "-q"]);
  git(fixtureRoot, ["add", "."]);
  commit(fixtureRoot, "fixture");
  const invocationPath = path.join(fakeDir, "invocation.json");
  const manager = path.join(fakeDir, packageManager.name);
  writeManager(manager, invocationPath, packageManager.version, successBody);
  return {
    fixtureRoot,
    outputDir,
    fakeDir,
    registryPath,
    invocationPath,
    manager,
    packageManager,
    project,
  };
}

const successBody = `fs.mkdirSync("node_modules", { recursive: true }); fs.writeFileSync("node_modules/installed", "yes"); process.stdout.write("installed");`;

function writeManager(
  pathname: string,
  invocationPath: string,
  version: string,
  installBody: string,
) {
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nimport fs from "node:fs";\nif (process.argv.includes("--version")) { console.log(${JSON.stringify(version)}); process.exit(0); }\nfs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), args: process.argv.slice(2), env: { CI: process.env.CI, npm: process.env.npm_config_ignore_scripts, yarn: process.env.YARN_ENABLE_SCRIPTS } }));\n${installBody}\n`,
  );
  fs.chmodSync(pathname, 0o755);
}

function run(fixture: ReturnType<typeof setup>, extraArgs: string[] = []) {
  return spawnSync(
    process.execPath,
    [script, "--registry", fixture.registryPath, "--output-dir", fixture.outputDir, ...extraArgs],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_SHA: commitSha,
        PATH: `${fixture.fakeDir}${path.delimiter}${process.env.PATH}`,
      },
    },
  );
}

function git(cwd: string, args: string[]) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
}

// CI runners have no git identity, so every commit has to carry its own.
function commit(cwd: string, message: string) {
  git(cwd, [
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.com",
    "commit",
    "-qm",
    message,
  ]);
}

function artifactPath(fixture: ReturnType<typeof setup>) {
  return path.join(fixture.outputDir, "fixture-typecheck-dependencies.json");
}

function writeJson(pathname: string, value: unknown) {
  fs.writeFileSync(pathname, `${JSON.stringify(value, null, 2)}\n`);
}

function cleanup(fixture: ReturnType<typeof setup>) {
  fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true });
  fs.rmSync(fixture.outputDir, { recursive: true, force: true });
  fs.rmSync(fixture.fakeDir, { recursive: true, force: true });
}

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
        args: packageManager.installArgs,
        env: { CI: "true", npm: "true", yarn: "false" },
      });
    } finally {
      cleanup(fixture);
    }
  }
});

test("dependency prepare rejects a mismatched detected package manager version", () => {
  const fixture = setup();
  try {
    writeManager(fixture.manager, fixture.invocationPath, "9.0.0", successBody);
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
      fixture.manager,
      fixture.invocationPath,
      "10.0.0",
      `if (process.argv[2] === "install") { ${successBody} } else process.exit(9);`,
    );
    const failed = run(fixture);
    assert.equal(failed.status, 1);
    assert.match(failed.stderr, /baseline prepare exited with status 9/);
    writeManager(
      fixture.manager,
      fixture.invocationPath,
      "10.0.0",
      `if (process.argv[2] === "install") { ${successBody} } else { fs.mkdirSync(".generated", { recursive: true }); fs.writeFileSync(".generated/tsconfig.json", "{}\\n"); process.stdout.write("prepared"); }`,
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(fs.readFileSync(artifactPath(fixture), "utf8"));
    assert.deepEqual(artifact.baselinePrepare.command, ["pnpm", "exec", "fixture", "prepare"]);
    assert.equal(artifact.baselinePrepare.exitCode, 0);
    assert.match(artifact.baselinePrepare.stdoutSha256, /^[0-9a-f]{64}$/);
    assert.equal(fs.existsSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")), true);
  } finally {
    cleanup(fixture);
  }
});

test("dependency prepare rejects an unavailable pinned package manager", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(fixture.manager, "#!/bin/sh\nexit 12\n");
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /pnpm is not runnable/);
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
      writeManager(fixture.manager, fixture.invocationPath, "10.0.0", body);
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
      writeManager(fixture.manager, fixture.invocationPath, "10.0.0", body);
      const result = run(fixture, [...args]);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(fs.existsSync(artifactPath(fixture)), false);
    } finally {
      cleanup(fixture);
    }
  }
});

test("dependency prepare requires one performance project and exact SHA evidence", () => {
  const fixture = setup();
  try {
    writeJson(fixture.registryPath, { projects: [] });
    const missing = run(fixture);
    assert.equal(missing.status, 1);
    assert.match(missing.stderr, /Expected exactly one.*found 0/);
    writeJson(fixture.registryPath, {
      projects: [fixture.project, { ...fixture.project, id: "duplicate" }],
    });
    const duplicate = run(fixture);
    assert.equal(duplicate.status, 1);
    assert.match(duplicate.stderr, /Expected exactly one.*found 2/);
    writeJson(fixture.registryPath, { projects: [fixture.project] });
    const malformedSha = spawnSync(
      process.execPath,
      [script, "--registry", fixture.registryPath, "--output-dir", fixture.outputDir],
      { cwd: root, encoding: "utf8", env: { ...process.env, GITHUB_SHA: "main" } },
    );
    assert.equal(malformedSha.status, 1);
    assert.match(malformedSha.stderr, /GITHUB_SHA must be a full lowercase commit SHA/);
  } finally {
    cleanup(fixture);
  }
});
