import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../../differential/build-receipt.ts";
import { cases } from "./cross-file-sfc-effects/corpus.ts";
import { checkWasm, expectedReport } from "./cross-file-sfc-effects/oracle.ts";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const output = path.resolve(repo, "target/vize-tests/cross-file-sfc-effects");
fs.mkdirSync(output, { recursive: true });
const hash = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");
const statuses = new Map<string, number | null>();
function save(name: string, value: unknown): void {
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
}
function record(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function run(
  label: string,
  binary: string,
  args: string[],
  cwd: string,
  cleanStderr = true,
): Buffer {
  const result = spawnSync(binary, args, { cwd, maxBuffer: 16 * 1024 * 1024 });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  fs.writeFileSync(path.join(output, `${label}.stdout`), stdout);
  fs.writeFileSync(path.join(output, `${label}.stderr`), stderr);
  save(`${label}.packet.json`, {
    binary,
    args,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: { bytes: stdout.length, sha256: hash(stdout) },
    stderr: { bytes: stderr.length, sha256: hash(stderr) },
  });
  assert.equal(result.error, undefined, label);
  assert.equal(result.signal, null, label);
  if (cleanStderr) assert.equal(stderr.length, 0, `${label}: full stderr must be empty`);
  assert.ok(result.status === 0 || result.status === 1, `${label}: process status`);
  if (label === "install") assert.equal(result.status, 0);
  statuses.set(label, result.status);
  return stdout;
}

const expected = expectedBuildIdentity(repo);
assert.equal(expected.sourceRevision, process.env.EXPECTED_SOURCE_SHA);
const sourceReceipt = JSON.parse(
  fs.readFileSync(path.join(repo, "target/ci/vize.differential-build.json"), "utf8"),
) as unknown;
validateBuildReceipt(sourceReceipt, expected);
save("source-build.json", sourceReceipt);
const sourceBinary = path.join(repo, expected.binaryPath);
const consumer = fs.realpathSync(
  fs.mkdtempSync(path.join(os.tmpdir(), "vize-cross-file-effects-")),
);
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify({ private: true, dependencies: { vize: "0.565.0", "@vizejs/wasm": "0.565.0" } }),
);
run("install", "npm", ["install", "--ignore-scripts", "--no-audit", "--no-fund"], consumer, false);
fs.copyFileSync(
  path.join(consumer, "package-lock.json"),
  path.join(output, "public-package-lock.json"),
);

const lock = record(JSON.parse(fs.readFileSync(path.join(consumer, "package-lock.json"), "utf8")));
const packages = record(lock.packages);
const packageIdentities: unknown[] = [];
for (const [key, raw] of Object.entries(packages)) {
  if (!(key === "node_modules/vize" || key.startsWith("node_modules/@vizejs/"))) continue;
  const entry = record(raw);
  const name = key.slice("node_modules/".length);
  assert.equal(entry.version, "0.565.0", name);
  const installedPath = path.join(consumer, key, "package.json");
  if (!fs.existsSync(installedPath)) {
    assert.equal(
      entry.optional,
      true,
      `only platform-filtered optional packages may be absent: ${name}`,
    );
    continue;
  }
  const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(name)}/0.565.0`, {
    signal: AbortSignal.timeout(30_000),
  });
  assert.equal(response.status, 200, name);
  const manifest = record(await response.json());
  const dist = record(manifest.dist);
  assert.equal(entry.resolved, dist.tarball, name);
  assert.equal(entry.integrity, dist.integrity, name);
  const installed = record(JSON.parse(fs.readFileSync(installedPath, "utf8")));
  assert.equal(installed.name, name);
  assert.equal(installed.version, "0.565.0");
  packageIdentities.push({
    name,
    manifest,
    lock: entry,
    installedPath,
    installedSha256: hash(fs.readFileSync(installedPath)),
  });
}
assert.ok(packageIdentities.length >= 4);
save("public-packages.json", packageIdentities);
const publicBinary = path.join(consumer, "node_modules/vize/bin/vize");
save("public-entrypoint.json", { path: publicBinary, sha256: hash(fs.readFileSync(publicBinary)) });
assert.equal(
  run("public-version", publicBinary, ["--version"], consumer).toString("utf8"),
  "vize 0.565.0\n",
);
assert.equal(
  run("source-version", sourceBinary, ["--version"], repo).toString("utf8"),
  expected.cliVersion + "\n",
);
const publicRequire = createRequire(path.join(consumer, "package.json"));
const native: unknown = publicRequire("@vizejs/native");
assert.equal(typeof record(native).runCli, "function");
const nativeImages = Object.keys(publicRequire.cache)
  .filter((image) => image.endsWith(".node") && image.startsWith(consumer + path.sep))
  .map((image) => ({ path: image, sha256: hash(fs.readFileSync(image)) }));
assert.equal(nativeImages.length, 1);
save("public-native-images.json", nativeImages);

interface WasmApi {
  default: (input: { module_or_path: Uint8Array }) => Promise<unknown>;
  analyzeCrossFile: (files: { path: string; source: string }[], options: object) => unknown;
}
const wasmPath = path.join(consumer, "node_modules/@vizejs/wasm/vize_vitrine.js");
const loaded: unknown = await import(pathToFileURL(wasmPath).href);
assert.equal(typeof record(loaded).default, "function");
assert.equal(typeof record(loaded).analyzeCrossFile, "function");
const wasm = loaded as WasmApi;
const wasmBytes = fs.readFileSync(
  path.join(consumer, "node_modules/@vizejs/wasm/vize_vitrine_bg.wasm"),
);
save("public-wasm-image.json", {
  loaderSha256: hash(fs.readFileSync(wasmPath)),
  wasmSha256: hash(wasmBytes),
});
await wasm.default({ module_or_path: wasmBytes });
const project = path.join(consumer, 'project-"日本語🙂');
fs.mkdirSync(project);
const observations: unknown[] = [];

function profileCheck(label: string, fileCount: number, blocks: number, off = false): unknown {
  const profile: unknown = JSON.parse(
    fs.readFileSync(path.join(output, label + ".profile.json"), "utf8"),
  );
  const data = record(profile);
  assert.deepEqual(data.truncation, { dropped_spans: 0, dropped_counters: 0 });
  assert.ok(Array.isArray(data.spans) && Array.isArray(data.counters));
  const selectedSpans = data.spans
    .map(record)
    .filter((row) => String(row.key).startsWith("cli.lint.cross_file.sfc."));
  const selectedCounters = data.counters
    .map(record)
    .filter((row) => row.key === "cli.lint.cross_file.sfc.effect_script_parse_inputs");
  if (off) {
    assert.deepEqual(selectedSpans, []);
    assert.deepEqual(selectedCounters, []);
  } else {
    assert.equal(selectedSpans.length, 2);
    for (const span of selectedSpans) assert.equal(span.count, fileCount);
    assert.equal(selectedCounters.length, 1);
    assert.equal(selectedCounters[0].samples, fileCount);
    assert.equal(selectedCounters[0].total, blocks);
  }
  return { spans: selectedSpans, counters: selectedCounters, allocation: data.allocation };
}

for (const test of cases) {
  for (const name of fs.readdirSync(project)) fs.rmSync(path.join(project, name));
  const inputs = Object.entries(test.files).map(([name, source]) => {
    fs.writeFileSync(path.join(project, name), source);
    return { path: name, source, sha256: hash(source) };
  });
  save(`${test.id}.inputs.json`, inputs);
  const oracle = wasm.analyzeCrossFile(inputs, {});
  save(`${test.id}.wasm-complete.json`, oracle);
  checkWasm(test, oracle);
  const flags = ["lint", "--no-config", "--cross-file-complexity", "--help-level", "short"];
  const beforeJson = run(
    `${test.id}.public-json`,
    publicBinary,
    [...flags, "--format", "json", "*.vue"],
    project,
  );
  const afterJson = run(
    `${test.id}.source-json`,
    sourceBinary,
    [...flags, "--format", "json", "*.vue"],
    project,
  );
  assert.deepEqual(afterJson, beforeJson, `${test.id}: complete diagnostic packet`);
  assert.equal(statuses.get(`${test.id}.source-json`), statuses.get(`${test.id}.public-json`));
  if (test.diagnostics) {
    const diagnostics: unknown = JSON.parse(beforeJson.toString("utf8"));
    assert.ok(Array.isArray(diagnostics));
    assert.ok(
      diagnostics.some((file: unknown) => {
        const messages = record(file).messages;
        return Array.isArray(messages) && messages.length > 0;
      }),
      "the complete preservation control must have actual diagnostics",
    );
  }
  const beforePlain = run(
    `${test.id}.public-plain`,
    publicBinary,
    [...flags, "--format", "plain", "*.vue"],
    project,
  );
  const profileName = `${test.id}.source-plain`;
  const afterPlain = run(
    profileName,
    sourceBinary,
    [
      ...flags,
      "--format",
      "plain",
      "--profile-json",
      path.join(output, profileName + ".profile.json"),
      "*.vue",
    ],
    project,
  );
  const expectedPlain = expectedReport(test, beforePlain.toString("utf8"));
  fs.writeFileSync(path.join(output, `${test.id}.expected-plain`), expectedPlain);
  assert.equal(afterPlain.toString("utf8"), expectedPlain, `${test.id}: complete report packet`);
  assert.equal(statuses.get(profileName), statuses.get(`${test.id}.public-plain`));
  if (test.graph.edges)
    assert.notDeepEqual(
      beforePlain,
      afterPlain,
      "old public product must reproduce the regression",
    );
  observations.push({
    id: test.id,
    inputs,
    expected: test.graph,
    wasmComparable: test.wasmComparable,
    diagnosticsSha256: hash(afterJson),
    reportSha256: hash(afterPlain),
    profile: profileCheck(profileName, inputs.length, test.blocks),
  });
  save("observations.json", observations);
}
const noCrossFlags = ["lint", "--no-config", "--format", "json", "*.vue"];
const beforeOff = run("opt-out.public", publicBinary, noCrossFlags, project);
const afterOff = run(
  "opt-out.source",
  sourceBinary,
  [...noCrossFlags, "--profile-json", path.join(output, "opt-out.source.profile.json")],
  project,
);
assert.deepEqual(afterOff, beforeOff);
assert.equal(statuses.get("opt-out.source"), statuses.get("opt-out.public"));
profileCheck("opt-out.source", 0, 0, true);
assert.deepEqual(cases[0].files, cases.at(-1)?.files, "repair must restore all exact input bytes");
save("qualification.json", {
  passed: true,
  source: expected,
  publicVersion: "0.565.0",
  scenarios: observations.length,
  observations,
  optOutDiagnosticSha256: hash(afterOff),
  note: "Source-phase measurements observe necessary corrected work in an unoptimized CI binary; no throughput speedup or allocation claim.",
});
process.stdout.write(
  `cross-file SFC effects: ${observations.length} complete two-file scenarios and opt-out passed\n`,
);
