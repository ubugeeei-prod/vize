import assert from "node:assert/strict";
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
import { dirname, isAbsolute, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

export const upstreamSha = "c9a59cb07c236cbf7e0a97aa06ca3d02756415ea";
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const json = (path) => JSON.parse(readFileSync(path, "utf8"));
export const write = (path, value) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export const evidence = (path) => ({
  path,
  bytes: readFileSync(path).length,
  sha256: sha256(readFileSync(path)),
});

export function git(directory, ...args) {
  const result = spawnSync("git", ["-C", directory, ...args], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

export function inventory(directory, prefix = "") {
  return readdirSync(join(directory, prefix))
    .sort()
    .flatMap((name) => {
      const path = join(prefix, name);
      const stat = lstatSync(join(directory, path));
      if (stat.isDirectory()) return inventory(directory, path);
      if (stat.isSymbolicLink())
        return [
          {
            path,
            symlink: readlinkSync(join(directory, path)),
            resolved: realpathSync(join(directory, path)),
          },
        ];
      assert.ok(stat.isFile(), `unexpected non-file input ${path}`);
      const bytes = readFileSync(join(directory, path));
      return [{ path, bytes: bytes.length, sha256: sha256(bytes) }];
    });
}

/** Keep every message continuation and every duplicate; a Set would hide regressions. */
export function diagnostics(text, workspace, expected) {
  const normalized = text.replaceAll(workspace + "/", "").replaceAll("\r\n", "\n");
  const result = [];
  let active;
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
  const order = (a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b));
  const locations = result.map(({ message: _message, ...location }) => location).sort(order);
  assert.equal(result.length, 12, "require all twelve diagnostics, with no duplicate or extra");
  assert.deepEqual(
    locations,
    [...expected].sort(order),
    "authored diagnostic locations/codes changed",
  );
  return result.sort(order);
}

export function metrics(text) {
  const number = (name, unit = "") => {
    const match = new RegExp(`^${name}:\\s+([\\d.]+)${unit}\\s*$`, "m").exec(text);
    assert.ok(match, `missing extended diagnostic ${name}`);
    return Number(match[1]);
  };
  return {
    mapperWaitSeconds: number("Content mapper request wait time", "s"),
    checkSeconds: number("Check time", "s"),
    types: number("Types"),
    instantiations: number("Instantiations"),
    checkerMemoryKiB: number("Memory used", "K"),
  };
}

export function decodeFrames(bytes) {
  const result = [];
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

export function resourceUsage(text) {
  const result = {};
  for (const key of ["wall_s", "user_s", "system_s", "max_rss_kib", "status"]) {
    const match = new RegExp(`^${key}=([\\d.]+)$`, "m").exec(text);
    assert.ok(match, `missing GNU time field ${key}`);
    result[key] = Number(match[1]);
  }
  return result;
}

export async function capture(directory, command, args, cwd, env = {}) {
  mkdirSync(directory, { recursive: false });
  const stdoutPath = join(directory, "stdout.bin");
  const stderrPath = join(directory, "stderr.bin");
  const timePath = join(directory, "time.txt");
  const stdout = openSync(stdoutPath, "wx");
  const stderr = openSync(stderrPath, "wx");
  const started = new Date().toISOString();
  const cache = join(cwd, ".empty-cache");
  const temporary = join(cwd, ".empty-tmp");
  mkdirSync(cache);
  mkdirSync(temporary);
  const explicitEnvironment = { ...env, NO_COLOR: "1", XDG_CACHE_HOME: cache, TMPDIR: temporary };
  let timedOut = false;
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
      env: { ...process.env, ...explicitEnvironment },
      stdio: ["ignore", stdout, stderr],
      detached: true,
    },
  );
  const timer = setTimeout(() => {
    timedOut = true;
    process.kill(-child.pid, "SIGKILL");
  }, 300_000);
  let status;
  let signal;
  let error;
  try {
    [status, signal] = await new Promise((done, fail) => {
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

export function pristine(directory, sha) {
  assert.match(sha, /^[0-9a-f]{40}$/);
  assert.equal(git(directory, "rev-parse", "HEAD"), sha);
  assert.equal(git(directory, "status", "--porcelain", "--untracked-files=all"), "");
  return { sha, tree: git(directory, "rev-parse", "HEAD^{tree}") };
}

function linkProvider(destination, provider) {
  mkdirSync(destination);
  for (const name of readdirSync(provider)) {
    if (name !== "vize") symlinkSync(join(provider, name), join(destination, name));
  }
}

function mapperCommand(packageRoot, manifest) {
  return manifest.typescript.contentMapper.exec.map((part, index) =>
    index === 0 && part === "node"
      ? process.execPath
      : part.startsWith("./")
        ? join(packageRoot, part)
        : part,
  );
}

export function deriveConfig(original, packageName) {
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

export function prepareWorkspace(upstream, provider, directory, variant, captureDirectory) {
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
    cpSync(variant.package, packageRoot, { recursive: true, dereference: true });
    command = variant.binary
      ? [variant.binary, "content-mapper"]
      : mapperCommand(variant.package, json(join(variant.package, "package.json")));
  }
  const manifestPath = join(packageRoot, "package.json");
  const manifest = json(manifestPath);
  manifest.typescript.contentMapper.exec = command;
  if (captureDirectory) {
    mkdirSync(captureDirectory);
    copyFileSync(
      join(scriptDirectory, "template-alias-paired-11822-tee.mjs"),
      join(packageRoot, "capture.mjs"),
    );
    write(join(packageRoot, "capture-command.json"), command);
    manifest.typescript.contentMapper.exec = [process.execPath, join(packageRoot, "capture.mjs")];
  }
  writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
  const configFile =
    variant.id === "vue" ? "tsconfig.vue-content-mapper.json" : "tsconfig.vize-content-mapper.json";
  const original = json(join(upstream, configFile));
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

export function installedNative(provider, providerInputs, packageName, version, binaryNames) {
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
    .filter(({ path, symlink }) => !symlink && binaryNames.includes(path.split("/").at(-1)))
    .filter(({ path }) =>
      readFileSync(join(packageRoot, path))
        .subarray(0, 4)
        .equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])),
    );
  assert.equal(binaries.length, 1, `require one actual native binary for ${packageName}`);
  const binary = join(packageRoot, binaries[0].path);
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
    integrity: integrity[1],
    manifest: evidence(join(packageRoot, "package.json")),
    binary: evidence(binary),
    elfHeader: header.toString("hex"),
  };
}

export function collectTransforms(captureDirectory, workspace, output) {
  mkdirSync(output);
  const input = new Map();
  for (const name of readdirSync(captureDirectory).filter((name) => name.startsWith("in-"))) {
    for (const message of decodeFrames(readFileSync(join(captureDirectory, name)))) {
      if (message.id !== undefined) input.set(`${name.slice(3)}:${message.id}`, message);
    }
  }
  const transforms = [];
  for (const name of readdirSync(captureDirectory).filter((name) => name.startsWith("out-"))) {
    for (const message of decodeFrames(readFileSync(join(captureDirectory, name)))) {
      const request = input.get(`${name.slice(4)}:${message.id}`);
      if (request?.method !== "transform") continue;
      assert.equal(typeof message.result?.text, "string", "transform missing or failed");
      const path = relative(workspace, request.params.fileName);
      assert.ok(!path.startsWith("..") && !isAbsolute(path), "transform escaped corpus");
      const target = join(output, path + (message.result.extension ?? ".ts"));
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, message.result.text, { flag: "wx" });
      const { text, ...metadata } = message.result;
      write(target + ".metadata.json", { request, result: metadata });
      transforms.push({
        path,
        bytes: Buffer.byteLength(text),
        lines: text.split("\n").length,
        sha256: sha256(text),
        metadata,
      });
    }
  }
  assert.equal(transforms.length, 2500, "require complete transforms; no partial-work speedup");
  assert.equal(new Set(transforms.map(({ path }) => path)).size, 2500);
  write(join(output, "inventory.json"), transforms);
  return {
    count: transforms.length,
    bytes: transforms.reduce((n, t) => n + t.bytes, 0),
    lines: transforms.reduce((n, t) => n + t.lines, 0),
  };
}
