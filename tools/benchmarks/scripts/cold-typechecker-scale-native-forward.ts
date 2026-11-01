#!/usr/bin/env node
/** Untimed observer only: forward one exact native invocation without changing its bytes. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { seal, validateNativeArgs } from "./cold-typechecker-scale-native-inputs.ts";
import { recordReplayBeforeInspection } from "./cold-typechecker-scale-native-custody.ts";

const image = process.env.VIZE_COLD_NATIVE,
  output = process.env.VIZE_COLD_CAPTURE,
  project = process.env.VIZE_COLD_PROJECT,
  provider = process.env.VIZE_COLD_PROVIDER;
assert(image && output && project && provider);
assert([image, output, project, provider].every((value) => path.isAbsolute(value)));
const cwd = fs.realpathSync(process.cwd()),
  argv = process.argv.slice(2),
  executable = fs.realpathSync(image),
  directory = fs.mkdtempSync(path.join(output, "call-"));
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const write = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n", {
    flag: "wx",
  });
const startedNs = process.hrtime.bigint(),
  imageBefore = hash(fs.readFileSync(executable));
const canonicalForwarder = fs.realpathSync(process.argv[1]);
write("invocation.json", {
  startedNs: startedNs.toString(),
  executable,
  imageBefore,
  argv,
  cwd,
  canonicalForwarder,
  observerPid: process.pid,
  nodeVersion: process.version,
  nodeExecutable: fs.realpathSync(process.execPath),
  nodeSha256: hash(fs.readFileSync(process.execPath)),
});
assert.match(process.version, /^v24\./u);
assert.equal(canonicalForwarder, path.resolve(process.argv[1]));
assert.equal(fs.lstatSync(canonicalForwarder).isSymbolicLink(), false);
assert.equal(path.basename(path.dirname(canonicalForwarder)), ".bin");
assert.notEqual(path.basename(path.dirname(path.dirname(canonicalForwarder))), "node_modules");
const projectAt = argv.indexOf("--project");
const config = projectAt >= 0 ? path.resolve(cwd, argv[projectAt + 1]) : null;
const probe =
  config !== null &&
  argv.length === 4 &&
  argv[0] === "--pretty" &&
  argv[1] === "false" &&
  argv[2] === "--project";
const args = config && !probe ? validateNativeArgs(argv, config) : argv;
if (!config) assert.deepEqual(argv, ["--version"]);
if (probe) assert.deepEqual(argv, ["--pretty", "false", "--project", config]);
const buildInfo = config && !probe ? path.resolve(args[4]) : null;
if (buildInfo) {
  assert(buildInfo.startsWith(cwd + path.sep));
  assert.equal(
    fs.existsSync(buildInfo),
    false,
    "fresh untimed native compilation requires absent build info",
  );
}
const nativeEnv = { ...process.env };
for (const key of [
  "VIZE_COLD_NATIVE",
  "VIZE_COLD_CAPTURE",
  "VIZE_COLD_PROJECT",
  "VIZE_COLD_PROVIDER",
])
  delete nativeEnv[key];
write("request.json", {
  startedNs: startedNs.toString(),
  executable,
  imageBefore,
  argv,
  cwd,
  kind: config ? (probe ? "option-probe" : "main-cli") : "version",
  canonicalForwarder,
  apiClassification: "async-json-rpc (canonical physical parent .bin; not node_modules/.bin)",
  config,
  buildInfo,
  buildInfoInitiallyAbsent: buildInfo ? true : null,
  environment: Object.fromEntries(
    [
      "GOMAXPROCS",
      "GOGC",
      "GOMEMLIMIT",
      "RAYON_NUM_THREADS",
      "NO_COLOR",
      "PATH",
      "XDG_CACHE_HOME",
      "TMPDIR",
    ].map((key) => [key, nativeEnv[key] ?? null]),
  ),
  scope: "untimed forwarding observer; potential inputs sealed, native loaded files unobserved",
});
if (config) fs.copyFileSync(config, path.join(directory, "config.bytes"));
const roots = config ? [project, provider, cwd] : [project, provider],
  before = seal(roots, buildInfo);
write("inputs-before.json", before);
const nativeStartedNs = process.hrtime.bigint();
const result = spawnSync(executable, argv, {
  cwd,
  env: nativeEnv,
  timeout: 85000,
  maxBuffer: 256 * 1024 * 1024,
});
recordReplayBeforeInspection(
  directory,
  result,
  (process.hrtime.bigint() - nativeStartedNs).toString(),
  () => {
    const after = seal(roots, buildInfo);
    write("inputs-after.json", after);
    assert.deepEqual(
      after,
      before,
      "native process changed an input outside the owned build-info output",
    );
    const imageAfter = hash(fs.readFileSync(executable));
    write("image-after.json", { sha256: imageAfter });
    assert.equal(imageAfter, imageBefore);
    if (buildInfo && fs.existsSync(buildInfo)) {
      const bytes = fs.readFileSync(buildInfo);
      fs.copyFileSync(buildInfo, path.join(directory, "native.tsbuildinfo"));
      write("build-info.json", { bytes: bytes.length, sha256: hash(bytes) });
    }
  },
);
if (result.stdout?.length)
  await new Promise<void>((resolve, reject) =>
    process.stdout.write(result.stdout, (error) => (error ? reject(error) : resolve())),
  );
if (result.stderr?.length)
  await new Promise<void>((resolve, reject) =>
    process.stderr.write(result.stderr, (error) => (error ? reject(error) : resolve())),
  );
if (result.signal) process.kill(process.pid, result.signal);
else process.exitCode = result.status ?? 1;
