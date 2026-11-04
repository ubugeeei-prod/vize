import assert from "node:assert/strict";
import type {
  FileEvidence,
  InventoryEntry,
  ClosureSnapshot,
  PackageManifest,
  Location,
  Diagnostic,
  ProtocolMessage,
  Resources,
  Metrics,
  Observation,
  GitIdentity,
  MapperConfig,
  Variant,
  WorkspaceClosure,
} from "./template-alias-paired-11822-types.ts";
import { remainingBudget } from "./template-alias-paired-11822-process.ts";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  copyFileSync,
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";
import { installMapperCapture } from "./template-alias-paired-11822-capture.ts";
import { copyPublishedPackage } from "./template-alias-paired-11822-packages.ts";

export const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export const json = <T = PackageManifest>(path: string): T =>
  JSON.parse(readFileSync(path, "utf8")) as T;
export const write = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export const evidence = (path: string): FileEvidence => ({
  path,
  bytes: readFileSync(path).length,
  sha256: sha256(readFileSync(path)),
});

export function git(directory: string, ...args: string[]): string {
  const result = spawnSync("git", ["-C", directory, ...args], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function inventory(
  directory: string,
  prefix = "",
  allowedRoots: string[] = [directory],
): InventoryEntry[] {
  const roots = allowedRoots.map((root) => realpathSync(root));
  return readdirSync(join(directory, prefix))
    .sort()
    .flatMap((name) => {
      const path = join(prefix, name);
      const stat = lstatSync(join(directory, path));
      if (stat.isDirectory()) return inventory(directory, path, roots);
      if (stat.isSymbolicLink()) {
        const resolved = realpathSync(join(directory, path));
        assert.ok(
          roots.some((root) => {
            const suffix = relative(root, resolved);
            return suffix === "" || (!suffix.startsWith("..") && !isAbsolute(suffix));
          }),
          `external symlink escapes admitted full closure: ${path}`,
        );
        return [
          {
            path,
            symlink: readlinkSync(join(directory, path)),
            resolved,
          },
        ];
      }
      assert.ok(stat.isFile(), `unexpected non-file input ${path}`);
      const bytes = readFileSync(join(directory, path));
      return [{ path, bytes: bytes.length, sha256: sha256(bytes) }];
    });
}

export function closureSnapshot(roots: string[], files: string[] = []): ClosureSnapshot {
  const directories = roots.map((root) => realpathSync(root));
  return {
    directories: directories.map((root) => ({ root, files: inventory(root, "", directories) })),
    files: [...new Set(files.map((file) => resolve(file)))].sort().map(evidence),
  };
}

export function strictNumber(token: string, name: string, integer = false): number {
  assert.match(token, /^\d+(?:\.\d+)?$/, `malformed numeric ${name}`);
  const value = Number(token);
  assert.ok(Number.isFinite(value) && value >= 0, `non-finite numeric ${name}`);
  if (integer) assert.ok(Number.isSafeInteger(value), `unsafe integer ${name}`);
  return value;
}

/** Keep every message continuation and every duplicate; a Set would hide regressions. */
export function diagnostics(text: string, workspace: string, expected: Location[]): Diagnostic[] {
  const normalized = text.replaceAll(workspace + "/", "").replaceAll("\r\n", "\n");
  const result: Diagnostic[] = [];
  let active: Diagnostic | undefined;
  for (const line of normalized.split("\n")) {
    const match =
      /^(src\/generated\/GeneratedComponent\d{4}\.vue)\((\d+),(\d+)\): error TS(\d+): (.*)$/.exec(
        line,
      );
    if (match) {
      active = {
        file: match[1],
        line: +match[2],
        column: +match[3],
        code: +match[4],
        message: match[5],
      };
      result.push(active);
    } else {
      assert.ok(!line.includes("error TS"), `unparsed diagnostic: ${line}`);
      if (active && /^\s+\S/.test(line)) active.message += "\n" + line;
      else active = undefined;
    }
  }
  const order = (a: unknown, b: unknown) => JSON.stringify(a).localeCompare(JSON.stringify(b));
  const locations = result.map(({ message: _message, ...location }) => location).sort(order);
  assert.equal(result.length, 12, "require all twelve diagnostics, with no duplicate or extra");
  assert.deepEqual(
    locations,
    [...expected].sort(order),
    "authored diagnostic locations/codes changed",
  );
  return result.sort(order);
}

export function metrics(text: string): Metrics {
  const number = (name: string, unit = "", integer = false) => {
    const match = new RegExp(`^${name}:\\s+(\\S+?)${unit}\\s*$`, "m").exec(text);
    assert.ok(match, `missing extended diagnostic ${name}`);
    return strictNumber(match[1], name, integer);
  };
  return {
    mapperWaitSeconds: number("Content mapper request wait time", "s"),
    checkSeconds: number("Check time", "s"),
    types: number("Types", "", true),
    instantiations: number("Instantiations", "", true),
    checkerMemoryKiB: number("Memory used", "K", true),
  };
}

export function decodeFrames(bytes: Buffer): ProtocolMessage[] {
  const result: ProtocolMessage[] = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert.ok(end >= offset, "truncated protocol header");
    const header = bytes.subarray(offset, end).toString("ascii");
    const lengths = [...header.matchAll(/(?:^|\r\n)Content-Length:\s*(\d+)(?=\r\n|$)/gi)];
    assert.equal(lengths.length, 1, "ambiguous protocol length");
    const length = Number(lengths[0][1]);
    const start = end + 4;
    assert.ok(
      Number.isSafeInteger(length) && start + length <= bytes.length,
      "truncated protocol body",
    );
    result.push(JSON.parse(bytes.subarray(start, start + length).toString("utf8")));
    offset = start + length;
  }
  return result;
}

export function resourceUsage(text: string): Resources {
  const result = {} as Resources;
  for (const key of ["wall_s", "user_s", "system_s", "max_rss_kib", "status"] as const) {
    const match = new RegExp(`^${key}=(\\S+)$`, "m").exec(text);
    assert.ok(match, `missing GNU time field ${key}`);
    result[key] = strictNumber(match[1], key, key === "status" || key === "max_rss_kib");
  }
  return result;
}

/** External mappers and package hooks receive no CI credentials or token URLs. */
export function childEnvironment(
  extra: NodeJS.ProcessEnv = {},
  ambient: NodeJS.ProcessEnv = process.env,
): NodeJS.ProcessEnv {
  const keys = [
    "PATH",
    "CI",
    "HOME",
    "USER",
    "LOGNAME",
    "LANG",
    "LC_ALL",
    "TZ",
    "SHELL",
    "GOMAXPROCS",
    "RAYON_NUM_THREADS",
    "NODE_OPTIONS",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
  ];
  return {
    ...Object.fromEntries(
      keys.filter((key) => ambient[key] !== undefined).map((key) => [key, ambient[key]]),
    ),
    ...extra,
  };
}

export async function capture(
  directory: string,
  command: string,
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv = {},
): Promise<Observation> {
  mkdirSync(directory, { recursive: false });
  const stdoutPath = join(directory, "stdout.bin");
  const stderrPath = join(directory, "stderr.bin");
  const timePath = join(directory, "time.txt");
  const timeoutMilliseconds = Math.min(300_000, remainingBudget());
  const stdout = openSync(stdoutPath, "wx");
  const stderr = openSync(stderrPath, "wx");
  const started = new Date().toISOString();
  const cache = join(cwd, ".empty-cache");
  const temporary = join(cwd, ".empty-tmp");
  mkdirSync(cache);
  mkdirSync(temporary);
  const explicitEnvironment = { ...env, NO_COLOR: "1", XDG_CACHE_HOME: cache, TMPDIR: temporary };
  let timedOut = false;
  write(join(directory, "process-start.json"), {
    command,
    args,
    cwd,
    explicitEnvironment,
    started,
    timeoutMilliseconds,
    sourceHead: process.env.HEAD_SHA,
  });
  const child = spawn(
    "/usr/bin/time",
    [
      "-f",
      "wall_s=%e\nuser_s=%U\nsystem_s=%S\nmax_rss_kib=%M\nstatus=%x",
      "-o",
      timePath,
      command,
      ...args,
    ],
    {
      cwd,
      env: childEnvironment(explicitEnvironment),
      stdio: ["ignore", stdout, stderr],
      detached: true,
    },
  );
  const timer = setTimeout(() => {
    timedOut = true;
    if (child.pid) process.kill(-child.pid, "SIGKILL");
  }, timeoutMilliseconds);
  let status: number | null = null;
  let signal: NodeJS.Signals | null = null;
  let error: string | undefined;
  try {
    [status, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((done, fail) => {
      child.on("error", fail);
      child.on("close", (...values) => done(values));
    });
  } catch (cause) {
    error = String(cause);
  } finally {
    clearTimeout(timer);
    closeSync(stdout);
    closeSync(stderr);
  }
  const observation = {
    command,
    args,
    cwd,
    explicitEnvironment,
    started,
    ended: new Date().toISOString(),
    status,
    signal,
    timedOut,
    error,
    stdout: evidence(stdoutPath),
    stderr: evidence(stderrPath),
    time: existsSync(timePath) ? evidence(timePath) : null,
  };
  write(join(directory, "process.json"), observation);
  assert.equal(error, undefined, "spawn failed; raw process evidence retained");
  assert.equal(signal, null, "process terminated; raw evidence retained");
  assert.equal(timedOut, false);
  const usage = resourceUsage(readFileSync(timePath, "utf8"));
  assert.equal(usage.status, status);
  return {
    ...observation,
    usage,
    text: readFileSync(stdoutPath, "utf8") + "\n" + readFileSync(stderrPath, "utf8"),
  };
}

export function pristine(directory: string, sha: string): GitIdentity {
  assert.match(sha, /^[0-9a-f]{40}$/);
  assert.equal(git(directory, "rev-parse", "HEAD"), sha);
  assert.equal(git(directory, "status", "--porcelain", "--untracked-files=all"), "");
  return { sha, tree: git(directory, "rev-parse", "HEAD^{tree}") };
}

function linkProvider(destination: string, provider: string) {
  mkdirSync(destination);
  for (const name of readdirSync(provider)) {
    if (name !== "vize") symlinkSync(join(provider, name), join(destination, name));
  }
}

function mapperCommand(packageRoot: string, manifest: PackageManifest): string[] {
  assert.ok(manifest.typescript?.contentMapper);
  return manifest.typescript.contentMapper.exec.map((part, index) =>
    index === 0 && part === "node"
      ? process.execPath
      : part.startsWith("./")
        ? join(packageRoot, part)
        : part,
  );
}

export function deriveConfig(original: MapperConfig, packageName: string): MapperConfig {
  assert.equal(original.contentMappers.length, 1);
  assert.equal(original.contentMappers[0].package, packageName);
  const config = structuredClone(original);
  config.compilerOptions = { ...config.compilerOptions, incremental: false };
  config.contentMappers[0].package =
    packageName === "@vue/content-mapper" ? "template-alias-vue" : "vize";
  const restored = structuredClone(config);
  if (original.compilerOptions === undefined) delete restored.compilerOptions;
  else restored.compilerOptions = original.compilerOptions;
  restored.contentMappers[0].package = packageName;
  assert.deepEqual(
    restored,
    original,
    "only cache policy and wrapper package may differ from pinned config",
  );
  return config;
}

export function prepareWorkspace(
  upstream: string,
  provider: string,
  directory: string,
  variant: Variant,
  captureDirectory?: string,
): WorkspaceClosure {
  mkdirSync(directory);
  cpSync(join(upstream, "src"), join(directory, "src"), { recursive: true });
  copyFileSync(join(upstream, "tsconfig.base.json"), join(directory, "tsconfig.base.json"));
  linkProvider(join(directory, "node_modules"), join(provider, "node_modules"));
  const packageName = variant.id === "vue" ? "@vue/content-mapper" : "vize";
  let packageRoot;
  let command;
  if (variant.id === "vue") {
    // Use an unscoped wrapper without replacing the shared provider's scoped symlink.
    packageRoot = join(directory, "node_modules", "template-alias-vue");
    const referenceRoot = realpathSync(join(provider, "node_modules", packageName));
    const reference = json(join(referenceRoot, "package.json"));
    command = mapperCommand(referenceRoot, reference);
    cpSync(referenceRoot, packageRoot, { recursive: true, dereference: true });
  } else {
    packageRoot = join(directory, "node_modules", "vize");
    copyPublishedPackage(variant.package, packageRoot);
    command = mapperCommand(variant.package, json(join(variant.package, "package.json")));
  }
  const manifestPath = join(packageRoot, "package.json");
  const manifest = json(manifestPath);
  assert.ok(manifest.typescript?.contentMapper);
  manifest.typescript.contentMapper.exec = command;
  if (captureDirectory) {
    manifest.typescript.contentMapper.exec = installMapperCapture(
      packageRoot,
      captureDirectory,
      command,
    );
  }
  writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
  const configFile =
    variant.id === "vue" ? "tsconfig.vue-content-mapper.json" : "tsconfig.vize-content-mapper.json";
  const original = json<MapperConfig>(join(upstream, configFile));
  const config = deriveConfig(original, packageName);
  write(join(directory, configFile), config);
  assert.equal(
    inventory(join(directory, "src")).filter(({ path }) => path.endsWith(".vue")).length,
    2500,
  );
  assert.equal(
    inventory(join(directory, "src")).filter(({ path }) => path.endsWith(".tsbuildinfo")).length,
    0,
  );
  return {
    configFile,
    packageRoot,
    originalConfig: original,
    config,
    changedFields: [
      "compilerOptions.incremental",
      ...(variant.id === "vue" ? ["contentMappers[0].package"] : []),
    ],
    package: inventory(packageRoot),
    authoredInputs: inventory(join(directory, "src")),
  };
}

export function installedNative(
  provider: string,
  providerInputs: InventoryEntry[],
  packageName: string,
  version: string,
  binaryNames: string[],
  kind = "executable",
) {
  const manifests = providerInputs
    .filter(({ path }) => `/${path}`.endsWith(`/node_modules/${packageName}/package.json`))
    .filter(({ path }) => {
      const manifest = json(join(provider, path));
      return manifest.name === packageName && manifest.version === version;
    });
  assert.equal(
    manifests.length,
    1,
    `require one exact native platform package ${packageName}@${version}`,
  );
  const packageRoot = dirname(join(provider, manifests[0].path));
  const binaries = inventory(packageRoot)
    .filter(({ path, symlink }) => !symlink && binaryNames.includes(basename(path)))
    .filter(({ path }) =>
      readFileSync(join(packageRoot, path))
        .subarray(0, 4)
        .equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])),
    );
  assert.equal(binaries.length, 1, `require one actual native binary for ${packageName}`);
  const binary = join(packageRoot, binaries[0].path);
  assert.equal(
    binary.endsWith(".node"),
    kind === "napi",
    "NAPI images are not executable commands",
  );
  const header = readFileSync(binary).subarray(0, 20);
  assert.equal(header[4], 2, "ELF64 required");
  assert.equal(header[5], 1, "little-endian required");
  assert.equal(header.readUInt16LE(18), 62, "x86-64 required");
  const lock = readFileSync(join(provider, "pnpm-lock.yaml"), "utf8").split("\n");
  const key = `${packageName}@${version}`;
  const line = lock.findIndex((line) => line.trim() === `'${key}':` || line.trim() === `${key}:`);
  assert.ok(line >= 0, "native platform package is absent from pinned lock");
  const integrity = /resolution: \{integrity: (sha512-[^}]+)\}/.exec(lock[line + 1]);
  assert.ok(integrity, "native platform package has no pinned SRI");
  return {
    packageName,
    version,
    kind,
    integrity: integrity[1],
    integrityMeaning:
      "pinned tarball SRI recorded; installed payload SHA and pre/post closure are separate receipts",
    manifest: evidence(join(packageRoot, "package.json")),
    binary: evidence(binary),
    elfHeader: header.toString("hex"),
  };
}
