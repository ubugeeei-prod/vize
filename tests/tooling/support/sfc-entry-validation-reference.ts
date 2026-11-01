import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

export const CASES_SHA256 = "a16ca0e07eaefff3925e7f6a015b6e22396034c4915a6ed518ecc7cc7f9e4710";
export const REFERENCE_VERSION = "3.6.0-rc.14";
export type SfcEntryExpectation = "export-error" | "missing-block" | "valid";
export interface SfcEntryCase {
  id: string;
  file: string;
  filename: string;
  issue: number;
  original: boolean;
  expect: SfcEntryExpectation;
  bytes: number;
  sha256: string;
  source: string;
}
export const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);
const text = (value: unknown): string => {
  assert(typeof value === "string");
  return value;
};

export function loadSfcEntryCases(root: string): readonly SfcEntryCase[] {
  const directory = path.join(
    root,
    "tests/_fixtures/differential/compiler/sfc-entry-validation-11262-11263",
  );
  const catalog = fs.readFileSync(path.join(directory, "cases.json"));
  assert.equal(sha256(catalog), CASES_SHA256, "the complete frozen corpus catalog");
  const parsed: unknown = JSON.parse(catalog.toString("utf8"));
  assert(isRecord(parsed));
  assert.equal(parsed.schema, "vize.sfc-entry-validation-originals");
  assert.equal(parsed.version, 1);
  assert(Array.isArray(parsed.cases));
  assert.equal(parsed.cases.length, 20);
  const cases = parsed.cases.map((value: unknown): SfcEntryCase => {
    assert(isRecord(value));
    const file = text(value.file),
      filename = text(value.filename);
    assert.match(file, /^[A-Za-z]+\.vue\.txt$/);
    assert.equal(filename, file.slice(0, -4));
    const expected = value.expect;
    assert(expected === "valid" || expected === "export-error" || expected === "missing-block");
    assert(typeof value.issue === "number" && Number.isInteger(value.issue));
    assert(typeof value.original === "boolean");
    assert(typeof value.bytes === "number" && Number.isInteger(value.bytes) && value.bytes >= 0);
    const bytes = fs.readFileSync(path.join(directory, file));
    assert.equal(bytes.length, value.bytes, file);
    assert.equal(sha256(bytes), value.sha256, file);
    const source = bytes.toString("utf8");
    assert.deepEqual(Buffer.from(source), bytes, "no UTF8 replacement or newline normalization");
    return {
      id: text(value.id),
      file,
      filename,
      issue: value.issue,
      original: value.original,
      expect: expected,
      bytes: bytes.length,
      sha256: text(value.sha256),
      source,
    };
  });
  assert.equal(new Set(cases.map((entry) => entry.id)).size, 20);
  assert.equal(cases.filter((entry) => entry.original).length, 8);
  return cases;
}

export type WholeValue =
  | null
  | boolean
  | number
  | string
  | WholeValue[]
  | { [key: string]: WholeValue };

/** Every own property/descriptor survives, including nonenumerable Error data.
 * Functions retain source/own properties, not callable closure state or VM internals.
 * General getters are recorded rather than invoked; standard Error text/stack
 * is also observed explicitly. References and symbols retain identity.
 */
export function captureWholeValue(input: unknown): WholeValue {
  const seen = new Map<object, number>(),
    symbols = new Map<symbol, number>();
  const capture = (value: unknown): WholeValue => {
    if (value === null || typeof value === "string" || typeof value === "boolean") return value;
    if (typeof value === "undefined") return { type: "undefined" };
    if (typeof value === "number")
      return Number.isFinite(value) && !Object.is(value, -0)
        ? value
        : { type: "number", value: Object.is(value, -0) ? "-0" : String(value) };
    if (typeof value === "bigint") return { type: "bigint", value: value.toString() };
    if (typeof value === "symbol") {
      const previous = symbols.get(value);
      if (previous !== undefined) return { symbolRef: previous };
      const id = symbols.size;
      symbols.set(value, id);
      return {
        symbol: id,
        description: value.description ?? null,
        globalKey: Symbol.keyFor(value) ?? null,
      };
    }
    assert(typeof value === "object" || typeof value === "function");
    const previous = seen.get(value);
    if (previous !== undefined) return { ref: previous };
    const id = seen.size;
    seen.set(value, id);
    const prototype: unknown = Object.getPrototypeOf(value);
    const constructor =
      prototype === null ? undefined : Object.getOwnPropertyDescriptor(prototype, "constructor");
    const constructorValue: unknown = constructor?.value;
    const output: { [key: string]: WholeValue } = {
      id,
      kind: Object.prototype.toString.call(value),
      prototype: typeof constructorValue === "function" ? constructorValue.name : null,
      properties: Reflect.ownKeys(value).map((key): WholeValue => {
        const descriptor = Object.getOwnPropertyDescriptor(value, key);
        assert(descriptor);
        const property: { [key: string]: WholeValue } = {
          key: capture(key),
          enumerable: !!descriptor.enumerable,
          configurable: !!descriptor.configurable,
        };
        if ("value" in descriptor) {
          const data: unknown = descriptor.value;
          property.writable = !!descriptor.writable;
          property.value = capture(data);
        } else {
          property.get = capture(descriptor.get);
          property.set = capture(descriptor.set);
        }
        return property;
      }),
    };
    if (typeof value === "function") output.source = Function.prototype.toString.call(value);
    if (value instanceof Error) {
      output.errorName = capture(value.name);
      output.errorMessage = capture(value.message);
      output.errorStack = capture(value.stack);
    }
    if (value instanceof Map) {
      const map: ReadonlyMap<unknown, unknown> = value;
      output.entries = [...map].map(([key, item]) => [capture(key), capture(item)]);
    }
    if (value instanceof Set) {
      const set: ReadonlySet<unknown> = value;
      output.values = [...set].map(capture);
    }
    if (value instanceof Date) output.time = capture(value.getTime());
    if (value instanceof RegExp) {
      output.source = value.source;
      output.flags = value.flags;
    }
    if (value instanceof ArrayBuffer) output.binary = Buffer.from(value).toString("base64");
    if (ArrayBuffer.isView(value))
      output.binary = Buffer.from(value.buffer, value.byteOffset, value.byteLength).toString(
        "base64",
      );
    return output;
  };
  return capture(input);
}

export interface ReferenceCompiler {
  version: string;
  parse(source: string, options: { filename: string; sourceMap: false }): unknown;
  compileScript(descriptor: unknown, options: { id: string; sourceMap: false }): unknown;
}
type RecordPhase = (phase: string, value: unknown) => void;
const PACKAGE = "@vue/compiler-sfc",
  REGISTRY = "https://registry.npmjs.org";

/** Called only by the Actions qualification. No checkout/store provider fallback. */
export async function acquireSfcEntryReference(record: RecordPhase) {
  assert.equal(
    process.env.GITHUB_ACTIONS,
    "true",
    "rc14 installation/reference execution is Actions-only",
  );
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-sfc-entry-reference-")),
  );
  const metadataUrl = `${REGISTRY}/@vue%2fcompiler-sfc/${REFERENCE_VERSION}`;
  const response = await fetch(metadataUrl, {
    redirect: "error",
    signal: AbortSignal.timeout(30_000),
  });
  assert(response.body, "exact-version registry response has a body");
  const reader = response.body.getReader(),
    chunks: Uint8Array[] = [];
  let received = 0;
  try {
    for (;;) {
      const next = await reader.read();
      if (next.done) break;
      received += next.value.byteLength;
      if (received > 1024 * 1024) {
        record("registry-cap-refusal", {
          url: metadataUrl,
          status: response.status,
          received,
          partialBase64: Buffer.concat(chunks).toString("base64"),
        });
        await reader.cancel();
        throw new Error("exact-version registry metadata exceeded 1 MiB");
      }
      chunks.push(next.value);
    }
  } finally {
    reader.releaseLock();
  }
  const metadataText = Buffer.concat(chunks).toString("utf8");
  record("registry", {
    url: metadataUrl,
    status: response.status,
    headers: [...response.headers],
    body: metadataText,
  });
  assert.equal(response.status, 200);
  assert(Buffer.byteLength(metadataText) <= 1024 * 1024, "bounded exact-version metadata");
  const metadata: unknown = JSON.parse(metadataText);
  assert(isRecord(metadata) && isRecord(metadata.dist));
  assert.equal(metadata.name, PACKAGE);
  assert.equal(metadata.version, REFERENCE_VERSION);
  const integrity = text(metadata.dist.integrity),
    tarball = text(metadata.dist.tarball);
  assert.match(integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/);
  assert.equal(new URL(tarball).origin, REGISTRY);
  fs.writeFileSync(
    path.join(directory, "package.json"),
    JSON.stringify({ name: "vize-sfc-entry-reference", private: true, version: "1.0.0" }),
  );
  const config = path.join(directory, ".npmrc");
  fs.writeFileSync(config, "");
  const globalConfig = path.join(directory, "global.npmrc");
  fs.writeFileSync(globalConfig, "");
  const env = Object.fromEntries(
    Object.entries(process.env).filter(
      ([key]) => !/^npm_config_/i.test(key) && key !== "NODE_PATH" && key !== "NODE_OPTIONS",
    ),
  );
  Object.assign(env, {
    npm_config_registry: REGISTRY,
    npm_config_userconfig: config,
    npm_config_globalconfig: globalConfig,
    npm_config_cache: path.join(directory, "cache"),
  });
  const npm = (args: string[]) => {
    const command = process.platform === "win32" ? "npm.cmd" : "npm";
    const result = spawnSync(command, args, {
      cwd: directory,
      env,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 8 * 1024 * 1024,
      shell: process.platform === "win32",
    });
    record("npm", {
      command,
      args,
      status: result.status,
      signal: result.signal,
      stdout: result.stdout,
      stderr: result.stderr,
      error: captureWholeValue(result.error),
    });
    assert.equal(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
  };
  npm([
    "install",
    "--package-lock-only",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    "--save-exact",
    `${PACKAGE}@${REFERENCE_VERSION}`,
  ]);
  const lockPath = path.join(directory, "package-lock.json"),
    lockBytes = fs.readFileSync(lockPath);
  record("dependency-lock", {
    bytes: lockBytes.length,
    sha256: sha256(lockBytes),
    body: lockBytes.toString("utf8"),
  });
  const lock: unknown = JSON.parse(lockBytes.toString("utf8"));
  assert(isRecord(lock) && isRecord(lock.packages));
  const target = lock.packages[`node_modules/${PACKAGE}`];
  assert(isRecord(target));
  assert.equal(target.version, REFERENCE_VERSION);
  assert.equal(target.integrity, integrity);
  assert.equal(target.resolved, tarball);
  npm(["ci", "--ignore-scripts", "--no-audit", "--no-fund"]);
  assert.deepEqual(
    fs.readFileSync(lockPath),
    lockBytes,
    "install consumes the unchanged complete lock",
  );
  const fromProvider = createRequire(path.join(directory, "package.json"));
  const packagePath = fromProvider.resolve(`${PACKAGE}/package.json`),
    main = fromProvider.resolve(PACKAGE);
  assert(fs.realpathSync(main).startsWith(`${fs.realpathSync(directory)}${path.sep}`));
  const packageBytes = fs.readFileSync(packagePath),
    packageData: unknown = JSON.parse(packageBytes.toString("utf8"));
  assert(isRecord(packageData));
  assert.equal(packageData.name, PACKAGE);
  assert.equal(packageData.version, REFERENCE_VERSION);
  const mainBytes = fs.readFileSync(main);
  record("provider-source", {
    package: PACKAGE,
    version: REFERENCE_VERSION,
    integrity,
    packageJson: {
      bytes: packageBytes.length,
      sha256: sha256(packageBytes),
      body: packageBytes.toString("utf8"),
    },
    entry: {
      path: path.relative(directory, main),
      bytes: mainBytes.length,
      sha256: sha256(mainBytes),
      base64Chunks: mainBytes.toString("base64").match(/.{1,4096}/g) ?? [],
    },
    node: process.versions,
    platform: process.platform,
    arch: process.arch,
    lockSha256: sha256(lockBytes),
  });
  const loaded: unknown = fromProvider(PACKAGE);
  assert(
    isRecord(loaded) &&
      typeof loaded.parse === "function" &&
      typeof loaded.compileScript === "function",
  );
  assert.equal(loaded.version, REFERENCE_VERSION);
  const compiler = loaded as unknown as ReferenceCompiler;
  return {
    compiler,
    loadedSourceReceipt: () => {
      const visited = new Set<NodeModule>();
      const visit = (module: NodeModule) => {
        if (visited.has(module)) return;
        visited.add(module);
        for (const child of module.children) visit(child);
      };
      const entry = fromProvider.cache[main];
      assert(entry);
      visit(entry);
      const files = [...visited]
        .map((module) => {
          const file = fs.realpathSync(module.filename),
            bytes = fs.readFileSync(file);
          return {
            path: path.relative(directory, file),
            bytes: bytes.length,
            sha256: sha256(bytes),
            insideProvider: file.startsWith(`${fs.realpathSync(directory)}${path.sep}`),
          };
        })
        .sort((left, right) => left.path.localeCompare(right.path));
      record("loaded-provider-source", files);
      assert(
        files.every((file) => file.insideProvider),
        "no checkout/store/ancestor provider dependency fallback",
      );
      return files;
    },
  };
}
