// Public fresh-project acceptance for #3956. The orchestration is read from the
// selected source, but every product process and Vize dependency comes from npm.
import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

import { collectOutput, runMain, type CollectedOutput } from "../command_io.ts";
import {
  APP,
  assertCheckoutIsolation,
  BROKEN_APP,
  PLANNED_DEPENDENCIES,
  sha256,
  sourceSnapshot,
  writeJson,
  writeProject,
} from "./fresh_project_fixture.ts";
import { watchDiagnostics } from "./lsp_client.ts";

const VERSION = /^\d+\.\d+\.\d+$/u;
const REGISTRY = "https://registry.npmjs.org/";
const INIT_ARGS = ["--yes", "--no-lint", "--vite", "--fmt", "--typecheck", "--editor"];

interface LockPackage {
  name?: unknown;
  version?: unknown;
  resolved?: unknown;
}

interface PackageLock {
  packages?: Record<string, LockPackage>;
}

async function command(
  label: string,
  executable: string,
  args: readonly string[],
  cwd: string,
): Promise<CollectedOutput> {
  const outcome = await collectOutput(executable, [...args], { cwd });
  if (outcome.status !== 0) {
    throw new Error(
      `${label} exited ${outcome.status}\nstdout:\n${outcome.stdout}\nstderr:\n${outcome.stderr}`,
    );
  }
  return outcome;
}

function npmCommand(label: string, args: readonly string[], cwd: string): Promise<CollectedOutput> {
  if (process.platform !== "win32") return command(label, "npm", args, cwd);
  const cli = join(dirname(process.execPath), "node_modules", "npm", "bin", "npm-cli.js");
  return command(label, process.execPath, [cli, ...args], cwd);
}

function linuxLibc(): "gnu" | "musl" {
  const report = process.report?.getReport();
  const header = report?.header as { glibcVersionRuntime?: unknown } | undefined;
  return typeof header?.glibcVersionRuntime === "string" ? "gnu" : "musl";
}

/** Public optional package that the current process must select. */
export function expectedNativePackage(
  platform = process.platform,
  arch = process.arch,
  libc: "gnu" | "musl" = linuxLibc(),
): string {
  assert.ok(arch === "x64" || arch === "arm64", `unsupported public smoke arch: ${arch}`);
  if (platform === "darwin") return `@vizejs/native-darwin-${arch}`;
  if (platform === "win32") return `@vizejs/native-win32-${arch}-msvc`;
  if (platform === "linux") return `@vizejs/native-linux-${arch}-${libc}`;
  throw new Error(`unsupported public smoke platform: ${platform}`);
}

async function nativeIdentity(project: string, version: string): Promise<Record<string, string>> {
  const expected = expectedNativePackage();
  const probe = await command(
    "native identity probe",
    process.execPath,
    [
      "-e",
      [
        'const assert = require("node:assert/strict");',
        'const { createRequire } = require("node:module");',
        "const expected = process.argv[1];",
        'const fromVize = createRequire(require.resolve("vize"));',
        'const native = fromVize("@vizejs/native");',
        'const fromNative = createRequire(fromVize.resolve("@vizejs/native"));',
        "const selected = fromNative(expected);",
        'assert.strictEqual(native, selected, expected + " was installed but not loaded");',
        "process.stdout.write(JSON.stringify({ package: expected, token: native.getNativeBuildToken() }));",
      ].join("\n"),
      expected,
    ],
    project,
  );
  const identity = JSON.parse(probe.stdout) as Record<string, string>;
  assert.equal(identity.package, expected);
  assert.match(identity.token, /^[0-9a-f]{32}$/u);
  const manifest = JSON.parse(
    readFileSync(join(project, "node_modules", ...expected.split("/"), "package.json"), "utf8"),
  ) as { version?: unknown };
  assert.equal(manifest.version, version, `${expected} version differs`);
  return identity;
}

function registryPackages(project: string, version: string): Record<string, string> {
  const lock = JSON.parse(readFileSync(join(project, "package-lock.json"), "utf8")) as PackageLock;
  const packages: Record<string, string> = {};
  for (const [location, entry] of Object.entries(lock.packages ?? {})) {
    const installed = location.slice(
      location.lastIndexOf("node_modules/") + "node_modules/".length,
    );
    const segments = installed.split("/");
    const inferred = segments[0]?.startsWith("@") ? segments.slice(0, 2).join("/") : segments[0];
    const name = typeof entry.name === "string" ? entry.name : inferred;
    if (name !== "vize" && !name?.startsWith("@vizejs/")) continue;
    assert.equal(entry.version, version, `${name} version differs`);
    assert.ok(
      typeof entry.resolved === "string" && entry.resolved.startsWith(REGISTRY),
      `${name} did not resolve from public npm`,
    );
    packages[name] = entry.resolved;
  }
  for (const required of [...PLANNED_DEPENDENCIES, "@vizejs/native", expectedNativePackage()])
    assert.ok(packages[required], `${required} is absent from the public lock`);
  return Object.fromEntries(Object.entries(packages).sort(([a], [b]) => a.localeCompare(b)));
}

export async function verifyPublishedFreshProject(
  version: string,
  evidence: string,
): Promise<void> {
  assert.match(version, VERSION, "published version must be X.Y.Z");
  const checkout = realpathSync(process.cwd());
  const temporary = mkdtempSync(join(tmpdir(), "vize-published-fresh-"));
  const runner = join(temporary, "runner");
  const project = join(temporary, "project");
  mkdirSync(runner);
  mkdirSync(project);
  try {
    writeJson(join(runner, "package.json"), { private: true });
    await npmCommand(
      "published CLI install",
      ["install", "--no-audit", "--no-fund", "--include=optional", `vize@${version}`],
      runner,
    );
    writeProject(project);
    await npmCommand(
      "fresh project bootstrap",
      ["install", "--no-audit", "--no-fund", "--include=optional"],
      project,
    );

    const runnerBin = join(runner, "node_modules", "vize", "bin", "vize");
    assert.equal(
      (
        await command("published CLI version", process.execPath, [runnerBin, "--version"], runner)
      ).stdout.trim(),
      `vize ${version}`,
    );
    const beforeDryRun = sourceSnapshot(project);
    const dryRun = await command(
      "published init dry run",
      process.execPath,
      [runnerBin, "init", project, ...INIT_ARGS, "--no-install", "--dry-run"],
      runner,
    );
    assert.deepEqual(sourceSnapshot(project), beforeDryRun, "init --dry-run changed the project");
    assert.match(dryRun.stdout, /bundler\s+configured adds vize\(\)/u);
    assert.match(dryRun.stdout, /typecheck configured/u);
    assert.match(dryRun.stdout, /editor\s+configured/u);

    const applied = await command(
      "published init",
      process.execPath,
      [runnerBin, "init", project, ...INIT_ARGS, "--no-install"],
      runner,
    );
    const plan = PLANNED_DEPENDENCIES.map((name) => `${name}@${version}`);
    await npmCommand(
      "generated dependency plan",
      [
        "install",
        "--save-dev",
        "--save-exact",
        "--no-audit",
        "--no-fund",
        "--include=optional",
        ...plan,
      ],
      project,
    );
    const beforeIdempotency = sourceSnapshot(project);
    const idempotent = await command(
      "published init idempotency",
      process.execPath,
      [runnerBin, "init", project, ...INIT_ARGS, "--no-install"],
      runner,
    );
    assert.deepEqual(sourceSnapshot(project), beforeIdempotency, "second init changed the project");

    const manifest = JSON.parse(readFileSync(join(project, "package.json"), "utf8")) as {
      scripts?: Record<string, string>;
    };
    assert.equal(manifest.scripts?.["vize:check"], "vize check");
    assert.match(readFileSync(join(project, "vite.config.ts"), "utf8"), /plugins: \[vize\(\)\]/u);
    assert.deepEqual(
      JSON.parse(readFileSync(join(project, ".vscode", "extensions.json"), "utf8")),
      {
        recommendations: ["ubugeeei.vize"],
      },
    );

    const projectBin = join(project, "node_modules", "vize", "bin", "vize");
    assert.equal(
      (
        await command(
          "project-local CLI version",
          process.execPath,
          [projectBin, "--version"],
          project,
        )
      ).stdout.trim(),
      `vize ${version}`,
    );
    const clean = await npmCommand("generated clean check", ["run", "vize:check"], project);
    writeFileSync(join(project, "src", "App.vue"), BROKEN_APP);
    const broken = await collectOutput(
      process.execPath,
      [projectBin, "check", "--format", "json"],
      {
        cwd: project,
      },
    );
    assert.notEqual(broken.status, 0, "broken project passed vize check");
    assert.match(broken.stdout, /TS2322/u, "broken project did not report its type error");
    writeFileSync(join(project, "src", "App.vue"), APP);
    const repaired = await npmCommand("generated repaired check", ["run", "vize:check"], project);
    const lsp = await watchDiagnostics({
      bin: projectBin,
      root: project,
      files: [join(project, "src", "App.vue")],
      version,
      settle: (latest) =>
        JSON.stringify([...latest.values()]).includes("vue/require-v-for-key") ? null : undefined,
    });
    assert.equal(lsp, null, lsp ?? "LSP diagnostics failed");

    const native = await nativeIdentity(project, version);
    const packages = registryPackages(project, version);
    assertCheckoutIsolation(checkout, project);
    mkdirSync(dirname(resolve(evidence)), { recursive: true });
    writeJson(resolve(evidence), {
      schema: "vize.publishedFreshProject",
      schemaVersion: 1,
      version,
      host: { platform: process.platform, arch: process.arch },
      native,
      plannedDependencies: plan,
      registryPackages: packages,
      init: {
        dryRunSha256: sha256(dryRun.stdout),
        applySha256: sha256(applied.stdout),
        idempotentSha256: sha256(idempotent.stdout),
      },
      checks: {
        clean: clean.status,
        broken: broken.status,
        repaired: repaired.status,
        lsp: "passed",
      },
      checkoutIsolated: true,
    });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  runMain(async () => {
    const [version] = process.argv.slice(2);
    const evidence = process.env.VIZE_PUBLISHED_FRESH_EVIDENCE;
    assert.ok(version, "Usage: fresh_project.ts <version>");
    assert.ok(evidence, "VIZE_PUBLISHED_FRESH_EVIDENCE is required");
    try {
      await verifyPublishedFreshProject(version, evidence);
    } catch (error) {
      mkdirSync(dirname(resolve(evidence)), { recursive: true });
      writeJson(resolve(evidence), {
        schema: "vize.publishedFreshProject",
        schemaVersion: 1,
        version,
        host: { platform: process.platform, arch: process.arch },
        failure: error instanceof Error ? (error.stack ?? error.message) : String(error),
      });
      throw error;
    }
    return existsSync(evidence) ? 0 : 1;
  });
}
