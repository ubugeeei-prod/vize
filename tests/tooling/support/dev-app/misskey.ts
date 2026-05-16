import * as fs from "node:fs";
import * as path from "node:path";
import {
  commandAvailable,
  ensureFileContent,
  formatCommandFailure,
  run,
  sleep,
  spawnForOutput,
} from "./commands.ts";
import type { MisskeyBeforeStartSteps } from "./types.ts";

type MisskeyRuntimeInfo = {
  version: string;
  modules: string;
};

type MisskeyRuntimeMarker = MisskeyRuntimeInfo & {
  validatedWith: "backend-re2";
};

export function getMisskeyPnpmCommand(
  _misskeyRoot: string,
  args: string[],
): {
  command: string;
  args: string[];
} {
  return {
    command: "pnpm",
    args,
  };
}

function runMisskeyPnpm(misskeyRoot: string, args: string[], env?: Record<string, string>): void {
  const command = getMisskeyPnpmCommand(misskeyRoot, args);
  run(command.command, command.args, misskeyRoot, env);
}

function spawnMisskeyPnpm(misskeyRoot: string, args: string[], env?: Record<string, string>) {
  const command = getMisskeyPnpmCommand(misskeyRoot, args);
  return spawnForOutput(command.command, command.args, misskeyRoot, env);
}

function ensureMisskeyDockerEnv(misskeyRoot: string): void {
  const dockerEnv = path.join(misskeyRoot, ".config", "docker.env");
  if (fs.existsSync(dockerEnv)) {
    return;
  }

  fs.copyFileSync(path.join(misskeyRoot, ".config", "docker_example.env"), dockerEnv);
}

/**
 * Builds the minimal Misskey config file needed for a local Vize dev session.
 *
 * The fixture owns PostgreSQL and Redis through Docker Compose, so the generated
 * config pins loopback services and a deterministic `pidFile`. The `id: aidx`
 * setting mirrors Misskey's supported ID generation method and is intentionally
 * covered by tests because invalid IDs make the backend fail late during boot.
 */
export function buildMisskeyDevConfig(port: number, pidFile: string): string {
  return [
    `url: http://127.0.0.1:${port}`,
    `port: ${port}`,
    "id: aidx",
    `pidFile: ${pidFile}`,
    "setupPassword: dev-password",
    "",
    "db:",
    "  host: 127.0.0.1",
    "  port: 5432",
    "  db: misskey",
    "  user: example-misskey-user",
    "  pass: example-misskey-pass",
    "",
    "redis:",
    "  host: 127.0.0.1",
    "  port: 6379",
    '  pass: ""',
    "",
  ].join("\n");
}

export function ensureMisskeyDevConfig(misskeyRoot: string, port: number): string {
  const configPath = path.join(misskeyRoot, ".config", "vize-dev.yml");
  const pidFile = path.join(misskeyRoot, ".config", "vize-dev.pid");
  ensureFileContent(configPath, buildMisskeyDevConfig(port, pidFile));
  return "vize-dev.yml";
}

function ensureMisskeyLocalServicesStarted(misskeyRoot: string): void {
  if (!commandAvailable("docker", ["compose", "version"])) {
    throw new Error("docker compose is required to start misskey. Install Docker first.");
  }

  ensureMisskeyDockerEnv(misskeyRoot);
  run("docker", ["compose", "-f", "compose.local-db.yml", "up", "-d"], misskeyRoot);
}

function ensureMisskeyLocalServicesReady(misskeyRoot: string, configName: string): void {
  const env = { MISSKEY_CONFIG_YML: configName };
  let lastFailure = "";

  for (let attempt = 0; attempt < 30; attempt += 1) {
    const check = spawnMisskeyPnpm(misskeyRoot, ["check:connect"], env);
    if (check.status === 0) {
      return;
    }

    lastFailure = formatCommandFailure(check.stdout, check.stderr);
    sleep(2_000);
  }

  if (lastFailure) {
    throw new Error(`misskey middleware did not become ready.\n${lastFailure}`);
  }

  throw new Error("misskey middleware did not become ready. Check PostgreSQL and Redis logs.");
}

function ensureMisskeyBackendBuilt(misskeyRoot: string, configName: string): void {
  const env = { MISSKEY_CONFIG_YML: configName };
  runMisskeyPnpm(misskeyRoot, ["build-pre"], env);
  runMisskeyPnpm(misskeyRoot, ["--filter", "backend...", "build"], env);
}

function getMisskeyRuntimeInfo(misskeyRoot: string): MisskeyRuntimeInfo {
  const runtime = spawnMisskeyPnpm(misskeyRoot, [
    "exec",
    "node",
    "-p",
    "JSON.stringify({ version: process.version, modules: process.versions.modules })",
  ]);

  if (runtime.status !== 0) {
    const message = formatCommandFailure(runtime.stdout, runtime.stderr);
    throw new Error(`Failed to resolve misskey runtime.\n${message}`);
  }

  return JSON.parse(runtime.stdout) as MisskeyRuntimeInfo;
}

function readMisskeyRuntimeMarker(markerPath: string): MisskeyRuntimeMarker | null {
  if (!fs.existsSync(markerPath)) {
    return null;
  }

  try {
    const marker = JSON.parse(
      fs.readFileSync(markerPath, "utf-8"),
    ) as Partial<MisskeyRuntimeMarker>;
    if (marker.validatedWith !== "backend-re2") {
      return null;
    }
    return marker as MisskeyRuntimeMarker;
  } catch {
    return null;
  }
}

function probeMisskeyNativeDependencies(misskeyRoot: string) {
  return spawnMisskeyPnpm(misskeyRoot, [
    "exec",
    "node",
    "-e",
    "const mod = require.resolve('re2', { paths: ['./packages/backend'] }); require(mod);",
  ]);
}

/**
 * Ensures Misskey's native backend dependency is ABI-compatible with the Node
 * runtime selected by the fixture package manager.
 *
 * Misskey can keep an old `re2` build artifact around after Node changes. The
 * marker file records the exact Node version and module ABI that successfully
 * loaded the backend dependency, so repeated dev launches skip the rebuild when
 * the runtime has not changed.
 */
function ensureMisskeyNativeDependencies(misskeyRoot: string): void {
  const runtimeInfo = getMisskeyRuntimeInfo(misskeyRoot);
  const markerPath = path.join(misskeyRoot, ".config", "vize-dev-runtime.json");
  const currentMarker = readMisskeyRuntimeMarker(markerPath);

  if (
    currentMarker?.version === runtimeInfo.version &&
    currentMarker.modules === runtimeInfo.modules
  ) {
    return;
  }

  const probe = probeMisskeyNativeDependencies(misskeyRoot);

  if (probe.status !== 0) {
    runMisskeyPnpm(misskeyRoot, ["--dir", "packages/backend", "rebuild", "re2"]);
    const retry = probeMisskeyNativeDependencies(misskeyRoot);
    if (retry.status !== 0) {
      const message = formatCommandFailure(retry.stdout, retry.stderr);
      throw new Error(`Failed to load misskey native dependency re2.\n${message}`);
    }
  }

  const nextMarker: MisskeyRuntimeMarker = {
    ...runtimeInfo,
    validatedWith: "backend-re2",
  };
  ensureFileContent(markerPath, `${JSON.stringify(nextMarker, null, 2)}\n`);
}

function ensureMisskeyMigrated(misskeyRoot: string, configName: string): void {
  const env = { MISSKEY_CONFIG_YML: configName };
  runMisskeyPnpm(misskeyRoot, ["migrate"], env);
}

/**
 * Runs the ordered Misskey preflight sequence before the foreground dev server.
 *
 * The order is important: services must be started before backend builds can
 * validate connectivity, native dependencies need the final package-manager
 * runtime, readiness should be checked before migrations, and only then is the
 * dev server allowed to take over the terminal.
 */
export function runMisskeyBeforeStart(
  misskeyRoot: string,
  configName: string,
  steps: MisskeyBeforeStartSteps = {
    startLocalServices: ensureMisskeyLocalServicesStarted,
    ensureBackendBuilt: ensureMisskeyBackendBuilt,
    ensureNativeDependencies: ensureMisskeyNativeDependencies,
    waitForLocalServices: ensureMisskeyLocalServicesReady,
    ensureMigrated: ensureMisskeyMigrated,
  },
): void {
  steps.startLocalServices(misskeyRoot);
  steps.ensureBackendBuilt(misskeyRoot, configName);
  steps.ensureNativeDependencies(misskeyRoot);
  steps.waitForLocalServices(misskeyRoot, configName);
  steps.ensureMigrated(misskeyRoot, configName);
}
