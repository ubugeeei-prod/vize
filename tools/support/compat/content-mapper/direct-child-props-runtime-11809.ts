import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import Module from "node:module";
import path from "node:path";

export const roles = ["official", "tsrs"] as const;
export const props = ["bench-count", "benchCount", "count"] as const;
export const states = ["wrong", "valid", "missing"] as const;
export const expectedCells = roles.flatMap((role) =>
  props.flatMap((prop) =>
    [false, true].flatMap((shim) => states.map((state) => `${role}-${prop}-${shim}-${state}`)),
  ),
);
export function assertInventory(cells: readonly string[]) {
  assert.equal(expectedCells.length, 36);
  assert.equal(new Set(cells).size, cells.length, "duplicate cell");
  assert.deepEqual([...cells].sort(), [...expectedCells].sort(), "complete fixed cell inventory");
}
export function expectedOutput(state: (typeof states)[number]) {
  return state === "wrong"
    ? "PropsParent.vue(4,24): error TS2322: Type 'string' is not assignable to type 'number'.\n"
    : state === "missing"
      ? "Missing.vue(4,20): error TS2339: Property 'missing' does not exist on type '{ present: number; }'.\n"
      : "";
}
export function assertOutput(
  status: number | null,
  stdout: Uint8Array,
  stderr: Uint8Array,
  expected: string,
) {
  assert.equal(status, expected === "" ? 0 : 2);
  assert.deepEqual(Buffer.from(stdout), Buffer.from(expected), "complete diagnostic bytes");
  assert.equal(stderr.length, 0, "complete stderr must be empty");
}
export const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export function identity(file: string) {
  const physical = fs.realpathSync(file);
  const bytes = fs.readFileSync(physical);
  return { path: physical, bytes: bytes.length, sha256: sha256(bytes) };
}
export function physicalHost(
  owner: NodeJS.Require,
  name: string,
  version: string,
  relativeExecutable: string,
) {
  const manifest = owner.resolve(`${name}/package.json`);
  const value = JSON.parse(fs.readFileSync(manifest, "utf8")) as { name: string; version: string };
  assert.equal(value.name, name);
  assert.equal(value.version, version);
  // The package exposes its manifest, while the CLI is a pinned physical file.
  const executable = path.join(path.dirname(manifest), relativeExecutable);
  return { manifest: identity(manifest), executable: identity(executable) };
}
export function archiveIdentity(file: string, expectedIntegrity: string) {
  const integrity = `sha512-${createHash("sha512").update(fs.readFileSync(file)).digest("base64")}`;
  assert.equal(integrity, expectedIntegrity, "actual tarball must match npm's SRI");
  return { ...identity(file), integrity };
}
export function lockedPackage(lock: unknown, key: string, version: string) {
  const packages = (lock as { packages?: Record<string, unknown> }).packages;
  const value = packages?.[key] as
    | { version?: string; integrity?: string; resolved?: string }
    | undefined;
  assert.ok(value, `missing physical package lock entry: ${key}`);
  assert.equal(value.version, version);
  assert.match(value.integrity ?? "", /^sha512-[A-Za-z0-9+/]+=*$/u);
  assert.equal(typeof value.resolved, "string");
  return value;
}
export const writeJson = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
export function run(
  command: string,
  args: string[],
  cwd: string,
  capture: string,
  env: NodeJS.ProcessEnv,
) {
  fs.mkdirSync(capture, { recursive: true });
  const result = spawnSync(command, args, {
    cwd,
    env,
    timeout: 120_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(capture, "stdout.bin"), result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(path.join(capture, "stderr.bin"), result.stderr ?? Buffer.alloc(0));
  writeJson(path.join(capture, "process.json"), {
    command,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  });
  assert.equal(result.error, undefined, capture);
  assert.equal(result.signal, null, capture);
  return result;
}
export function successful(result: ReturnType<typeof run>) {
  assert.equal(result.status, 0, result.stderr.toString());
}
export interface NativeObservation {
  runtime: {
    version: string;
    platform: string;
    arch: string;
    executable: ReturnType<typeof identity>;
  };
  image: ReturnType<typeof identity>;
  token: string;
  argv: string[];
  cwd: string;
  pid: number;
}
export function assertNativeObservation(
  observed: NativeObservation,
  node: ReturnType<typeof identity>,
  imageSha256: string,
  token: string,
  cli: string,
) {
  assert.deepEqual(observed.runtime, {
    version: "v22.23.3",
    platform: "linux",
    arch: "x64",
    executable: node,
  });
  assert.equal(observed.image.sha256, imageSha256);
  assert.match(observed.token, /^[a-f0-9]{32}$/u);
  assert.equal(observed.token, token, "opaque image token; not a source revision");
  assert.equal(observed.argv[1], cli);
  assert.equal(observed.argv[2], "content-mapper");
  assert.equal(observed.argv.length, 3);
  assert.ok(Number.isSafeInteger(observed.pid) && observed.pid > 0);
}

// Only the separate observation contrasts enable this preload. Primary cells
// run without NODE_OPTIONS. Delegate once and preserve the native exports.
const trace = process.env.VIZE_TEST_CHILD_PROPS_TRACE;
if (trace) {
  type NativeModule = { exports: Record<string, unknown> };
  type Loader = (module: NativeModule, filename: string) => void;
  const loaders = (Module as unknown as { _extensions: Record<string, Loader> })._extensions;
  const original = loaders[".node"];
  assert.equal(typeof original, "function");
  assert.ok(original);
  let sequence = 0;
  loaders[".node"] = (module, filename) => {
    original(module, filename);
    if (!path.basename(filename).startsWith("vize-vitrine.")) return;
    const getter = module.exports["getNativeBuildToken"];
    assert.equal(typeof getter, "function");
    const token: unknown = Reflect.apply(
      getter as (...args: never[]) => unknown,
      module.exports,
      [],
    );
    assert.equal(typeof token, "string");
    fs.mkdirSync(trace, { recursive: true });
    writeJson(path.join(trace, `${process.pid}-${sequence++}.json`), {
      runtime: {
        version: process.version,
        platform: process.platform,
        arch: process.arch,
        executable: identity(process.execPath),
      },
      image: identity(filename),
      token,
      argv: process.argv,
      cwd: process.cwd(),
      pid: process.pid,
    });
  };
}
