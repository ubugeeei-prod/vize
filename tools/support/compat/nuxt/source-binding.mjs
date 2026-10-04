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
} from "../../../../npm/native/scripts/formatter-history-build.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const preload = fileURLToPath(new URL("./source-binding-preload.cjs", import.meta.url));

export function prepareNuxtSourceBinding(root, artifacts) {
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_NUXT_NATIVE_CUSTODY",
  ])
    assert.ok(!process.env[key], `ambient ${key} cannot qualify the source binding`);
  const nativeDir = path.join(root, "npm/native");
  const receipt = JSON.parse(fs.readFileSync(nativeHistoryReceipt(nativeDir), "utf8"));
  validateNativeHistoryBuild(nativeDir, receipt);
  if (process.env.GITHUB_SHA) assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  const directory = path.join(artifacts, "source-native");
  fs.mkdirSync(directory);
  const binary = path.join(directory, "source-native.node");
  fs.copyFileSync(receipt.frozen.path, binary, fs.constants.COPYFILE_EXCL);
  assert.equal(sha256(fs.readFileSync(binary)), receipt.frozen.sha256);
  for (const name of [
    "build-receipt.json",
    "cargo.stdout.bin",
    "cargo.stderr.bin",
    "build-process.json",
  ])
    fs.copyFileSync(path.join(nativeHistoryDirectory(nativeDir), name), path.join(directory, name));
  const custody = {
    schema: "vize.nuxt.source-binding",
    version: 1,
    source: receipt.source,
    profile: "dev",
    features: receipt.emitted.cargo.features,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
    calls: path.join(directory, "native-calls.jsonl"),
    fixtures: execFileSync(
      "git",
      ["ls-files", "-z", "--", "tools/support/compat/nuxt/fixtures/nuxt3-module-build"],
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
      NAPI_RS_NATIVE_LIBRARY_PATH: binary,
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
        .map((line) => JSON.parse(line));
      return verifyNuxtSourceBindingEvents(custody, events);
    },
  };
}

export function verifyNuxtSourceBindingEvents(custody, events) {
  assert.equal(custody.schema, "vize.nuxt.source-binding");
  assert.equal(custody.version, 1);
  assert.match(custody.source.head, /^[a-f0-9]{40}$/);
  assert.match(custody.binary.sha256, /^[a-f0-9]{64}$/);
  assert.ok(events.length > 0, "no physical source-addon load was recorded");
  const loaded = new Set();
  const compiled = new Set();
  assert.ok(custody.fixtures.length > 0, "no original fixture SFCs were retained");
  let calls = 0;
  for (const event of events) {
    assert.equal(event.sourceHead, custody.source.head);
    assert.equal(event.binary, custody.binary.path);
    assert.equal(event.sha256, custody.binary.sha256);
    assert.ok(Number.isSafeInteger(event.pid) && event.pid > 0);
    if (event.kind === "load") loaded.add(event.pid);
    else {
      assert.equal(event.kind, "call");
      assert.ok(loaded.has(event.pid), "compiler call has no original process load");
      assert.ok(["compileSfc", "compileSfcBatchWithResults"].includes(event.entrypoint));
      assert.equal(event.outcome, "return", "source compiler threw during the fixture build");
      assert.ok(Array.isArray(event.args));
      assert.ok(event.result && typeof event.result === "object");
      if (event.entrypoint === "compileSfcBatchWithResults")
        assert.ok(Array.isArray(event.args[0]));
      const inputs =
        event.entrypoint === "compileSfc"
          ? [{ filename: event.args[1]?.filename, source: event.args[0] }]
          : event.args[0].map((input) => ({ filename: input.path, source: input.source }));
      const backend = event.args[1]?.ssr === true ? "ssr" : "client";
      for (const input of inputs) {
        const original = custody.fixtures.find((fixture) => fixture.filename === input.filename);
        if (original) {
          assert.equal(
            input.source,
            original.source,
            "compiler input differs from original fixture bytes",
          );
          compiled.add(`${backend}:${original.filename}`);
        }
      }
      calls++;
    }
  }
  assert.ok(calls > 0, "fixture build never called the source SFC compiler");
  for (const fixture of custody.fixtures)
    for (const backend of ["client", "ssr"])
      assert.ok(
        compiled.has(`${backend}:${fixture.filename}`),
        `source compiler missed ${backend} original fixture ${fixture.filename}`,
      );
  return {
    source: custody.source,
    binary: custody.binary,
    profile: custody.profile,
    calls,
    processes: loaded.size,
    fixtureTargets: [...compiled].sort(),
  };
}
