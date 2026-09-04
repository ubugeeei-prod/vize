import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const script = path.join(
  root,
  "tools",
  "commands",
  "fixtures",
  "typecheck-dependency-prepare.rs",
);
export const commitSha = "a".repeat(40);
export const packageManagers = [
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
    installArgs: [
      "install",
      "--frozen-lockfile",
      "--ignore-scripts",
      "--prefer-offline",
      "--ignore-workspace",
    ],
  },
  {
    name: "yarn",
    version: "4.9.2",
    lockfile: "yarn.lock",
    lockfileContents: "__metadata:\n  version: 8\n",
    installArgs: ["install", "--immutable", "--mode=skip-build"],
  },
] as const;

export const successBody = `fs.mkdirSync("node_modules", { recursive: true }); fs.writeFileSync("node_modules/installed", "yes"); process.stdout.write("installed");`;

type PackageManager = (typeof packageManagers)[number];

export function setup(packageManager: PackageManager = packageManagers[1]) {
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
  const corepack = path.join(fakeDir, "corepack");
  const runner = packageManager.name === "npm" ? manager : corepack;
  writeManager(manager, invocationPath, packageManager.version, successBody);
  writeManager(corepack, invocationPath, packageManager.version, successBody, {
    spec: `${packageManager.name}@${packageManager.version}`,
  });
  return {
    corepack,
    fixtureRoot,
    outputDir,
    fakeDir,
    registryPath,
    invocationPath,
    manager,
    packageManager,
    project,
    runner,
  };
}

export function writeManager(
  pathname: string,
  invocationPath: string,
  version: string,
  installBody: string,
  options: { spec?: string } = {},
) {
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node
import fs from "node:fs";
const rawArgs = process.argv.slice(2);
const expectedSpec = ${JSON.stringify(options.spec ?? null)};
let managerArgs = rawArgs;
if (expectedSpec !== null) {
  if (rawArgs[0] !== expectedSpec) {
    console.error(\`expected \${expectedSpec}, received \${rawArgs[0] ?? "<missing>"}\`);
    process.exit(13);
  }
  managerArgs = rawArgs.slice(1);
}
if (managerArgs.includes("--version")) {
  console.log(${JSON.stringify(version)});
  process.exit(0);
}
const command = process.argv[1].split(/[\\\\/]/u).pop();
fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), command, args: rawArgs, managerArgs, env: { CI: process.env.CI, npm: process.env.npm_config_ignore_scripts, yarn: process.env.YARN_ENABLE_SCRIPTS, corepackProjectSpec: process.env.COREPACK_ENABLE_PROJECT_SPEC } }));
process.argv = [process.argv[0], process.argv[1], ...managerArgs];
${installBody}
`,
  );
  fs.chmodSync(pathname, 0o755);
}

export function expectedInvocationArgs(packageManager: PackageManager) {
  return packageManager.name === "npm"
    ? packageManager.installArgs
    : [`${packageManager.name}@${packageManager.version}`, ...packageManager.installArgs];
}

export function expectedInvocationCommand(packageManager: PackageManager) {
  return packageManager.name === "npm" ? packageManager.name : "corepack";
}

export function expectedInvocationEnv(packageManager: PackageManager) {
  return {
    CI: "true",
    npm: "true",
    yarn: "false",
    ...(packageManager.name === "npm" ? {} : { corepackProjectSpec: "0" }),
  };
}

export function run(
  fixture: ReturnType<typeof setup>,
  extraArgs: string[] = [],
  options: { timeoutMs?: number } = {},
) {
  return spawnSync(
    "rust-script",
    [script, "--registry", fixture.registryPath, "--output-dir", fixture.outputDir, ...extraArgs],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_SHA: commitSha,
        PATH: `${fixture.fakeDir}${path.delimiter}${process.env.PATH}`,
      },
      timeout: options.timeoutMs,
    },
  );
}

export function git(cwd: string, args: string[]) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
}

// CI runners have no git identity, so every commit has to carry its own.
export function commit(cwd: string, message: string) {
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

export function artifactPath(fixture: ReturnType<typeof setup>) {
  return path.join(fixture.outputDir, "fixture-typecheck-dependencies.json");
}

export function writeJson(pathname: string, value: unknown) {
  fs.writeFileSync(pathname, `${JSON.stringify(value, null, 2)}\n`);
}

export function cleanup(fixture: ReturnType<typeof setup>) {
  fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true });
  fs.rmSync(fixture.outputDir, { recursive: true, force: true });
  fs.rmSync(fixture.fakeDir, { recursive: true, force: true });
}
