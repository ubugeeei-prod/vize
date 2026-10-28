/** Bounded public negative/repair controls precede the retained normal CPU replay. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { MINIMAL_PLANTS } from "./check-gate-plants.mjs";
import { captureSourceOperationPerf } from "./source-operation-perf.ts";
import { boundedCommand, digest, renderSourceCpu } from "./source-symbolizer-render.ts";
import { retainedNormalIdentity } from "./source-symbolizer-smoke.ts";

type Packet = {
  fileCount: number;
  errorCount: number;
  warningCount: number;
  files: { file: string; diagnostics: string[] }[];
};
const json = (path: string) => JSON.parse(readFileSync(path, "utf8"));
const sha = (body: string) => createHash("sha256").update(body).digest("hex");
const plant = MINIMAL_PLANTS.find((entry) => entry.id === "script")!;
export function verifyStagePacket(
  packet: Packet,
  project: string,
  names: string[],
  invalid: boolean,
) {
  assert.equal(packet.fileCount, names.length);
  assert.equal(packet.files.length, names.length, "array count independently matches fileCount");
  assert.equal(packet.errorCount, invalid ? names.length : 0);
  assert.equal(packet.warningCount, 0);
  const normalized = packet.files
    .map((entry) => ({
      file: relative(project, resolve(project, entry.file)).replaceAll("\\", "/"),
      diagnostics: entry.diagnostics,
    }))
    .sort((left, right) => left.file.localeCompare(right.file));
  assert.deepEqual(
    normalized.map((entry) => entry.file),
    [...names].sort(),
  );
  assert.equal(new Set(normalized.map((entry) => entry.file)).size, names.length);
  for (const entry of normalized)
    assert.deepEqual(entry.diagnostics, invalid ? [plant.expected.diagnostic] : []);
  return { ...packet, files: normalized };
}
export function runBoundedStages(options: {
  perf: string;
  out: string;
  root: string;
  provider: string;
  carrier: string;
}) {
  assert.ok(!existsSync(options.out), "fresh bounded evidence required");
  mkdirSync(options.out, { recursive: true });
  const carrier = json(options.carrier);
  assert.equal(carrier.qualified, true);
  const { original, build, binary } = retainedNormalIdentity(options.root);
  const dwarf = boundedCommand(
    join(options.out, "sections"),
    "readelf",
    ["--section-headers", "--wide", binary],
    10_000,
  );
  assert.equal(dwarf.status, 0);
  assert.ok(dwarf.stdout.includes(".debug_line"), "retained normal binary has no line tables");
  const providerRequire = createRequire(join(options.provider, "package.json"));
  const nativeRequire = createRequire(providerRequire.resolve("typescript-native/package.json"));
  const backend = join(
    dirname(
      nativeRequire.resolve("@typescript/typescript-linux-" + process.arch + "/package.json"),
    ),
    "lib/tsc",
  );
  assert.equal(digest(backend), original.identity.backendSha256, "exact retained backend required");
  const providers = Object.fromEntries(
    ["vue", "typescript", "typescript-native"].map((name) => {
      const path = providerRequire.resolve(name + "/package.json");
      return [name, { path, version: json(path).version, sha256: digest(path) }];
    }),
  );
  writeFileSync(
    join(options.out, "provider-lock.json"),
    readFileSync(join(options.provider, "package-lock.json")),
  );
  const stages = [1, 10, 100].map((count) => {
    const out = join(options.out, String(count)),
      project = join(out, "project");
    mkdirSync(project, { recursive: true });
    symlinkSync(join(options.provider, "node_modules"), join(project, "node_modules"), "dir");
    const names = Array.from({ length: count }, (_, index) => "Binding" + index + ".vue");
    const config = {
      compilerOptions: {
        strict: true,
        skipLibCheck: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "bundler",
        noEmit: true,
        types: [],
        lib: ["ESNext", "DOM"],
        incremental: false,
      },
      vueCompilerOptions: { strictTemplates: true },
      include: ["*.vue"],
    };
    writeFileSync(join(project, "tsconfig.json"), JSON.stringify(config, null, 2) + "\n");
    writeFileSync(join(project, "package.json"), '{"private":true,"type":"module"}\n');
    const source = plant.files["App.vue"]!;
    const sources = names.map((name, index) => ({
      file: name,
      invalid: source + "\n<!-- unique original source " + index + " -->\n",
    }));
    for (const entry of sources) writeFileSync(join(project, entry.file), entry.invalid);
    const args = [
      "check",
      "--no-config",
      "--format",
      "json",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      backend,
    ];
    const env = {
      NO_COLOR: "1",
      GOMAXPROCS: "8",
      RAYON_NUM_THREADS: "8",
      TMPDIR: join(out, "tmp"),
      XDG_CACHE_HOME: join(out, "xdg-cache"),
    };
    mkdirSync(env.TMPDIR);
    mkdirSync(env.XDG_CACHE_HOME);
    const negative = boundedCommand(join(out, "all-invalid"), binary, args, 30_000, undefined, {
      cwd: project,
      env,
    });
    assert.equal(negative.status, 1);
    verifyStagePacket(JSON.parse(negative.stdout), project, names, true);
    for (const entry of sources)
      writeFileSync(join(project, entry.file), entry.invalid.replace('"not-a-number"', "1"));
    const repairs = [0, 1, 2].map((ordinal) => {
      const run = boundedCommand(join(out, "repair-" + ordinal), binary, args, 30_000, undefined, {
        cwd: project,
        env,
      });
      assert.equal(run.status, 0);
      return { ...run, packet: verifyStagePacket(JSON.parse(run.stdout), project, names, false) };
    });
    for (const run of repairs) assert.deepEqual(run.packet, repairs[0].packet);
    const capture = captureSourceOperationPerf({
      mode: "record",
      perfExecutable: options.perf,
      addr2lineExecutable: carrier.selectedDecoder,
      outputDir: join(out, "capture"),
      timeoutMs: 30_000,
      reportTimeoutMs: 30_000,
      command: { executable: binary, args, cwd: project, env },
      identity: {
        source: build.sourceSha,
        binarySha256: original.binarySha256,
        backendSha256: original.identity.backendSha256,
        sfcCount: count,
        fixture: "public script assignment negative and repaired whole packets",
      },
    });
    assert.equal(capture.perf.status, 0);
    assert.equal(capture.collection, "collected");
    assert.deepEqual(
      verifyStagePacket(json(join(out, "capture/stdout.txt")), project, names, false),
      repairs[0].packet,
    );
    const decoded =
      capture.sampleCount !== null && capture.sampleCount > 0
        ? renderSourceCpu({
            binary,
            data: join(out, "capture/perf.data"),
            decoder: carrier.selectedDecoder,
            perf: options.perf,
            out: join(out, "decoded"),
            totalSamples: capture.sampleCount,
            reportTimeoutMs: 30_000,
          })
        : null;
    const result = {
      sfcCount: count,
      accuracyQualified: true,
      captureCompleted: true,
      sourceQualified: decoded?.qualified ?? false,
      sourceUnavailableReason: decoded?.qualified
        ? null
        : "bounded capture lacks qualified Rust source samples",
      sourceSha256: sha(JSON.stringify(sources)),
      tsconfigSha256: digest(join(project, "tsconfig.json")),
      originalFiles: sources.map((entry) => ({
        file: entry.file,
        bytes: Buffer.byteLength(entry.invalid),
        invalidSha256: sha(entry.invalid),
        repairedSha256: digest(join(project, entry.file)),
      })),
      actualArgv: [binary, ...args],
      environment: env,
      sampleCount: capture.sampleCount,
      normalFreshWallMs: repairs.map((run) => run.wallMs),
      captureWallMs: Date.parse(capture.captureFinishedAt) - Date.parse(capture.startedAt),
      cpuClock: decoded,
      coldQualified: false,
      hardwareInstructions: null,
      scope:
        "process-fresh diagnostics and software source samples; OS and persistent caches may be warm",
    };
    writeFileSync(join(out, "result.json"), JSON.stringify(result, null, 2) + "\n");
    return result;
  });
  assert.equal(digest(binary), original.binarySha256);
  assert.equal(digest(backend), original.identity.backendSha256);
  const result = {
    schemaVersion: 1,
    qualified: stages.every((stage) => stage.accuracyQualified && stage.captureCompleted),
    binarySha256: original.binarySha256,
    compiledSource: build.sourceSha,
    backendSha256: original.identity.backendSha256,
    providers,
    providerLockSha256: digest(join(options.provider, "package-lock.json")),
    stages,
    coldQualified: false,
    hardwareInstructions: null,
    scope:
      "bounded public controls; source qualification is per-stage and separate from packet accuracy",
  };
  writeFileSync(join(options.out, "result.json"), JSON.stringify(result, null, 2) + "\n");
  return result;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      perf: { type: "string" },
      out: { type: "string" },
      root: { type: "string" },
      provider: { type: "string" },
      carrier: { type: "string" },
    },
  });
  assert.ok(values.perf && values.out && values.root && values.provider && values.carrier);
  runBoundedStages({
    perf: values.perf,
    out: resolve(values.out),
    root: resolve(values.root),
    provider: resolve(values.provider),
    carrier: resolve(values.carrier),
  });
}
