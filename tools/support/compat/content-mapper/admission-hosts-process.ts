import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { spawn } from "node:child_process";
import {
  closeSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  realpathSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { createRequire } from "node:module";
import { basename, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { fileImage, type FileImage } from "../../release/computed_event_custody.ts";
import { linuxIdentity, OwnedProcesses, ownerVariable } from "./admission-hosts-owned.ts";
import { childEnvironment } from "../../release/constructor_original27_public/capture.ts";
const self = fileURLToPath(import.meta.url);
export const save = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export function environment() {
  const env = childEnvironment(process.env);
  for (const name of Object.keys(env))
    if (
      /^(?:NAPI_RS_|CORSA_|TSGO_|TSRS_)/u.test(name) ||
      [
        "NODE_OPTIONS",
        "NODE_PATH",
        "GOMAXPROCS",
        "VIZE_PREFER_WORKSPACE_BINDING",
        "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
      ].includes(name)
    )
      delete env[name];
  env.PATH = process.execPath.slice(0, process.execPath.lastIndexOf("/")) + ":" + (env.PATH ?? "");
  return env;
}
export async function run(
  directory: string,
  argv: string[],
  cwd: string,
  expectedHost?: FileImage,
  expectedNative?: FileImage,
) {
  mkdirSync(directory);
  const stdout = openSync(join(directory, "stdout.raw"), "wx"),
    stderr = openSync(join(directory, "stderr.raw"), "wx");
  const env = environment(),
    ownerNonce = randomUUID(),
    deadline = Number(process.env.ADMISSION_DEADLINE_MS);
  assert.ok(
    Number.isSafeInteger(deadline) && deadline > Date.now(),
    "functional job soft deadline reached",
  );
  env[ownerVariable] = ownerNonce;
  save(join(directory, "spawn.json"), {
    ownerNonce,
    argv,
    cwd,
    deadline,
    node: fileImage(process.execPath),
    environment: { NODE_OPTIONS: env.NODE_OPTIONS ?? null, GOMAXPROCS: env.GOMAXPROCS ?? null },
  });
  let timedOut = false,
    error: string | null = null;
  const observedHosts = new Map<string, unknown>(),
    observedNatives = new Map<string, unknown>();
  const child = spawn(argv[0], argv.slice(1), {
    cwd,
    env,
    detached: true,
    stdio: ["ignore", stdout, stderr],
  });
  child.once("error", (value) => {
    error = value.message;
  });
  const owned = new OwnedProcesses(child.pid, ownerNonce);
  const inspectHost = () => {
    for (const row of owned.scan()) {
      if (!expectedHost && !expectedNative) continue;
      const name = String(row.pid);
      try {
        const exe = realpathSync("/proc/" + name + "/exe");
        const key = name + ":" + row.startTicks;
        let nativeObservation: Record<string, unknown> | undefined;
        let hostObservation: Record<string, unknown> | undefined;
        if (expectedNative && !observedNatives.has(key)) {
          const maps = readFileSync("/proc/" + name + "/maps", "utf8");
          if (
            maps
              .split("\n")
              .some((line) => line.trim().split(/\s+/).slice(5).join(" ") === expectedNative.path)
          ) {
            const image = fileImage(expectedNative.path);
            assert.deepEqual(image, expectedNative);
            const affinity = readFileSync("/proc/" + name + "/status", "utf8")
              .split("\n")
              .find((line) => line.startsWith("Cpus_allowed_list:"))!;
            assert.match(
              affinity.split(":")[1].trim(),
              /^[0-9]+$/u,
              "normal mapper must retain one-CPU affinity",
            );
            nativeObservation = {
              affinity,
              ...row,
              image,
              maps,
              argv: readFileSync("/proc/" + name + "/cmdline", "utf8")
                .split("\0")
                .filter(Boolean),
            };
          }
        }
        if (expectedHost && exe === expectedHost.path && !observedHosts.has(key)) {
          const image = fileImage(exe);
          assert.deepEqual(image, expectedHost);
          hostObservation = {
            ...row,
            image,
            argv: readFileSync("/proc/" + name + "/cmdline", "utf8")
              .split("\0")
              .filter(Boolean),
          };
        }
        if (nativeObservation || hostObservation) {
          const identityAfter = linuxIdentity(name);
          assert.equal(
            identityAfter.startTicks,
            row.startTicks,
            "process identity changed during custody capture",
          );
          if (nativeObservation) observedNatives.set(key, { ...nativeObservation, identityAfter });
          if (hostObservation) observedHosts.set(key, { ...hostObservation, identityAfter });
        }
      } catch (value) {
        if (value instanceof assert.AssertionError) error = value.message;
        else if (!["ENOENT", "ESRCH"].includes((value as NodeJS.ErrnoException).code ?? ""))
          error = String(value);
      }
    }
    error ??= owned.error;
  };
  inspectHost();
  const observer = setInterval(inspectHost, 20);
  const timer = setTimeout(
    () => {
      timedOut = true;
      inspectHost();
      owned.killAll();
    },
    Math.max(1, Math.min(75_000, deadline - Date.now())),
  );
  let status: number | null, signal: NodeJS.Signals | null;
  try {
    [status, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((done) =>
      child.once("close", (code, reason) => done([code, reason])),
    );
  } finally {
    inspectHost();
    clearInterval(observer);
    clearTimeout(timer);
    closeSync(stdout);
    closeSync(stderr);
  }
  let remaining = owned.scan();
  const reapDeadline = Math.min(Date.now() + 5000, deadline);
  while (remaining.length > 0 && Date.now() < reapDeadline) {
    await new Promise((done) => setTimeout(done, 20));
    remaining = owned.scan();
  }
  const survivors = remaining.length > 0;
  const cleanup = survivors ? owned.killAll() : [];
  error ??= owned.error;
  const result = {
    pid: child.pid ?? null,
    status,
    signal,
    error,
    timedOut,
    survivors,
    remaining,
    cleanup,
    observedProcesses: [...owned.identities.values()],
    observedHosts: [...observedHosts.values()],
    observedNatives: [...observedNatives.values()],
    stdout: fileImage(join(directory, "stdout.raw")),
    stderr: fileImage(join(directory, "stderr.raw")),
  };
  save(join(directory, "result.json"), result);
  assert.ok(
    !timedOut && !survivors && !signal && !error,
    "host timeout/crash/survivor is a functional failure",
  );
  assert.ok(
    !expectedHost || observedHosts.size > 0,
    "actual pinned Go ELF process was not observed",
  );
  assert.ok(
    !expectedNative || observedNatives.size > 0,
    "normal mapper NAPI image was not observed in actual process maps",
  );
  return {
    ...result,
    text: readFileSync(join(directory, "stdout.raw"), "utf8"),
    errors: readFileSync(join(directory, "stderr.raw"), "utf8"),
  };
}
export function recordNativeIdentity(root: string, directory: string, token: string) {
  const require = createRequire(join(root, "package.json"));
  const binding = require("@vizejs/native") as { getNativeBuildToken?: () => string };
  const images = Object.keys(require.cache)
    .filter((name) => name.endsWith(".node") && basename(name).startsWith("vize-vitrine."))
    .map(fileImage);
  const sharedObjects = (
    process.report!.getReport() as unknown as { sharedObjects: string[] }
  ).sharedObjects.map((name) => {
    try {
      return realpathSync(name);
    } catch {
      return name;
    }
  });
  const actual = binding.getNativeBuildToken?.();
  save(join(directory, "native-identity.json"), {
    token: actual,
    images,
    sharedObjects,
    nativeEntrypoint: require.resolve("@vizejs/native"),
    node: fileImage(process.execPath),
  });
  assert.equal(actual, token, "actual loaded native token differs from reviewed build");
  assert.equal(images.length, 1);
  assert.ok(sharedObjects.includes(images[0].path), "expected NAPI image is not actually mapped");
}
async function tee(root: string, capture: string, token: string) {
  const directory = mkdtempSync(join(capture, "mapper-"));
  const input = openSync(join(directory, "stdin.raw"), "wx"),
    output = openSync(join(directory, "stdout.raw"), "wx"),
    errors = openSync(join(directory, "stderr.raw"), "wx");
  const preload =
    "data:text/javascript," +
    encodeURIComponent(
      `import { recordNativeIdentity } from ${JSON.stringify(pathToFileURL(self).href)}; recordNativeIdentity(${JSON.stringify(root)}, ${JSON.stringify(directory)}, ${JSON.stringify(token)});`,
    );
  const argv = [process.execPath, "--import", preload, join(root, "bin/vize"), "content-mapper"];
  const env = environment();
  env.VIZE_CONTENT_MAPPER_WORKER_CAPTURE = join(directory, "workers.json");
  save(join(directory, "spawn.json"), {
    argv,
    cwd: process.cwd(),
    workerCapture: env.VIZE_CONTENT_MAPPER_WORKER_CAPTURE,
  });
  const child = spawn(argv[0], argv.slice(1), {
    cwd: process.cwd(),
    env,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let error: string | null = null;
  child.once("error", (value) => {
    error = value.message;
  });
  const captureInput = (bytes: Buffer) => writeSync(input, bytes);
  process.stdin.on("data", captureInput);
  child.stdout.on("data", (bytes) => writeSync(output, bytes));
  child.stderr.on("data", (bytes) => writeSync(errors, bytes));
  process.stdin.pipe(child.stdin);
  child.stdout.pipe(process.stdout);
  child.stderr.pipe(process.stderr);
  child.stdin.on("error", (value) => {
    error = value.message;
  });
  const [status, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((done) =>
    child.once("close", (code, reason) => done([code, reason])),
  );
  process.stdin.unpipe(child.stdin);
  process.stdin.removeListener("data", captureInput);
  process.stdin.pause();
  closeSync(input);
  closeSync(output);
  closeSync(errors);
  save(join(directory, "result.json"), { pid: child.pid ?? null, status, signal, error });
  process.exitCode = status === 0 && !signal && !error ? 0 : 1;
}
export const teeCommand = (root: string, capture: string, token: string) => [
  process.execPath,
  self,
  "--tee",
  root,
  capture,
  token,
];
if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  assert.equal(process.argv[2], "--tee");
  const [root, capture, token] = process.argv.slice(3);
  assert.ok(root && capture && token);
  tee(root, capture, token).catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
