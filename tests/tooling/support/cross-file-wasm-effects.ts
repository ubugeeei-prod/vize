import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  wasmPackageCargoArgs,
  wasmPackageCargoEnvironment,
} from "../../../tools/support/tasks/build-vize-wasm-package.ts";
import { wasmCases } from "./cross-file-sfc-effects/wasm-corpus.ts";
import { expectedWasmPacket } from "./cross-file-sfc-effects/wasm-oracle.ts";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const output = path.join(repo, "target/vize-tests/cross-file-wasm-effects");
fs.mkdirSync(output, { recursive: true });
const hash = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");
function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function save(name: string, value: unknown): void {
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
}
function command(
  label: string,
  binary: string,
  args: string[],
  cwd: string,
  allowStderr = false,
): string {
  const result = spawnSync(binary, args, { cwd, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 });
  fs.writeFileSync(path.join(output, label + ".stdout"), result.stdout ?? "");
  fs.writeFileSync(path.join(output, label + ".stderr"), result.stderr ?? "");
  save(label + ".process.json", {
    binary,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `${label}: ${result.stderr}`);
  if (!allowStderr) assert.equal(result.stderr, "", label);
  return result.stdout;
}

const source = command("source-head", "git", ["rev-parse", "HEAD"], repo).trim();
assert.equal(source, process.env.EXPECTED_SOURCE_SHA);
command(
  "rust-source-clean",
  "git",
  ["diff", "--exit-code", "--", "crates/vize_vitrine/src/wasm"],
  repo,
);
const sourceDirectory = path.join(repo, "npm/wasm");
const sourceImage = fs.readFileSync(path.join(sourceDirectory, "vize_vitrine_bg.wasm"));
const sourceLoader = fs.readFileSync(path.join(sourceDirectory, "vize_vitrine.js"));
const cargoEnvironment = wasmPackageCargoEnvironment();
save("source-build.json", {
  source,
  recipe: "node tools/support/tasks/build-vize-wasm-package.ts",
  cargoArgs: wasmPackageCargoArgs("build"),
  compilerEnvironment: {
    RUSTFLAGS: cargoEnvironment.RUSTFLAGS,
    CARGO_PROFILE_RELEASE_PANIC: cargoEnvironment.CARGO_PROFILE_RELEASE_PANIC,
  },
  cargoVersion: command("cargo-version", "cargo", ["--version"], repo).trim(),
  rustcVersion: command("rustc-version", "rustc", ["--version"], repo).trim(),
  bindgenVersion: command("bindgen-version", "wasm-bindgen", ["--version"], repo).trim(),
  image: { bytes: sourceImage.length, sha256: hash(sourceImage) },
  loader: { bytes: sourceLoader.length, sha256: hash(sourceLoader) },
  cargoImageSha256: hash(
    fs.readFileSync(path.join(repo, "target/wasm32-unknown-unknown/release/vize_vitrine.wasm")),
  ),
});
const consumer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-wasm-effects-")));
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify({ private: true, dependencies: { "@vizejs/wasm": "0.565.0" } }),
);
command(
  "install-public",
  "npm",
  ["install", "--ignore-scripts", "--no-audit", "--no-fund"],
  consumer,
  true,
);
const lockText = fs.readFileSync(path.join(consumer, "package-lock.json"), "utf8");
fs.writeFileSync(path.join(output, "public-package-lock.json"), lockText);
const lock = record(record(JSON.parse(lockText)).packages);
const entry = record(lock["node_modules/@vizejs/wasm"]);
const response = await fetch("https://registry.npmjs.org/%40vizejs%2Fwasm/0.565.0", {
  signal: AbortSignal.timeout(30_000),
});
assert.equal(response.status, 200);
const manifest = record(await response.json());
assert.equal(entry.version, "0.565.0");
assert.equal(entry.integrity, record(manifest.dist).integrity);
assert.equal(entry.resolved, record(manifest.dist).tarball);
const installed = record(
  JSON.parse(
    fs.readFileSync(path.join(consumer, "node_modules/@vizejs/wasm/package.json"), "utf8"),
  ),
);
assert.equal(installed.name, "@vizejs/wasm");
assert.equal(installed.version, "0.565.0");
save("public-package.json", { manifest, lock: entry, installed });

interface WasmApi {
  default: (input: { module_or_path: Uint8Array }) => Promise<unknown>;
  analyzeCrossFile: (files: { path: string; source: string }[], options: object) => unknown;
}
async function load(directory: string, label: string): Promise<WasmApi> {
  const modulePath = path.join(directory, "vize_vitrine.js");
  const image = fs.readFileSync(path.join(directory, "vize_vitrine_bg.wasm"));
  const loaded: unknown = await import(pathToFileURL(modulePath).href);
  assert.equal(typeof record(loaded).default, "function");
  assert.equal(typeof record(loaded).analyzeCrossFile, "function");
  const api = loaded as WasmApi;
  save(label + ".module.json", {
    exports: Object.keys(record(loaded)).sort(),
    loaderSha256: hash(fs.readFileSync(modulePath)),
    wasmSha256: hash(image),
    wasmBytes: image.length,
  });
  await api.default({ module_or_path: image });
  return api;
}
const previous = await load(path.join(consumer, "node_modules/@vizejs/wasm"), "public");
const current = await load(sourceDirectory, "source");
assert.deepEqual(
  Object.keys(previous).sort(),
  Object.keys(current).sort(),
  "actual public exports stay unchanged",
);
const project = path.join(consumer, 'project-"日本語🙂');
fs.mkdirSync(project);
const observations: unknown[] = [];
for (const test of wasmCases) {
  for (const file of fs.readdirSync(project)) fs.rmSync(path.join(project, file));
  const inputs = Object.entries(test.files).map(([name, source]) => {
    fs.writeFileSync(path.join(project, name), source);
    return { path: name, source, sha256: hash(source) };
  });
  save(test.id + ".inputs.json", inputs);
  for (const [mode, options] of [
    ["minimal", {}],
    ["all", { all: true }],
  ] as const) {
    for (const order of ["public-first", "source-first"]) {
      const captured = new Map<string, { packet: unknown; elapsedNs: string }>();
      const sequence =
        order === "public-first"
          ? ([
              ["public", previous],
              ["source", current],
            ] as const)
          : ([
              ["source", current],
              ["public", previous],
            ] as const);
      for (const [label, api] of sequence) {
        const originalInputs = JSON.stringify(inputs);
        const started = process.hrtime.bigint();
        const packet = api.analyzeCrossFile(inputs, options);
        const elapsedNs = (process.hrtime.bigint() - started).toString();
        assert.equal(JSON.stringify(inputs), originalInputs, "API must retain input ownership");
        captured.set(label, { packet, elapsedNs });
        save(`${test.id}.${mode}.${order}.${label}.json`, packet);
      }
      const before = captured.get("public");
      const after = captured.get("source");
      assert.ok(before && after);
      const expected = expectedWasmPacket(test, before.packet);
      save(`${test.id}.${mode}.${order}.expected.json`, expected);
      assert.deepEqual(after.packet, expected, `${test.id}/${mode}/${order}: complete response`);
      assert.equal(
        JSON.stringify(after.packet),
        JSON.stringify(expected),
        "complete serialized response ordering",
      );
      if (test.before.score !== test.after.score)
        assert.notDeepEqual(
          before.packet,
          after.packet,
          "actual public provider must reproduce this effect-grammar defect",
        );
      if (test.diagnostics && mode === "all") {
        const diagnostics = record(before.packet).diagnostics;
        assert.ok(
          Array.isArray(diagnostics) && diagnostics.length > 0,
          "nonempty complete diagnostic preservation witness",
        );
      }
      observations.push({
        case: test.id,
        mode,
        order,
        before: test.before,
        after: test.after,
        publicWholeCallNs: before.elapsedNs,
        sourceWholeCallNs: after.elapsedNs,
        completeResponseSha256: hash(JSON.stringify(after.packet)),
      });
      save("observations.json", observations);
    }
  }
}
save("qualification.json", {
  passed: true,
  source,
  publicVersion: "0.565.0",
  cases: wasmCases.length,
  observations,
  note: "Only effect edges/cycles and their existing complexity projections change. Existing Croquis node/resource facts and full diagnostics are preserved. Whole-call observations include all adapter/analysis work; internal WASM phases and a speedup are not claimed.",
});
process.stdout.write(
  `cross-file WASM effects: ${wasmCases.length} cases, ${observations.length} complete observations passed\n`,
);
