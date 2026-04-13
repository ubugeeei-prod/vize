import { execFileSync, spawn, spawnSync } from "node:child_process";
import * as fs from "node:fs";
import { createServer } from "node:net";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import process from "node:process";
import { elkApp, misskeyApp, npmxApp, vuefesApp, type AppConfig } from "../tests/_helpers/apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const VITE_PLUS_BIN = `${process.env.HOME ?? ""}/.vite-plus/bin`;
const MISE_BIN = `${process.env.HOME ?? ""}/.local/bin/mise`;
const BASE_ENV = {
  ...process.env,
  PATH: `${VITE_PLUS_BIN}:${process.env.PATH ?? ""}`,
};

type Target = "playground" | "misskey" | "npmx" | "elk" | "vuefes";

type LaunchConfig = {
  target: Target;
  url: string;
  setup?: () => void;
  beforeStart?: () => void;
  cwd: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
};

function replacePortInArgs(args: string[], nextPort: number): string[] {
  const nextArgs = [...args];
  const portFlagIndex = nextArgs.findIndex((arg) => arg === "--port" || arg === "-p");
  if (portFlagIndex >= 0 && portFlagIndex + 1 < nextArgs.length) {
    nextArgs[portFlagIndex + 1] = String(nextPort);
  }
  return nextArgs;
}

function replacePortInUrl(url: string, currentPort: number, nextPort: number): string {
  return url.replace(`:${currentPort}`, `:${nextPort}`);
}

type MisskeyBeforeStartSteps = {
  startLocalServices: (misskeyRoot: string) => void;
  ensureBackendBuilt: (misskeyRoot: string, configName: string) => void;
  ensureNativeDependencies: (misskeyRoot: string) => void;
  waitForLocalServices: (misskeyRoot: string, configName: string) => void;
  ensureMigrated: (misskeyRoot: string, configName: string) => void;
};

const target = normalizeTarget(process.env.usage_target);
const skipSetup = process.env.usage_skip_setup === "true";
const skipBuild = process.env.usage_skip_build === "true";

function normalizeTarget(value: string | undefined): Target {
  switch (value ?? "playground") {
    case "playground":
    case "misskey":
    case "npmx":
    case "elk":
    case "vuefes":
      return value ?? "playground";
    default:
      throw new Error(
        `Unsupported target "${value}". Expected one of: playground, misskey, npmx, elk, vuefes.`,
      );
  }
}

function run(command: string, args: string[], cwd = REPO_ROOT, env?: Record<string, string>): void {
  console.log(`$ ${command} ${args.join(" ")}`);
  execFileSync(command, args, {
    cwd,
    env: {
      ...BASE_ENV,
      ...env,
    },
    stdio: "inherit",
  });
}

function commandAvailable(command: string, args: string[] = ["--version"]): boolean {
  const result = spawnSync(command, args, {
    cwd: REPO_ROOT,
    env: BASE_ENV,
    stdio: "ignore",
  });
  return result.status === 0;
}

function hasListeningProcessOnPort(port: number): boolean {
  if (!commandAvailable("lsof", ["-v"])) {
    return false;
  }

  const result = spawnSync("lsof", ["-nP", `-iTCP:${port}`, "-sTCP:LISTEN"], {
    cwd: REPO_ROOT,
    env: BASE_ENV,
    stdio: "ignore",
  });
  return result.status === 0;
}

const misskeyNodeVersionCache = new Map<string, string>();

function getMisskeyNodeVersion(misskeyRoot: string): string {
  const cached = misskeyNodeVersionCache.get(misskeyRoot);
  if (cached != null) {
    return cached;
  }

  const versionFile = path.join(misskeyRoot, ".node-version");
  if (!fs.existsSync(versionFile)) {
    throw new Error(`Missing misskey node version file: ${versionFile}`);
  }

  const version = fs.readFileSync(versionFile, "utf-8").trim();
  if (version.length === 0) {
    throw new Error(`Misskey node version file is empty: ${versionFile}`);
  }

  misskeyNodeVersionCache.set(misskeyRoot, version);
  return version;
}

function resolveMisskeyCommandRoot(misskeyRoot: string): string {
  const versionFile = path.join(misskeyRoot, ".node-version");
  if (fs.existsSync(versionFile)) {
    return misskeyRoot;
  }

  const sourceFixtureRoot = path.join(REPO_ROOT, "tests", "_fixtures", "_git", "misskey");
  if (fs.existsSync(path.join(sourceFixtureRoot, ".node-version"))) {
    return sourceFixtureRoot;
  }

  return misskeyRoot;
}

function getMisskeyPnpmCommand(
  misskeyRoot: string,
  args: string[],
): {
  command: string;
  args: string[];
} {
  if (!fs.existsSync(MISE_BIN)) {
    throw new Error(`mise is required to start misskey: ${MISE_BIN}`);
  }

  return {
    command: MISE_BIN,
    args: ["exec", `node@${getMisskeyNodeVersion(misskeyRoot)}`, "--", "pnpm", ...args],
  };
}

function runMisskeyPnpm(misskeyRoot: string, args: string[], env?: Record<string, string>): void {
  const command = getMisskeyPnpmCommand(misskeyRoot, args);
  run(command.command, command.args, misskeyRoot, env);
}

function spawnMisskeyPnpm(
  misskeyRoot: string,
  args: string[],
  env?: Record<string, string>,
): ReturnType<typeof spawnSync<string>> {
  const command = getMisskeyPnpmCommand(misskeyRoot, args);
  return spawnSync(command.command, command.args, {
    cwd: misskeyRoot,
    env: {
      ...BASE_ENV,
      ...env,
    },
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function ensureBuildCommonVite(): void {
  run("pnpm", ["-C", "npm/vize-native", "build"]);
  run("wasm-pack", [
    "build",
    "crates/vize_vitrine",
    "--target",
    "nodejs",
    "--out-dir",
    "../../npm/vite-plugin-vize/wasm",
    "--features",
    "wasm",
    "--no-default-features",
  ]);
  run("pnpm", ["-C", "npm/vize", "build"]);
  run("pnpm", ["-C", "npm/vite-plugin-vize", "build"]);
}

function ensureBuildNuxtStack(): void {
  ensureBuildCommonVite();
  run("pnpm", ["-C", "npm/vite-plugin-musea", "build"]);
  run("pnpm", ["-C", "npm/musea-nuxt", "build"]);
  run("pnpm", ["-C", "npm/nuxt", "build"]);
}

function ensureBuildPlayground(): void {
  ensureBuildCommonVite();
  run("pnpm", ["-C", "playground", "build:wasm"]);
}

function ensureFileContent(filePath: string, content: string): void {
  const current = fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf-8") : null;
  if (current === content) {
    return;
  }

  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function ensureMisskeyDockerEnv(misskeyRoot: string): void {
  const dockerEnv = path.join(misskeyRoot, ".config", "docker.env");
  if (fs.existsSync(dockerEnv)) {
    return;
  }

  fs.copyFileSync(path.join(misskeyRoot, ".config", "docker_example.env"), dockerEnv);
}

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

function ensureMisskeyDevConfig(misskeyRoot: string, port: number): string {
  const configPath = path.join(misskeyRoot, ".config", "vize-dev.yml");
  const pidFile = path.join(misskeyRoot, ".config", "vize-dev.pid");
  ensureFileContent(configPath, buildMisskeyDevConfig(port, pidFile));
  return "vize-dev.yml";
}

async function isPortAvailable(port: number): Promise<boolean> {
  if (hasListeningProcessOnPort(port)) {
    return false;
  }

  return await new Promise((resolve) => {
    const server = createServer();

    server.unref();
    server.once("error", () => {
      resolve(false);
    });
    // Probe the default bind target so wildcard IPv6 listeners are treated as busy too.
    server.listen(port, () => {
      server.close(() => {
        resolve(true);
      });
    });
  });
}

export async function resolveAvailablePort(preferredPort: number): Promise<number> {
  for (let offset = 0; offset < 20; offset += 1) {
    const port = preferredPort + offset;
    if (await isPortAvailable(port)) {
      return port;
    }
  }

  throw new Error(`Unable to find an available port starting at ${preferredPort}.`);
}

function sleep(milliseconds: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
}

function trimOutput(output: string | Buffer | null | undefined): string {
  if (typeof output === "string") {
    return output.trim();
  }
  if (output == null) {
    return "";
  }
  return output.toString("utf8").trim();
}

function formatCommandFailure(
  stdout: string | Buffer | null,
  stderr: string | Buffer | null,
): string {
  const trimmedStdout = trimOutput(stdout);
  const trimmedStderr = trimOutput(stderr);
  return [trimmedStdout, trimmedStderr].filter(Boolean).join("\n");
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

type MisskeyRuntimeInfo = {
  version: string;
  modules: string;
};

type MisskeyRuntimeMarker = MisskeyRuntimeInfo & {
  validatedWith: "backend-re2";
};

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

function probeMisskeyNativeDependencies(misskeyRoot: string): ReturnType<typeof spawnSync<string>> {
  return spawnMisskeyPnpm(misskeyRoot, [
    "exec",
    "node",
    "-e",
    "const mod = require.resolve('re2', { paths: ['./packages/backend'] }); require(mod);",
  ]);
}

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

async function toLaunchConfig(
  app: AppConfig,
  targetName: Exclude<Target, "playground" | "misskey">,
): Promise<LaunchConfig> {
  const port = await resolveAvailablePort(app.port);
  if (port !== app.port) {
    console.log(`[${targetName}] Port ${app.port} is busy. Using ${port}.`);
  }

  return {
    target: targetName,
    url: replacePortInUrl(app.url, app.port, port),
    setup: app.setup,
    cwd: app.cwd,
    command: app.command,
    args: replacePortInArgs(app.args, port),
    env: app.env,
  };
}

async function createLaunchConfig(currentTarget: Target): Promise<LaunchConfig> {
  if (currentTarget === "playground") {
    return {
      target: "playground",
      url: "http://127.0.0.1:4173",
      cwd: REPO_ROOT,
      command: "pnpm",
      args: ["-C", "playground", "dev"],
      env: {
        CI: "true",
      },
    };
  }

  if (currentTarget === "misskey") {
    const misskeyRoot = path.resolve(misskeyApp.cwd, "../..");
    const port = await resolveAvailablePort(3000);
    const configName = "vize-dev.yml";
    const misskeyCommand = getMisskeyPnpmCommand(resolveMisskeyCommandRoot(misskeyRoot), ["dev"]);
    return {
      target: "misskey",
      url: `http://127.0.0.1:${port}`,
      setup: misskeyApp.setup,
      beforeStart: () => {
        ensureMisskeyDevConfig(misskeyRoot, port);
        runMisskeyBeforeStart(misskeyRoot, configName);
      },
      cwd: misskeyRoot,
      command: misskeyCommand.command,
      args: misskeyCommand.args,
      env: {
        MISSKEY_CONFIG_YML: configName,
      },
    };
  }

  if (currentTarget === "npmx") {
    return await toLaunchConfig(npmxApp, "npmx");
  }

  if (currentTarget === "elk") {
    return await toLaunchConfig(elkApp, "elk");
  }

  return await toLaunchConfig(vuefesApp, "vuefes");
}

function ensureTargetBuilds(currentTarget: Target): void {
  if (skipBuild) {
    return;
  }

  if (!commandAvailable("wasm-pack")) {
    throw new Error(
      "wasm-pack is required for dev startup. Install it or rerun with --skip-build.",
    );
  }

  switch (currentTarget) {
    case "playground":
      ensureBuildPlayground();
      break;
    case "misskey":
      ensureBuildCommonVite();
      break;
    case "npmx":
    case "elk":
    case "vuefes":
      ensureBuildNuxtStack();
      break;
  }
}

async function startForeground(config: LaunchConfig): Promise<never> {
  console.log(`Starting ${config.target} on ${config.url}`);

  return await new Promise<never>((_, reject) => {
    const child = spawn(config.command, config.args, {
      cwd: config.cwd,
      env: {
        ...BASE_ENV,
        ...config.env,
      },
      stdio: "inherit",
    });

    const forwardSignal = (signal: NodeJS.Signals) => {
      if (!child.killed) {
        child.kill(signal);
      }
    };

    process.on("SIGINT", () => forwardSignal("SIGINT"));
    process.on("SIGTERM", () => forwardSignal("SIGTERM"));

    child.on("error", reject);
    child.on("exit", (code, signal) => {
      if (signal) {
        process.kill(process.pid, signal);
        return;
      }
      process.exit(code ?? 0);
    });
  });
}

export async function main(): Promise<never> {
  const launchConfig = await createLaunchConfig(target);

  ensureTargetBuilds(target);

  if (!skipSetup) {
    launchConfig.setup?.();
  }

  launchConfig.beforeStart?.();
  return await startForeground(launchConfig);
}

function isMainModule(): boolean {
  const entryPath = process.argv[1];
  if (entryPath == null) {
    return false;
  }
  return path.resolve(entryPath) === __filename;
}

if (isMainModule()) {
  await main();
}
