// Qualify the actual source-built addon before the existing Nuxt fixture runs.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../../../npm/native/scripts/formatter-history-build.mts";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  asStringArray,
  at,
  readJson,
} from "../../json-value.ts";

export interface SourceFixture {
  filename: string;
  source: string;
}
export interface SourceBindingCustody {
  schema: string;
  version: number;
  source: { head: string; [key: string]: unknown };
  profile: string;
  features: unknown;
  binary: { path: string; sha256: string };
  calls: string;
  backends?: readonly string[];
  fixtures: SourceFixture[];
}
/** Narrow a custody file written by `prepareNuxtSourceBinding`. */
export function parseSourceBindingCustody(value: unknown): SourceBindingCustody {
  const custody = asRecord(value, "source-binding custody");
  const source = asRecord(custody.source, "custody.source");
  const binary = asRecord(custody.binary, "custody.binary");
  return {
    ...custody,
    schema: asString(custody.schema, "custody.schema"),
    version: asNumber(custody.version, "custody.version"),
    source: { ...source, head: asString(source.head, "custody.source.head") },
    profile: asString(custody.profile, "custody.profile"),
    features: custody.features,
    binary: {
      path: asString(binary.path, "custody.binary.path"),
      sha256: asString(binary.sha256, "custody.binary.sha256"),
    },
    calls: asString(custody.calls, "custody.calls"),
    backends:
      custody.backends === undefined ? undefined : asStringArray(custody.backends, "backends"),
    fixtures: asArray(custody.fixtures, "custody.fixtures").map((entry) => {
      const fixture = asRecord(entry, "fixture");
      return {
        filename: asString(fixture.filename, "fixture.filename"),
        source: asString(fixture.source, "fixture.source"),
      };
    }),
  };
}

export function readSourceBindingCustody(file: string): SourceBindingCustody {
  return parseSourceBindingCustody(readJson(file));
}

export interface SourceBindingOptions {
  backends?: readonly string[];
  fixtureDirectory?: string;
}

const sha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const preload = fileURLToPath(new URL("./source-binding-preload.cjs", import.meta.url));

export function prepareNuxtSourceBinding(
  root: string,
  artifacts: string,
  options: SourceBindingOptions = {},
) {
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_NUXT_NATIVE_CUSTODY",
  ])
    assert.ok(!process.env[key], `ambient ${key} cannot qualify the source binding`);
  const nativeDir = path.join(root, "npm/native");
  const receipt = asRecord(readJson(nativeHistoryReceipt(nativeDir)), "native history receipt");
  validateNativeHistoryBuild(nativeDir, receipt);
  const source = asRecord(receipt.source, "receipt.source");
  const head = asString(source.head, "receipt.source.head");
  if (process.env.GITHUB_SHA) assert.equal(head, process.env.GITHUB_SHA);
  const frozenPath = asString(at(receipt, "frozen", "path"), "receipt.frozen.path");
  const frozenSha256 = asString(at(receipt, "frozen", "sha256"), "receipt.frozen.sha256");
  const directory = path.join(artifacts, "source-native");
  fs.mkdirSync(directory);
  const binary = path.join(directory, "source-native.node");
  fs.copyFileSync(frozenPath, binary, fs.constants.COPYFILE_EXCL);
  assert.equal(sha256(fs.readFileSync(binary)), frozenSha256);
  for (const name of [
    "build-receipt.json",
    "cargo.stdout.bin",
    "cargo.stderr.bin",
    "build-process.json",
  ])
    fs.copyFileSync(path.join(nativeHistoryDirectory(nativeDir), name), path.join(directory, name));
  const custody: SourceBindingCustody = {
    schema: "vize.nuxt.source-binding",
    version: 1,
    source: { ...source, head },
    profile: "dev",
    features: at(receipt, "emitted", "cargo", "features"),
    binary: { path: fs.realpathSync(binary), sha256: frozenSha256 },
    calls: path.join(directory, "native-calls.jsonl"),
    backends: options.backends ?? ["client", "ssr"],
    fixtures: execFileSync(
      "git",
      [
        "ls-files",
        "-z",
        "--",
        options.fixtureDirectory ?? "tools/support/compat/nuxt/fixtures/nuxt3-module-build",
      ],
      { cwd: root },
    )
      .toString()
      .split("\0")
      .filter((name) => name.endsWith(".vue"))
      .map((name) => ({
        filename: fs.realpathSync(path.join(root, name)),
        source: fs.readFileSync(path.join(root, name), "utf8"),
      })),
  };
  const configuration = path.join(directory, "custody.json");
  fs.writeFileSync(configuration, JSON.stringify(custody, null, 2) + "\n", { flag: "wx" });
  return {
    custody,
    environment: {
      ...process.env,
      VIZE_NUXT_NATIVE_CUSTODY: configuration,
      NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(preload)}`.trim(),
    },
    verify() {
      validateNativeHistoryBuild(nativeDir, receipt);
      assert.equal(sha256(fs.readFileSync(binary)), custody.binary.sha256);
      const events = fs
        .readFileSync(custody.calls, "utf8")
        .trim()
        .split("\n")
        .map((line): unknown => JSON.parse(line));
      return verifyNuxtSourceBindingEvents(custody, events);
    },
  };
}

interface CompilerInput {
  filename: unknown;
  source: unknown;
}

function compilerInputs(entrypoint: unknown, args: unknown[]): CompilerInput[] {
  if (entrypoint === "compileSfc") return [{ filename: at(args, 1, "filename"), source: args[0] }];
  return asArray(args[0], "batch inputs").map((input) => ({
    filename: at(input, "path"),
    source: at(input, "source"),
  }));
}

export function verifyNuxtSourceBindingEvents(
  custody: SourceBindingCustody,
  events: readonly unknown[],
) {
  assert.equal(custody.schema, "vize.nuxt.source-binding");
  assert.equal(custody.version, 1);
  assert.match(custody.source.head, /^[a-f0-9]{40}$/);
  assert.match(custody.binary.sha256, /^[a-f0-9]{64}$/);
  assert.ok(events.length > 0, "no physical source-addon load was recorded");
  const loaded = new Set<number>();
  const compiled = new Set<string>();
  const backends = custody.backends ?? ["client", "ssr"];
  assert.ok(Array.isArray(backends) && backends.length > 0);
  assert.equal(new Set(backends).size, backends.length);
  for (const backend of backends) assert.ok(["client", "ssr"].includes(backend));
  assert.ok(custody.fixtures.length > 0, "no original fixture SFCs were retained");
  let calls = 0;
  for (const value of events) {
    const event = asRecord(value, "native event");
    assert.equal(event.sourceHead, custody.source.head);
    assert.equal(event.binary, custody.binary.path);
    assert.equal(event.sha256, custody.binary.sha256);
    const pid = event.pid;
    assert.ok(typeof pid === "number" && Number.isSafeInteger(pid) && pid > 0);
    if (event.kind === "load") loaded.add(pid);
    else {
      assert.equal(event.kind, "call");
      assert.ok(loaded.has(pid), "compiler call has no original process load");
      assert.ok(
        event.entrypoint === "compileSfc" || event.entrypoint === "compileSfcBatchWithResults",
      );
      assert.equal(event.outcome, "return", "source compiler threw during the fixture build");
      const args = asArray(event.args, "event.args");
      const result = event.result;
      assert.ok(result && typeof result === "object");
      if (event.entrypoint === "compileSfcBatchWithResults") assert.ok(Array.isArray(args[0]));
      const inputs = compilerInputs(event.entrypoint, args);
      const backend = at(args, 1, "ssr") === true ? "ssr" : "client";
      for (const input of inputs) {
        const original = custody.fixtures.find((fixture) => fixture.filename === input.filename);
        if (original) {
          assert.equal(
            input.source,
            original.source,
            "compiler input differs from original fixture bytes",
          );
          const batch = at(result, "results");
          const results: unknown[] | undefined =
            event.entrypoint === "compileSfc"
              ? [result]
              : Array.isArray(batch)
                ? batch.filter((entry: unknown) => at(entry, "path") === original.filename)
                : undefined;
          assert.ok(
            Array.isArray(results) && results.length === 1,
            "missing original native result",
          );
          const native: unknown = results[0];
          assert.deepEqual(
            at(native, "errors"),
            [],
            "original fixture returned native compiler errors",
          );
          const code = at(native, "code");
          assert.ok(typeof code === "string", "native generated code must be a string");
          assert.ok(code.trim(), "original fixture returned no native generated code");
          compiled.add(`${backend}:${original.filename}`);
        }
      }
      calls++;
    }
  }
  assert.ok(calls > 0, "fixture build never called the source SFC compiler");
  for (const fixture of custody.fixtures)
    for (const backend of backends)
      assert.ok(
        compiled.has(`${backend}:${fixture.filename}`),
        `source compiler missed ${backend} original fixture ${fixture.filename}`,
      );
  return {
    source: custody.source,
    binary: custody.binary,
    profile: custody.profile,
    backends,
    calls,
    processes: loaded.size,
    fixtureTargets: [...compiled].sort((left, right) => (left < right ? -1 : Number(left > right))),
  };
}
