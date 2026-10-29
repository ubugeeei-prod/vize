/** One fresh profiling CLI, exact graph admission and three paired source-census repetitions. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createRequire } from "node:module";
import { parseArgs } from "node:util";
import { buildColdScaleBinary } from "./cold-typechecker-scale-build.ts";
import {
  assertPacket,
  censusFixture,
  packetDiagnostics,
  type Diagnostic,
} from "./vts-source-census-fixture.ts";
import {
  VTS_SOURCE_CENSUS_REPETITIONS,
  VTS_SOURCE_CENSUS_SIZES,
  censusProfile,
  packCensusProfiles,
  type CensusProfileReceipt,
  type IdentityReceipt,
  type Profile,
} from "./vts-source-census-profile.ts";

const { values } = parseArgs({
  options: {
    out: { type: "string" },
    target: { type: "string" },
    provider: { type: "string" },
    runtime: { type: "string" },
    "source-sha": { type: "string" },
    "source-parent": { type: "string" },
    "driver-sha": { type: "string" },
  },
});
assert.ok(
  values.out &&
    values.target &&
    values.provider &&
    values.runtime &&
    values["source-sha"] &&
    values["source-parent"] &&
    values["driver-sha"],
);
const root = fs.realpathSync(process.cwd()),
  out = path.resolve(values.out),
  target = path.resolve(values.target),
  provider = path.resolve(values.provider),
  runtime = path.resolve(values.runtime),
  expectedSourceSha = values["source-sha"],
  sourceParent = values["source-parent"],
  driverSha = values["driver-sha"];
assert.match(driverSha, /^[a-f0-9]{40}$/u);
for (const dir of [out, target, runtime]) {
  assert.ok(!fs.existsSync(dir), "task-owned output must be absent");
  const relative = path.relative(root, dir);
  assert.ok(
    relative === ".." || relative.startsWith(".." + path.sep),
    "outputs stay outside source",
  );
}
fs.mkdirSync(out, { recursive: true });
fs.mkdirSync(runtime, { recursive: true });
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));
const checked = (executable: string, args: string[]) => {
  const result = spawnSync(executable, args, {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.signal, null);
  assert.equal(result.error, undefined);
  return result.stdout.trim();
};
const git = (...args: string[]) => checked("git", args);
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
assert.equal(sourceSha, expectedSourceSha);
assert.equal(git("rev-parse", "HEAD^"), sourceParent);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
const workflowPath = ".github/workflows/vts-source-census.yml",
  blobOf = (file: string) => git("rev-parse", "HEAD:" + file),
  driverWorkflowBlob = git("rev-parse", driverSha + ":" + workflowPath),
  sourceWorkflowBlob = blobOf(workflowPath);
const sourceBlobs = {
  workflow: sourceWorkflowBlob,
  harness: blobOf("tools/benchmarks/scripts/vts-source-census.ts"),
  fixture: blobOf("tools/benchmarks/scripts/vts-source-census-fixture.ts"),
  preflight: blobOf("tools/benchmarks/scripts/vts-source-census-preflight.ts"),
  profile: blobOf("tools/benchmarks/scripts/vts-source-census-profile.ts"),
  test: blobOf("tests/tooling/vts-source-census.test.ts"),
};
const provenanceBlobs = {
  ...sourceBlobs,
  workflow: driverWorkflowBlob,
  sourceWorkflow: sourceWorkflowBlob,
};

const require = createRequire(path.join(provider, "package.json"));
const nativeRequire = createRequire(require.resolve("typescript-native/package.json"));
const backend = path.join(
  path.dirname(nativeRequire.resolve("@typescript/typescript-linux-x64/package.json")),
  "lib",
  "tsc",
);
const backendSha = fileHash(backend);
type Validator = ((value: unknown) => boolean) & { errors: unknown };
const Ajv = require("ajv/dist/2020").default as new (options: unknown) => {
  compile(schema: unknown): Validator;
};
const profileSchema = JSON.parse(
  fs.readFileSync("docs/davinci/plan/profile-export.schema.json", "utf8"),
);
const validateProfile = new Ajv({ strict: false, allErrors: true }).compile(profileSchema);
const providerSeal = {
  manifest: fileHash(path.join(provider, "package.json")),
  lock: fileHash(path.join(provider, "package-lock.json")),
  vue: require("vue/package.json").version as string,
  nativeTypeScript: nativeRequire("./package.json").version as string,
  backend,
  backendSha256: backendSha,
  backendVersion: checked(backend, ["--version"]),
};
const binary = path.join(out, "binary", "vize");
const build = buildColdScaleBinary({
  sourceRoot: root,
  targetDir: target,
  binary,
  receipt: path.join(out, "build.json"),
  role: "vts-source-census-profiling",
  allocation: true,
});
assert.equal(build.sourceSha, sourceSha);
assert.equal(build.allocation, true);
assert.ok(build.cargoArtifact.features.includes("profiling"));
const binarySha = fileHash(binary);

const write = (file: string, text: string) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text);
};
const sealFiles = (project: string, files: Record<string, string>) =>
  Object.fromEntries(
    Object.keys(files)
      .toSorted()
      .map((name) => [
        name,
        {
          bytes: fs.statSync(path.join(project, name)).size,
          sha256: fileHash(path.join(project, name)),
        },
      ]),
  );

interface PacketRow {
  stage: string;
  sfcs: number;
  profile: boolean;
  elapsedNs: number;
  command: unknown;
  actual: Diagnostic[];
  expected: Diagnostic[];
  fileCount: number;
  sourceSeal: unknown;
}

const packets: PacketRow[] = [],
  profiles: CensusProfileReceipt[] = [],
  overhead: unknown[] = [];

function invoke(
  project: string,
  sfcs: number,
  stage: string,
  expected: Diagnostic[],
  files: Record<string, string>,
  profile: boolean,
) {
  const evidence = path.join(out, "packets", stage);
  fs.mkdirSync(evidence, { recursive: true });
  const owner = path.join(runtime, randomUUID());
  fs.mkdirSync(path.join(owner, "tmp"), { recursive: true });
  const before = sealFiles(project, files),
    profilePath = path.join(evidence, "profile.json"),
    identityPath = path.join(evidence, "source-identity.json"),
    args = [
      "check",
      "--no-config",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      backend,
      ...(profile ? ["--profile-json", profilePath] : []),
    ];
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    NO_COLOR: "1",
    XDG_CACHE_HOME: path.join(owner, "cache"),
    TMPDIR: path.join(owner, "tmp"),
  };
  for (const key of [
    "VIZE_CACHE_CUSTODY_CAPTURE_DIR",
    "VIZE_NATIVE_DIAGNOSTIC_AUDIT_DIR",
    "VIZE_TYPECHECK_BACKEND_OBSERVER_DIR",
    "VIZE_VTS_SOURCE_CENSUS_PATH",
  ])
    delete env[key];
  if (profile) env.VIZE_VTS_SOURCE_CENSUS_PATH = identityPath;
  const start = process.hrtime.bigint();
  const result = spawnSync(binary, args, {
    cwd: project,
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 64 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
  const elapsedNs = Number(process.hrtime.bigint() - start);
  write(path.join(evidence, "stdout.json"), result.stdout ?? "");
  write(path.join(evidence, "stderr.txt"), result.stderr ?? "");
  const command = {
    executable: binary,
    args,
    cwd: project,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    binarySha256: binarySha,
    backendSha256: backendSha,
    runtimeOwner: owner,
    configuredRayonThreads: process.env.RAYON_NUM_THREADS ?? null,
    configuredGoMaxProcs: process.env.GOMAXPROCS ?? null,
    sourceSha,
    sourceTree,
    coldQualified: false,
  };
  write(path.join(evidence, "process.json"), JSON.stringify(command, null, 2));
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, expected.length ? 1 : 0, result.stderr);
  const packet = JSON.parse(result.stdout),
    actual = packetDiagnostics(packet, project);
  assertPacket(actual, expected);
  assert.deepEqual(sealFiles(project, files), before);
  assert.equal(fileHash(binary), binarySha);
  assert.equal(fileHash(backend), backendSha);
  assert.equal(fs.existsSync(profilePath), profile);
  assert.equal(fs.existsSync(identityPath), profile);
  packets.push({
    stage,
    sfcs,
    profile,
    elapsedNs,
    command,
    actual,
    expected,
    fileCount: packet.fileCount,
    sourceSeal: before,
  });
  if (profile) {
    const rawProfile = JSON.parse(fs.readFileSync(profilePath, "utf8")) as Profile,
      rawIdentity = JSON.parse(fs.readFileSync(identityPath, "utf8")) as IdentityReceipt;
    assert.equal(validateProfile(rawProfile), true, JSON.stringify(validateProfile.errors));
    const selected = censusProfile(rawProfile, rawIdentity, fileHash(identityPath), sfcs);
    assert.ok(selected.sameContentParseIdentity);
    profiles.push({
      stage,
      rawProfileSha256: fileHash(profilePath),
      rawIdentitySha256: fileHash(identityPath),
      value: selected,
    });
  }
  fs.rmSync(owner, { recursive: true });
  assert.equal(fs.existsSync(owner), false);
  return elapsedNs;
}

const summarize = (values: number[]) => {
  assert.equal(values.length, VTS_SOURCE_CENSUS_REPETITIONS);
  const ordered = values.toSorted((left, right) => left - right);
  return { samples: values, min: ordered[0], median: ordered[1], max: ordered[2] };
};

for (const sfcs of VTS_SOURCE_CENSUS_SIZES) {
  const project = path.join(out, "projects", "sfcs-" + String(sfcs).padStart(3, "0"));
  fs.mkdirSync(project, { recursive: true });
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
  const valid = censusFixture(sfcs, false),
    invalid = censusFixture(sfcs, true);
  const materialize = (files: Record<string, string>) => {
    for (const [name, text] of Object.entries(files)) write(path.join(project, name), text);
  };
  materialize(valid.files);
  invoke(project, sfcs, sfcs + "-valid", [], valid.files, false);
  materialize(invalid.files);
  invoke(project, sfcs, sfcs + "-invalid", invalid.expected, invalid.files, false);
  materialize(valid.files);
  invoke(project, sfcs, sfcs + "-repaired", [], valid.files, false);
  const samples: { enabled: boolean; elapsedNs: number }[] = [];
  const orders = Array.from({ length: VTS_SOURCE_CENSUS_REPETITIONS }, (_, repetition) =>
    repetition % 2 === 0 ? [false, true] : [true, false],
  );
  for (const [pair, order] of orders.entries())
    for (const enabled of order) {
      const elapsedNs = invoke(
        project,
        sfcs,
        sfcs + "-collection-" + pair + "-" + enabled,
        [],
        valid.files,
        enabled,
      );
      samples.push({ enabled, elapsedNs });
    }
  overhead.push({
    sfcs,
    disabled: summarize(
      samples.filter((sample) => !sample.enabled).map((sample) => sample.elapsedNs),
    ),
    enabled: summarize(
      samples.filter((sample) => sample.enabled).map((sample) => sample.elapsedNs),
    ),
    order: samples.map((sample) => sample.enabled),
    scope:
      "same profiling-enabled ELF; collection/export on versus off, includes backend and process startup; not normal shipping performance qualification",
  });
}

assert.equal(git("rev-parse", "HEAD"), sourceSha);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
const packedProfiles = packCensusProfiles(profiles);
const summary = {
  schema: "vize.generated-vts.source-census",
  version: 2,
  sourceSha,
  sourceTree,
  productionBase: sourceParent,
  binarySha256: binarySha,
  cliVersion: checked(binary, ["--version"]),
  provider: providerSeal,
  host: {
    model: os.cpus()[0]?.model ?? null,
    parallelism: os.availableParallelism(),
    affinity: checked("taskset", ["-pc", String(process.pid)]),
  },
  provenance: {
    commit: sourceSha,
    parent: sourceParent,
    tree: sourceTree,
    driver: { commit: driverSha, workflowBlob: driverWorkflowBlob },
    source: {
      commit: sourceSha,
      parent: sourceParent,
      tree: sourceTree,
      workflowBlob: sourceWorkflowBlob,
      blobs: sourceBlobs,
    },
    blobs: provenanceBlobs,
  },
  buildReceiptSha256: fileHash(path.join(out, "build.json")),
  ...packedProfiles,
  overhead,
  admission: packets.map((packet) => ({
    stage: packet.stage,
    sfcs: packet.sfcs,
    profile: packet.profile,
    actualCount: packet.actual.length,
    expectedCount: packet.expected.length,
    fileCount: packet.fileCount,
    packetSha256: fileHash(path.join(out, "packets", packet.stage, "stdout.json")),
    diagnosticVectorSha256: hash(JSON.stringify(packet.actual)),
  })),
  totalPackets: packets.length,
  profilingRepetitionsPerSize: VTS_SOURCE_CENSUS_REPETITIONS,
  allFullDiagnosticTuplesPassed: true,
  allIdentityReceiptsComplete: profiles.every(
    (profile) =>
      profile.value.sameContentParseIdentity.consumerComplete &&
      profile.value.sameContentParseIdentity.producerComplete,
  ),
  allSameContentParseIdentity: profiles.every(
    (profile) => profile.value.sameContentParseIdentity.matches,
  ),
  optimizationEligible: profiles.every((profile) => profile.value.sameContentParseIdentity.matches),
  identityMismatchPolicy:
    "a complete false identity result is valid measurement evidence and blocks reuse optimization without failing this census",
  sourceCleanAfter: true,
  normalPerformanceQualified: false,
  coldQualified: false,
  processAllocationIncludesObserverStorage: true,
};
const summaryText = JSON.stringify(summary) + "\n";
assert.ok(Buffer.byteLength(summaryText) <= 256_000, "bounded summary must stay below 256KB");
write(path.join(out, "summary.json"), summaryText);
write(path.join(out, "admission.json"), JSON.stringify(packets, null, 2) + "\n");
console.log(
  JSON.stringify({
    sourceSha,
    sourceTree,
    driverSha,
    packets: packets.length,
    profiles: profiles.length,
    allFullDiagnosticTuplesPassed: true,
    allIdentityReceiptsComplete: summary.allIdentityReceiptsComplete,
    allSameContentParseIdentity: summary.allSameContentParseIdentity,
  }),
);
