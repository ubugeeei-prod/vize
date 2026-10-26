import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, type BinaryLike } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const CORSA_OVERRIDES = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];
export interface PackageIdentity {
  path: string;
  name: string;
  version: string;
  manifestSha256: string;
  lockFile: string;
  lockSha256: string;
  resolved: string;
  integrity: string;
  manifest: any;
}
export interface Executable {
  path: string;
  sha256: string;
  version: string;
}
export interface NativeProvider {
  root: string;
  meta: PackageIdentity;
  platform: PackageIdentity;
  executable: Executable;
  versionProbe: { status: number | null; signal: string | null; stdout: string; stderr: string };
  vue?: PackageIdentity;
}
export interface NativeProviders {
  beta: NativeProvider;
  six: PackageIdentity;
  vue: PackageIdentity;
  frozen702: NativeProvider | null;
}
export type ConsumerKind =
  | "canonical-beta"
  | "alias-beta"
  | "explicit-beta"
  | "selected-config"
  | "absent"
  | "stale-decoys"
  | "canonical-702"
  | "alias-702"
  | "explicit-702";
export interface ConsumerInput {
  path: string;
  bytes: number;
  sha256: string;
  text: string;
}
export interface CacheDecoy {
  kind: string;
  packageRoot: string;
  physicalExecutable: Executable;
  [detail: string]: unknown;
}
export interface NativeRuntimeConsumer {
  kind: ConsumerKind;
  workspace: string;
  configRoot: string;
  configFile: string;
  entry: string;
  expectedExecutable: Executable | null;
  cacheDecoys: CacheDecoy[];
  explicitExecutable: Executable | null;
  inputPaths: string[];
  providers: NativeProviders;
  inputFiles: ConsumerInput[];
}
export const sha256 = (value: BinaryLike) => createHash("sha256").update(value).digest("hex");
export const SOURCES = {
  valid:
    '<script setup lang="ts">\nimport { twice } from "./support";\nconst value: number = 2;\nconst doubled = twice(value);\n</script>\n<template><p>{{ value }} {{ doubled }}</p></template>\n',
  error:
    '<script setup lang="ts">\nimport { twice } from "./support";\nconst value: number = "wrong";\nconst doubled = twice(value);\n</script>\n<template><p>{{ value }} {{ doubled }}</p></template>\n',
  repair:
    '<script setup lang="ts">\nimport { twice } from "./support";\nconst value: number = 2;\nconst doubled = twice(value);\n</script>\n<template><p>{{ value }} {{ doubled }}</p></template>\n',
  support: "export const twice = (value: number): number => value * 2;\n",
};
export const COMPILER_OPTIONS = {
  strict: true,
  target: "ES2022",
  module: "ESNext",
  moduleResolution: "Bundler",
  skipLibCheck: true,
  noEmit: true,
};
const json = (file: string): any => JSON.parse(fs.readFileSync(file, "utf8"));
function packageIdentity(root: string, name: string, expectedVersion?: string): PackageIdentity {
  const packagePath = fs.realpathSync(path.join(root, "node_modules", name));
  const file = path.join(packagePath, "package.json"),
    bytes = fs.readFileSync(file),
    manifest = JSON.parse(bytes.toString("utf8"));
  assert.equal(manifest.name, name);
  if (expectedVersion) assert.equal(manifest.version, expectedVersion);
  const lockFile = path.join(root, "package-lock.json"),
    lock = json(lockFile),
    installed = lock.packages?.[`node_modules/${name}`];
  assert.ok(installed, "official npm provider lock entry is required");
  assert.equal(installed.version, manifest.version);
  assert.ok(
    installed.resolved?.startsWith(`https://registry.npmjs.org/${name}/-/`),
    "official npm registry tarball is required",
  );
  assert.match(installed.integrity, /^sha512-[A-Za-z0-9+/]+=*$/u);
  return {
    path: packagePath,
    name,
    version: manifest.version,
    manifestSha256: sha256(bytes),
    lockFile,
    lockSha256: sha256(fs.readFileSync(lockFile)),
    resolved: installed.resolved,
    integrity: installed.integrity,
    manifest,
  };
}
function nativeProvider(root: string, version: string): NativeProvider {
  const selected = fs.realpathSync(root),
    platformName = `@typescript/typescript-${process.platform}-${process.arch}`;
  const meta = packageIdentity(selected, "typescript", version);
  const platform = packageIdentity(selected, platformName, version);
  assert.equal(meta.manifest.optionalDependencies?.[platformName], version);
  const binary = fs.realpathSync(
    path.join(platform.path, "lib", process.platform === "win32" ? "tsc.exe" : "tsc"),
  );
  const probe = spawnSync(binary, ["--version"], { encoding: "utf8" });
  assert.equal(probe.error, undefined);
  assert.equal(probe.status, 0);
  assert.equal(probe.signal, null);
  assert.equal(probe.stdout, `Version ${version}\n`);
  assert.equal(probe.stderr, "");
  return {
    root: selected,
    meta,
    platform,
    executable: {
      path: binary,
      sha256: sha256(fs.readFileSync(binary)),
      version: `Version ${version}`,
    },
    versionProbe: {
      status: probe.status,
      signal: probe.signal,
      stdout: probe.stdout,
      stderr: probe.stderr,
    },
  };
}
export function loadNativeProviders(env: NodeJS.ProcessEnv = process.env): NativeProviders {
  assert.ok(env.VIZE_NATIVE_BETA_ROOT, "VIZE_NATIVE_BETA_ROOT is mandatory; no compiler fallback");
  assert.ok(
    env.VIZE_TYPESCRIPT_SIX_ROOT,
    "VIZE_TYPESCRIPT_SIX_ROOT is mandatory; no compiler fallback",
  );
  const beta = nativeProvider(env.VIZE_NATIVE_BETA_ROOT, "7.1.0-beta");
  const sixRoot = fs.realpathSync(env.VIZE_TYPESCRIPT_SIX_ROOT),
    six = packageIdentity(sixRoot, "typescript", "6.0.3");
  const vue = packageIdentity(beta.root, "vue");
  assert.match(vue.version, /^3\./u);
  const frozen702 = env.VIZE_NATIVE_702_ROOT
    ? nativeProvider(env.VIZE_NATIVE_702_ROOT, "7.0.2")
    : null;
  if (frozen702) frozen702.vue = packageIdentity(frozen702.root, "vue");
  return { beta, six, vue, frozen702 };
}
export function runtimeChildEnvironment(
  base: NodeJS.ProcessEnv = process.env,
  directory?: string,
): NodeJS.ProcessEnv {
  const env = { ...base };
  for (const key of [...CORSA_OVERRIDES, "CORSA_BIN"]) delete env[key];
  env.VIZE_CORSA_DEV_PATHS = "0";
  env.NO_COLOR = "1";
  env.FORCE_COLOR = "0";
  if (directory) {
    const toolPath = path.join(directory, "tool-path");
    fs.mkdirSync(toolPath, { recursive: true });
    const node = path.join(toolPath, process.platform === "win32" ? "node.exe" : "node");
    if (!fs.existsSync(node)) fs.symlinkSync(process.execPath, node, "file");
    assert.equal(fs.realpathSync(node), fs.realpathSync(process.execPath));
    fs.writeFileSync(
      path.join(directory, "node-identity.json"),
      `${JSON.stringify(
        {
          alias: node,
          physicalPath: fs.realpathSync(node),
          version: process.version,
          bytes: fs.statSync(node).size,
          sha256: sha256(fs.readFileSync(node)),
          custody:
            "External Node executable; complete image SHA retained, duplicate tool symlink bytes are not sealed in the source artifact.",
        },
        null,
        2,
      )}\n`,
    );
    // Config evaluation needs Node; compiler/global npm launchers remain absent.
    env.PATH = toolPath;
    // Isolate only the returned child environment; caller/shell HOME is untouched.
    const childHomeDirectory = path.join(directory, "empty-home");
    fs.mkdirSync(childHomeDirectory, { recursive: true });
    env.HOME = childHomeDirectory;
  }
  for (const key of CORSA_OVERRIDES) assert.equal(env[key], undefined);
  return env;
}
function link(target: string, destination: string) {
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.symlinkSync(target, destination, process.platform === "win32" ? "junction" : "dir");
}
export function prepareNativeRuntimeConsumer({
  directory,
  kind,
  providers,
}: {
  directory: string;
  kind: ConsumerKind;
  providers: NativeProviders;
}): NativeRuntimeConsumer {
  assert.ok(
    [
      "canonical-beta",
      "alias-beta",
      "explicit-beta",
      "selected-config",
      "absent",
      "stale-decoys",
      "canonical-702",
      "alias-702",
      "explicit-702",
    ].includes(kind),
  );
  assert.equal(fs.existsSync(directory), false, "each consumer starts absent");
  fs.mkdirSync(directory, { recursive: true });
  const workspace = fs.realpathSync(directory),
    configRoot = kind === "selected-config" ? path.join(workspace, "selected") : workspace;
  const entry = path.join(configRoot, "src/App.vue"),
    configFile = path.join(configRoot, "tsconfig.json"),
    inputPaths: string[] = [];
  const cacheDecoys: CacheDecoy[] = [];
  const write = (file: string, text: string) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text);
    inputPaths.push(file);
  };
  const frozen = kind.endsWith("-702");
  const native = frozen ? providers.frozen702 : providers.beta;
  assert.ok(native, "the selected official native provider must be explicitly installed");
  const vue = frozen ? native.vue : providers.vue;
  assert.ok(vue, "the selected provider must carry its Vue package");
  const explicit = kind === "explicit-beta" || kind === "explicit-702";
  const expectedExecutable =
    kind === "absent" || kind === "stale-decoys" ? null : native.executable;
  const meta = native.meta;
  const deps: Record<string, string> = { vue: vue.version };
  if (["canonical-beta", "canonical-702", "selected-config"].includes(kind))
    deps.typescript = meta.version;
  if (["alias-beta", "alias-702"].includes(kind)) {
    deps.typescript = "6.0.3";
    deps["typescript-native"] = `npm:typescript@${meta.version}`;
  }
  write(
    path.join(configRoot, "package.json"),
    JSON.stringify({ private: true, type: "module", devDependencies: deps }, null, 2) + "\n",
  );
  write(
    configFile,
    JSON.stringify(
      { compilerOptions: COMPILER_OPTIONS, include: ["src/**/*.vue", "src/**/*.ts"] },
      null,
      2,
    ) + "\n",
  );
  write(entry, SOURCES.valid);
  write(path.join(configRoot, "src/support.ts"), SOURCES.support);
  write(
    path.join(configRoot, "vite.config.mjs"),
    `export default ${JSON.stringify({ vue: { languageServer: { editor: true, typecheck: true, lint: false, ecosystem: false }, ...(explicit ? { typeChecker: { corsaPath: expectedExecutable?.path } } : {}) } })};\n`,
  );
  link(vue.path, path.join(configRoot, "node_modules/vue"));
  link(path.join(path.dirname(vue.path), "@vue"), path.join(configRoot, "node_modules/@vue"));
  if (["canonical-beta", "canonical-702", "selected-config"].includes(kind))
    link(meta.path, path.join(configRoot, "node_modules/typescript"));
  if (["alias-beta", "alias-702"].includes(kind)) {
    link(providers.six.path, path.join(configRoot, "node_modules/typescript"));
    link(meta.path, path.join(configRoot, "node_modules/typescript-native"));
  }
  if (kind === "selected-config") {
    write(
      path.join(workspace, "package.json"),
      JSON.stringify({ private: true, devDependencies: { typescript: "6.0.3" } }, null, 2) + "\n",
    );
    link(providers.six.path, path.join(workspace, "node_modules/typescript"));
  }
  if (kind === "stale-decoys") {
    const old = providers.frozen702;
    assert.ok(old, "real stale-cache controls require the separate official 7.0.2 provider");
    const platform = old.platform.name;
    const nested = path.join(configRoot, "node_modules/vize/node_modules");
    link(old.meta.path, path.join(nested, "typescript"));
    link(old.platform.path, path.join(nested, platform));
    const store = path.join(
      configRoot,
      "node_modules/.pnpm",
      `@typescript+typescript-${process.platform}-${process.arch}@7.0.2/node_modules`,
    );
    link(old.platform.path, path.join(store, platform));
    const globalStoragePath = path.join(workspace, "legacy-vscode-global-storage");
    const legacyCacheRoot = path.join(
      globalStoragePath,
      "corsa",
      "7.0.2",
      `${process.platform}-${process.arch}`,
    );
    link(old.platform.path, legacyCacheRoot);
    cacheDecoys.push(
      {
        kind: "legacy-editor-global-storage",
        globalStoragePath,
        packageRoot: legacyCacheRoot,
        manifestPath: path.join(legacyCacheRoot, "package.json"),
        manifestSha256: old.platform.manifestSha256,
        executablePath: path.join(
          legacyCacheRoot,
          "lib",
          process.platform === "win32" ? "tsc.exe" : "tsc",
        ),
        physicalExecutable: old.executable,
        legacySource:
          "editors/vscode/src/corsa-runtime.ts: resolveCorsaServerEnv cached globalStoragePath/corsa/version/platform-arch",
        scope:
          "Real cache-present direct CLI/LSP selection refusal only; no editor globalStorage dispatch or cache-read credit",
      },
      {
        kind: "vize-owned-runtime",
        packageRoot: path.join(nested, platform),
        physicalExecutable: old.executable,
      },
      {
        kind: "unlinked-pnpm-store",
        packageRoot: path.join(store, platform),
        physicalExecutable: old.executable,
      },
    );
  }

  const consumer: NativeRuntimeConsumer = {
    kind,
    workspace,
    configRoot,
    configFile,
    entry,
    expectedExecutable,
    cacheDecoys,
    explicitExecutable: explicit ? expectedExecutable : null,
    inputPaths,
    providers,
    inputFiles: [],
  };
  consumer.inputFiles = captureConsumerInputs(consumer);
  return consumer;
}
export function captureConsumerInputs(
  consumer: Pick<NativeRuntimeConsumer, "inputPaths" | "workspace">,
): ConsumerInput[] {
  return consumer.inputPaths.map((file) => {
    const bytes = fs.readFileSync(file);
    return {
      path: path.relative(consumer.workspace, file).split(path.sep).join("/"),
      bytes: bytes.length,
      sha256: sha256(bytes),
      text: bytes.toString("utf8"),
    };
  });
}
export function assertProviderFiles(providers: NativeProviders) {
  for (const provider of [providers.beta, providers.frozen702].filter((value) => value !== null))
    assert.equal(sha256(fs.readFileSync(provider.executable.path)), provider.executable.sha256);
  for (const pkg of [
    providers.beta.meta,
    providers.beta.platform,
    providers.six,
    providers.vue,
    providers.frozen702?.meta,
    providers.frozen702?.platform,
    providers.frozen702?.vue,
  ].filter((value) => value !== null && value !== undefined))
    assert.equal(sha256(fs.readFileSync(path.join(pkg.path, "package.json"))), pkg.manifestSha256);
}
