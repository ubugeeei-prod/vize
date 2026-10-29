import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
export type RecordValue = Record<string, unknown>;
export type Image = { file: string; bytes: number; sha256: string };
function isRecord(value: unknown): value is RecordValue {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
export function record(value: unknown): RecordValue {
  assert.ok(isRecord(value));
  return value;
}
export function string(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}
export function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
export function read(file: string): unknown {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  return value;
}
export const sha = (value: string | Buffer): string =>
  createHash("sha256").update(value).digest("hex");
export function image(file: string): Image {
  const actual = fs.realpathSync(file),
    bytes = fs.readFileSync(actual);
  return { file: actual, bytes: bytes.length, sha256: sha(bytes) };
}
export function save(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
export function run(
  output: string,
  label: string,
  executable: string,
  argv: string[],
  cwd: string,
  input?: Buffer,
): { stdout: Buffer; stderr: Buffer; status: number } {
  const result = spawnSync(executable, argv, {
    cwd,
    input,
    env: process.env,
    timeout: 90_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0),
    prefix = path.join(output, label);
  fs.mkdirSync(path.dirname(prefix), { recursive: true });
  fs.writeFileSync(prefix + ".stdin", input ?? Buffer.alloc(0));
  fs.writeFileSync(prefix + ".stdout", stdout);
  fs.writeFileSync(prefix + ".stderr", stderr);
  save(prefix + ".process.json", {
    executable: image(executable),
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutSha256: sha(stdout),
    stderrSha256: sha(stderr),
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.ok(result.status === 0 || result.status === 1 || result.status === 2);
  return { stdout, stderr, status: result.status };
}
function call(owner: RecordValue, key: string, args: unknown[]): unknown {
  const fn = owner[key];
  assert.ok(typeof fn === "function");
  const value: unknown = Reflect.apply(fn, owner, args);
  return value;
}
function within(root: string, file: string): void {
  const relative = path.relative(fs.realpathSync(root), fs.realpathSync(file));
  assert.ok(
    relative && !relative.startsWith("..") && !path.isAbsolute(relative),
    "public dependency must remain inside the fresh consumer",
  );
}
export function authenticate(
  workspace: string,
  version: string,
): {
  cli: string;
  provider: string;
  root: string;
  native: Image;
  cpp: RecordValue;
  metadata: RecordValue;
} {
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "NODE_OPTIONS",
    "NODE_PATH",
  ])
    assert.ok(!process.env[key], key + " must not override the public consumer");
  assert.equal(process.platform, "linux");
  const cpp = record(read(path.join(workspace, "acceptance.json")));
  assert.equal(cpp.version, version);
  assert.equal(cpp.status, "pass");
  assert.equal(cpp.originalErrorCount, 8);
  for (const name of ["vize", "@vizejs/native"])
    assert.equal(
      record(read(path.join(workspace, "node_modules", name, "package.json"))).version,
      version,
    );
  assert.equal(
    record(read(path.join(workspace, "node_modules/vue/package.json"))).version,
    "3.5.42",
  );
  const required = createRequire(path.join(workspace, "package.json"));
  const binding = record(required("@vizejs/native"));
  const identity = record(
    call(record(required("@vizejs/native/native-binding-identity.js")), "getBindingIdentity", [
      binding,
    ]),
  );
  assert.equal(identity.kind, "native");
  const token = string(call(binding, "getNativeBuildToken", []));
  assert.ok(token.length > 0);
  assert.equal(string(identity.token), token);
  const prior = record(cpp.nativeIdentity);
  assert.equal(token, string(prior.token));
  const loaded = Object.keys(required.cache).filter(
    (file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine."),
  );
  assert.equal(loaded.length, 1);
  const loadedFile = loaded[0];
  assert.ok(loadedFile);
  within(path.join(workspace, "node_modules"), loadedFile);
  assert.equal(fs.realpathSync(loadedFile), fs.realpathSync(string(prior.binary)));
  assert.equal(
    record(read(path.join(path.dirname(fs.realpathSync(loadedFile)), "package.json"))).version,
    version,
  );
  const native = image(loadedFile),
    entries = array(cpp.imageManifest).map(record);
  const nativeEntry = entries.find((entry) => fs.realpathSync(string(entry.file)) === native.file);
  assert.ok(nativeEntry);
  assert.equal(nativeEntry.sha256, native.sha256);
  assert.equal(nativeEntry.bytes, native.bytes);
  const providerEntry = entries.find((entry) => !string(entry.file).endsWith(".node"));
  assert.ok(providerEntry);
  const provider = fs.realpathSync(string(providerEntry.file));
  within(path.join(workspace, "node_modules/@typescript/typescript-linux-x64"), provider);
  assert.equal(
    record(read(path.join(workspace, "node_modules/@typescript/typescript-linux-x64/package.json")))
      .version,
    "7.0.2",
  );
  assert.equal(image(provider).sha256, providerEntry.sha256);
  const root = fs.realpathSync(path.join(workspace, "node_modules/vize")),
    cli = path.join(root, "bin/vize");
  within(path.join(workspace, "node_modules"), root);
  assert.ok(fs.existsSync(cli));
  const mapper = record(
    record(record(read(path.join(root, "package.json"))).typescript).contentMapper,
  );
  assert.deepEqual(array(mapper.exec), ["node", "./bin/vize", "content-mapper"]);
  const projectVue = fs.realpathSync(path.join(workspace, "node_modules/vue/package.json"));
  const helperVue = fs.realpathSync(
    createRequire(path.join(root, "package.json")).resolve("vue/package.json"),
  );
  assert.equal(projectVue, helperVue, "the public helper and consumer must use one physical Vue");
  return {
    cli,
    provider,
    root,
    native,
    cpp,
    metadata: {
      nativeIdentity: identity,
      nativeImage: native,
      providerImage: image(provider),
      vueManifest: image(projectVue),
      cli: image(cli),
      package: image(path.join(root, "package.json")),
      lock: image(path.join(workspace, "package-lock.json")),
    },
  };
}
