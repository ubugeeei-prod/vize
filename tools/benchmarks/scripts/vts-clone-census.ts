/** One profiling CLI image, then tiny public graph admission and source census. */
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
  censusFixture,
  packetDiagnostics,
  assertPacket,
  type Diagnostic,
} from "./vts-clone-census-fixture.ts";
import {
  censusProfile,
  packCensusProfiles,
  type CensusProfileReceipt,
  type Profile,
} from "./vts-clone-census-profile.ts";

const { values } = parseArgs({
  options: {
    out: { type: "string" },
    target: { type: "string" },
    provider: { type: "string" },
    runtime: { type: "string" },
  },
});
assert.ok(values.out && values.target && values.provider && values.runtime);
const root = fs.realpathSync(process.cwd()),
  out = path.resolve(values.out),
  target = path.resolve(values.target),
  provider = path.resolve(values.provider),
  runtime = path.resolve(values.runtime);
for (const dir of [out, target, runtime]) {
  assert.ok(!fs.existsSync(dir), "task-owned output must be absent");
  const relative = path.relative(root, dir);
  assert.ok(relative === ".." || relative.startsWith(".." + path.sep) || path.isAbsolute(relative));
}
fs.mkdirSync(out, { recursive: true });
fs.mkdirSync(runtime, { recursive: true });
const hash = (b: string | Buffer) => createHash("sha256").update(b).digest("hex");
const fileHash = (p: string) => hash(fs.readFileSync(p));
const checked = (exe: string, args: string[]) => {
  const r = spawnSync(exe, args, {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.signal, null);
  assert.equal(r.error, undefined);
  return r.stdout.trim();
};
const git = (...args: string[]) => checked("git", args);
const sourceSha = git("rev-parse", "HEAD"),
  sourceTree = git("rev-parse", "HEAD^{tree}");
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
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
  role: "vts-clone-census-profiling",
  allocation: true,
});
assert.equal(build.sourceSha, sourceSha);
assert.equal(build.allocation, true);
assert.ok(build.cargoArtifact.features.includes("profiling"));
const binarySha = fileHash(binary);
const write = (p: string, text: string) => {
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, text);
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
    args = [
      "check",
      "--no-config",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      backend,
      ...(profile ? ["--profile-json", path.join(evidence, "profile.json")] : []),
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
  ])
    delete env[key];
  const start = process.hrtime.bigint();
  const r = spawnSync(binary, args, {
    cwd: project,
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 64 * 1024 * 1024,
    killSignal: "SIGKILL",
  });
  const elapsedNs = Number(process.hrtime.bigint() - start);
  write(path.join(evidence, "stdout.json"), r.stdout ?? "");
  write(path.join(evidence, "stderr.txt"), r.stderr ?? "");
  const command = {
    executable: binary,
    args,
    cwd: project,
    status: r.status,
    signal: r.signal,
    error: r.error?.message ?? null,
    binarySha256: binarySha,
    backendSha256: backendSha,
    runtimeOwner: owner,
    configuredRayonThreads: process.env.RAYON_NUM_THREADS ?? null,
    configuredGoMaxProcs: process.env.GOMAXPROCS ?? null,
    explicitServers: null,
    explicitCheckers: null,
    actualBackendProcessCount: null,
    sourceConfigIncremental: false,
    actualBackendIncremental: null,
    coldQualified: false,
  };
  write(path.join(evidence, "process.json"), JSON.stringify(command, null, 2));
  assert.equal(r.error, undefined);
  assert.equal(r.signal, null);
  assert.equal(r.status, expected.length ? 1 : 0, r.stderr);
  const packet = JSON.parse(r.stdout),
    actual = packetDiagnostics(packet, project);
  assertPacket(actual, expected);
  assert.deepEqual(sealFiles(project, files), before);
  assert.equal(fileHash(binary), binarySha);
  assert.equal(fileHash(backend), backendSha);
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
    const raw = JSON.parse(fs.readFileSync(path.join(evidence, "profile.json"), "utf8")) as Profile;
    assert.equal(validateProfile(raw), true, JSON.stringify(validateProfile.errors));
    const selected = censusProfile(raw, sfcs);
    profiles.push({
      stage,
      rawProfileSha256: fileHash(path.join(evidence, "profile.json")),
      value: selected,
    });
  }
  fs.rmSync(owner, { recursive: true });
  assert.equal(fs.existsSync(owner), false);
  return elapsedNs;
}
for (const sfcs of [1, 10, 100]) {
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
  for (let pair = 0; pair < (sfcs === 1 ? 3 : 1); pair++)
    for (const enabled of pair % 2 === 0 ? [false, true] : [true, false]) {
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
    samples,
    scope:
      "same profiling-enabled ELF; collection/export on versus off, includes real backend and process startup; not isolated counter overhead, no normal shipping performance qualification",
  });
}
assert.equal(git("rev-parse", "HEAD"), sourceSha);
assert.equal(git("status", "--porcelain", "--untracked-files=all"), "");
const packedProfiles = packCensusProfiles(profiles);
const summary = {
  schema: "vize.generated-vts.source-census",
  version: 1,
  sourceSha,
  sourceTree,
  productionBase: "09ff11b5a3440381d4f4558652885ec19520e4f1",
  binarySha256: binarySha,
  cliVersion: checked(binary, ["--version"]),
  provider: providerSeal,
  host: {
    model: os.cpus()[0]?.model ?? null,
    parallelism: os.availableParallelism(),
    affinity: checked("taskset", ["-pc", String(process.pid)]),
  },
  buildReceiptSha256: fileHash(path.join(out, "build.json")),
  ...packedProfiles,
  overhead,
  admission: packets.map((p) => ({
    stage: p.stage,
    sfcs: p.sfcs,
    profile: p.profile,
    actualCount: p.actual.length,
    expectedCount: p.expected.length,
    fileCount: p.fileCount,
    packetSha256: fileHash(path.join(out, "packets", p.stage, "stdout.json")),
    diagnosticVectorSha256: hash(JSON.stringify(p.actual)),
  })),
  totalPackets: packets.length,
  allFullTuplesPassed: true,
  sourceCleanAfter: true,
  normalPerformanceQualified: false,
  coldQualified: false,
  allFeatureAccuracyQualified: false,
  actualHashOperations: null,
  physicalCopyBytes: null,
  processTreeRss: null,
  peakLiveBytes: null,
  sourceAccounting:
    "logical source lengths are not physical memcpy bytes; nested source spans change parent self allocation attribution; absent branch sites remain unobserved/null",
};
const summaryText = JSON.stringify(summary) + "\n";
assert.ok(Buffer.byteLength(summaryText) <= 90_000, "bounded summary must stay below 90KB");
write(path.join(out, "summary.json"), summaryText);
write(path.join(out, "admission.json"), JSON.stringify(packets, null, 2) + "\n");
console.log(
  JSON.stringify({
    sourceSha,
    sourceTree,
    binarySha256: binarySha,
    packets: packets.length,
    allFullTuplesPassed: true,
    profiles: profiles.length,
  }),
);
