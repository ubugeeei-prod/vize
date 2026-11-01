/** Exact-source normal A/B commands; full emitted virtual-code parity is untimed. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { run } from "./canon-instance-directives-io.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import {
  captureColdScale,
  prepareColdScale,
  verifyColdDiagnostics,
  coldScaleSupportScriptFiles,
} from "./cold-typechecker-scale.ts";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";
import { nearestRank } from "./cold-typechecker-scale-paired-stats.ts";
import { verifyColdScaleBuildPair } from "./cold-typechecker-scale-build.ts";
import {
  strictPairSources,
  verifyStrictSources,
  verifyStrictDebug,
  prepareStrictOutput,
} from "./cold-typechecker-scale-strict-pair.ts";
import {
  prepareNativeForwarder,
  collectNativePreflight,
  compareNativePreflights,
  assertSameObservedNative,
} from "./cold-typechecker-scale-native-preflight.ts";

import { captureReadBufferCommand } from "./cold-typechecker-scale-read-buffer-command.ts";
import { readBufferDistribution } from "./cold-typechecker-scale-read-buffer.ts";

export const readBufferOrders = [
  ["base", "head"],
  ["head", "base"],
  ["head", "base"],
  ["base", "head"],
] as const;

const readFileExists = (file: string) => {
  try {
    readFileSync(file);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
};
const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
type Build = {
  sourceSha: string;
  treeSha: string;
  binary: string;
  binarySha256: string;
  allocation: boolean;
  manifests: unknown;
  sourceRoot: string;
  cargoArtifact: { features: string[] };
};
type Sample = {
  role: string;
  order: number;
  wallMs: number;
  peakRssBytes: number;
  cpu: {
    userCpuMs: number;
    systemCpuMs: number;
    totalCpuMs: number;
    userSecondsRaw: string;
    systemSecondsRaw: string;
  };
  rootPeakBytes: number;
  concurrentObservedTreePeakBytes: number;
  directory: string;
};

function retainInputs(source: string, destination: string) {
  mkdirSync(destination, { recursive: true });
  for (const entry of readdirSync(source, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) retainInputs(join(source, entry.name), join(destination, entry.name));
    else copyFileSync(join(source, entry.name), join(destination, entry.name));
  }
}

export async function pairedColdComparison(options: {
  provider: string;
  out: string;
  baseReceipt: string;
  headReceipt: string;
}) {
  const { provider, out } = options;
  prepareStrictOutput(out);
  verifyColdScaleBuildPair(options.baseReceipt, options.headReceipt, process.cwd());
  const readBuild = (file: string) => JSON.parse(readFileSync(file, "utf8")) as Build;
  const base = readBuild(options.baseReceipt),
    head = readBuild(options.headReceipt);
  assert.equal(base.allocation, false);
  assert.equal(head.allocation, false);
  assert.equal(base.sourceSha, strictPairSources.base);
  assert.equal(head.sourceSha, strictPairSources.head);
  assert.deepEqual(head.cargoArtifact.features, base.cargoArtifact.features);
  for (const build of [base, head])
    assert(
      !build.cargoArtifact.features.some((name) => ["profiling", "cache-custody"].includes(name)),
    );
  for (const name of ["NATIVE", "CAPTURE", "PROJECT", "PROVIDER"])
    assert.equal(
      process.env["VIZE_COLD_" + name],
      undefined,
      "observer must not enter timed environment",
    );
  assert.deepEqual(
    base.manifests,
    head.manifests,
    "dependency and manifest changes are outside this comparison",
  );
  const providerLock = hash(join(provider, "package-lock.json"));
  assert.equal(providerLock, "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6");
  const sourceClosure = verifyStrictSources(base.sourceRoot, head.sourceRoot, provider);
  writeFileSync(join(out, "source-closure.json"), JSON.stringify(sourceClosure, null, 2) + "\n");
  const files = 2500 + coldScaleSupportScriptFiles;
  const parityDirectory = join(out, "virtual-output");
  mkdirSync(join(parityDirectory, "tmp"), { recursive: true });
  const corpus = prepareColdScale(join(parityDirectory, "project"), provider, files, false);
  retainInputs(join(parityDirectory, "project"), join(out, "authored-inputs"));
  writeFileSync(join(parityDirectory, "corpus.json"), JSON.stringify(corpus, null, 2) + "\n");
  let completeOutput: unknown;
  let completeStdout: Buffer | undefined;
  let completeDebug: Buffer | undefined;
  const forwarder = prepareNativeForwarder(out);
  const nativeCalls: ReturnType<typeof collectNativePreflight>[] = [];
  let backend: string | undefined;
  for (const [role, build] of [
    ["base", base],
    ["head", head],
  ] as const) {
    for (const generated of [
      join(parityDirectory, "xdg-cache"),
      join(parityDirectory, "project/node_modules/.vize"),
      join(parityDirectory, "tmp"),
    ])
      rmSync(generated, { recursive: true, force: true });
    mkdirSync(join(parityDirectory, "tmp"), { recursive: true });
    const selected = coldScaleCommand({
      provider,
      out: parityDirectory,
      app: corpus.app,
      tool: "cli",
      binary: build.binary,
    });
    const { command } = selected;
    assert.equal(
      hash(selected.backend),
      "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    );
    if (backend !== undefined) assert.equal(selected.backend, backend);
    backend = selected.backend;
    const args = [...command.args];
    const capture = join(parityDirectory, `native-${role}`);
    mkdirSync(capture);
    const at = args.indexOf("--corsa-path");
    assert(at >= 0 && args[at + 1] === backend);
    args[at + 1] = forwarder.executable;
    const captured = run(
      parityDirectory,
      role,
      command.executable,
      [...args, "--show-virtual-ts", "--quiet"],
      corpus.app,
      {
        ...process.env,
        ...command.env,
        VIZE_COLD_NATIVE: backend,
        VIZE_COLD_CAPTURE: capture,
        VIZE_COLD_PROJECT: join(parityDirectory, "project"),
        VIZE_COLD_PROVIDER: provider,
      },
    );
    nativeCalls.push(collectNativePreflight(capture));
    assert.equal(captured.status, 0);
    verifyColdDiagnostics(captured.stdout.toString(), "cli", files, false);
    const value: unknown = JSON.parse(captured.stdout.toString());
    verifyStrictDebug(captured.stderr, value, corpus.app);
    if (completeOutput === undefined) {
      completeOutput = value;
      completeStdout = captured.stdout;
      completeDebug = captured.stderr;
    } else {
      assert.deepEqual(
        value,
        completeOutput,
        "full Virtual TS and complete result changed at the same authored project path",
      );
      assert(completeDebug?.equals(captured.stderr), "entire debug stderr bytes changed");
      assert(completeStdout?.equals(captured.stdout), "entire CLI stdout bytes changed");
    }
  }
  const nativeProof = backend
    ? compareNativePreflights(nativeCalls[0], nativeCalls[1], {
        executable: backend,
        sha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
        forwarder: forwarder.executable,
        project: join(parityDirectory, "project"),
        provider,
        materializedRoot: join(parityDirectory, "xdg-cache"),
        authoredCwd: corpus.app,
      })
    : null;
  if (nativeProof)
    writeFileSync(
      join(parityDirectory, "native-proof.json"),
      JSON.stringify(nativeProof, null, 2) + "\n",
    );
  const instrumented = [];
  for (const [role, normal] of [
    ["base", base],
    ["head", head],
  ] as const) {
    const root = join(out, "..", "instrumented", role);
    const source = JSON.parse(readFileSync(join(root, "source.json"), "utf8"));
    const build = readBuild(join(root, "build.json"));
    assert.equal(source.productSource, normal.sourceSha);
    assert.equal(source.instrumentedSource, build.sourceSha);
    assert.equal(build.allocation, true);
    assert.deepEqual(build.manifests, normal.manifests);
    assert.equal(hash(build.binary), build.binarySha256);
    for (const generated of ["project", "xdg-cache", "tmp"])
      rmSync(join(parityDirectory, generated), { recursive: true, force: true });
    mkdirSync(join(parityDirectory, "tmp"));
    const repeated = prepareColdScale(join(parityDirectory, "project"), provider, files, false);
    assert.equal(repeated.corpusSha256, corpus.corpusSha256);
    const selected = coldScaleCommand({
      provider,
      out: parityDirectory,
      app: repeated.app,
      tool: "cli",
      binary: build.binary,
    });
    const args = [...selected.command.args];
    const at = args.indexOf("--corsa-path");
    assert(at >= 0 && args[at + 1] === backend);
    args[at + 1] = forwarder.executable;
    const capture = join(parityDirectory, "native-instrumented-" + role);
    mkdirSync(capture);
    const trace = join(parityDirectory, "read-buffer.jsonl"),
      profile = join(parityDirectory, "read-profile.json");
    assert(!readFileExists(trace));
    const observed = run(
      parityDirectory,
      "instrumented-" + role,
      selected.command.executable,
      [...args, "--show-virtual-ts", "--quiet", "--profile-json", profile],
      repeated.app,
      {
        ...process.env,
        ...selected.command.env,
        VIZE_COLD_NATIVE: backend,
        VIZE_COLD_CAPTURE: capture,
        VIZE_COLD_PROJECT: join(parityDirectory, "project"),
        VIZE_COLD_PROVIDER: provider,
        VIZE_OWNED_READ_BUFFER_TRACE: trace,
      },
    );
    const native = collectNativePreflight(capture);
    assert.equal(observed.status, 0);
    assert(observed.stdout.equals(completeStdout!), "instrumented whole CLI output differs");
    assert(observed.stderr.equals(completeDebug!), "instrumented whole generated debug differs");
    assert.equal(native.length, nativeCalls[role === "base" ? 0 : 1].length);
    native.forEach((call, index) =>
      assertSameObservedNative(nativeCalls[role === "base" ? 0 : 1][index], call),
    );
    const bytes = readFileSync(trace),
      distribution = readBufferDistribution(bytes.toString("utf8"));
    const retained = join(root, "read-buffer.jsonl");
    copyFileSync(trace, retained);
    rmSync(trace);
    copyFileSync(profile, join(root, "profile.json"));
    rmSync(profile);
    writeFileSync(join(root, "distribution.json"), JSON.stringify(distribution, null, 2) + "\n");
    instrumented.push({
      role,
      source,
      build,
      traceSha256: hash(retained),
      readEvents: distribution.reads.length,
      finalRows: distribution.rows.length,
      actualReadBytes: distribution.actualReadBytes,
      readCapacityBytes: distribution.readCapacityBytes,
      retainedBytes: distribution.retainedBytes,
      retainedCapacityBytes: distribution.retainedCapacityBytes,
      joinedCopyBytes: distribution.joinedCopyBytes,
      borrowedOrUnjoinedFinalRows: distribution.borrowedOrUnjoinedFinalRows,
    });
  }
  let expected: { stdout: Buffer; stderr: Buffer } | undefined;
  const pairs: { pair: number; base: Sample; head: Sample }[] = [];
  for (const [pairIndex, order] of readBufferOrders.entries()) {
    const collected = new Map<string, Sample>();
    for (const [index, role] of order.entries()) {
      const build = role === "base" ? base : head,
        directory = join(out, pairIndex + 1 + "-" + role);
      const measured = await captureReadBufferCommand({
        provider,
        runRoot: parityDirectory,
        directory,
        files,
        binary: build.binary,
      });
      assert.equal(measured.report.corpusSha256, corpus.corpusSha256);
      if (!expected) expected = { stdout: measured.stdout, stderr: measured.stderr };
      else {
        assert(measured.stdout.equals(expected.stdout), "entire timed CLI stdout changed");
        assert(measured.stderr.equals(expected.stderr), "entire timed CLI stderr changed");
      }
      collected.set(role, {
        role,
        order: index + 1,
        directory,
        wallMs: measured.report.wallMs,
        peakRssBytes: measured.report.peakRssBytes,
        cpu: measured.report.cpu,
        rootPeakBytes: measured.report.rootPeakBytes,
        concurrentObservedTreePeakBytes: measured.report.concurrentObservedTreePeakBytes,
      });
    }
    pairs.push({ pair: pairIndex + 1, base: collected.get("base")!, head: collected.get("head")! });
  }
  const descriptive = (get: (sample: Sample) => number) => ({
    base: pairs.map((p) => get(p.base)),
    head: pairs.map((p) => get(p.head)),
    baseP50: nearestRank(
      pairs.map((p) => get(p.base)),
      0.5,
    ),
    headP50: nearestRank(
      pairs.map((p) => get(p.head)),
      0.5,
    ),
    scope: "four pairs only, descriptive population; no bootstrap/significance/precise tail claim",
  });
  const normalPacket = (directory: string) => {
    const census = JSON.parse(readFileSync(join(directory, "corpus.json"), "utf8")) as {
      app: string;
      entries: { path: string }[];
    };
    return coldScalePacketView(JSON.parse(readFileSync(join(directory, "stdout.txt"), "utf8")), {
      ...census,
      root: join(directory, "project"),
    });
  };
  let negative: unknown;
  for (const [role, build] of [
    ["base", base],
    ["head", head],
  ] as const) {
    const directory = join(out, `invalid-${role}`);
    captureColdScale({
      provider,
      out: directory,
      files,
      tool: "cli",
      invalid: true,
      binary: build.binary,
    });
    const packet = normalPacket(directory);
    if (negative === undefined) negative = packet;
    else assert.deepEqual(packet, negative, "complete invalid-result vector changed");
  }
  for (const tool of ["mapper", "vue-tsc"])
    for (const invalid of [false, true])
      captureColdScale({
        provider,
        out: join(out, `${invalid ? "invalid" : "valid"}-${tool}`),
        files,
        tool,
        invalid,
      });
  assert.equal(hash(join(provider, "package-lock.json")), providerLock);
  assert.equal(hash(base.binary), base.binarySha256);
  assert.equal(hash(head.binary), head.binarySha256);
  const summary = {
    instrumented,
    order: readBufferOrders,
    source: { base, head },
    providerLock,
    corpusSha256: corpus.corpusSha256,
    files,
    sfcs: 2500,
    pairs,
    wallMs: descriptive((s) => s.wallMs),
    peakRssBytes: descriptive((s) => s.peakRssBytes),
    cpuMs: descriptive((s) => s.cpu.totalCpuMs),
    rootRss: descriptive((s) => s.rootPeakBytes),
    concurrentTreeRss: descriptive((s) => s.concurrentObservedTreePeakBytes),
    cpuScope:
      "existing GNU time -v reported command user/system CPU, 10ms resolution; not native-only or function attribution",
    rssScope: "GNU time child high-water RSS; not simultaneous process-tree RSS",
    fullVirtualOutput:
      "same authored project path; complete JSON and debug bytes equal without normalization",
    sourceClosure,
    nativeProof,
    mapperReference: "unchanged frozen public0.596; not current source SDK acceptance",
    nativeLoadedFiles: "unobserved",
    untimedFlags:
      "--show-virtual-ts and --quiet only; timed argv and direct physical native remain unchanged",
    secondaryUntimedCommands:
      "after eight timings: two invalid source CLI controls plus valid/invalid frozen-public mapper and VueTSC (four commands)",
    environment: Object.fromEntries(
      [
        "GOMAXPROCS",
        "GOGC",
        "GOMEMLIMIT",
        "RAYON_NUM_THREADS",
        "CARGO_BUILD_JOBS",
        "CARGO_INCREMENTAL",
        "VIZE_BENCH_CPU_SET",
        "NODE_OPTIONS",
        "NODE_PATH",
        "PATH",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
    timedOutput:
      "fixed physical project path for all eight normal commands; whole stdout/stderr bytes equal without filename or diagnostic normalization",
    fullVirtualOutputSha256: createHash("sha256")
      .update(JSON.stringify(completeOutput))
      .digest("hex"),
    preflightStdoutSha256: createHash("sha256").update(completeStdout!).digest("hex"),
    preflightStderrSha256: createHash("sha256").update(completeDebug!).digest("hex"),
    allCorrectnessGuardsPassed: true,
    coldCacheQualified: false,
    comparisonQualified: true,
  };
  writeFileSync(join(out, "paired-comparison.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(
    JSON.stringify({
      base: base.sourceSha,
      head: head.sourceSha,
      wallMs: summary.wallMs,
      peakRssBytes: summary.peakRssBytes,
      cpuMs: summary.cpuMs,
      allCorrectnessGuardsPassed: true,
    }),
  );
  return summary;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      base: { type: "string" },
      head: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out && values.base && values.head);
  await pairedColdComparison({
    provider: resolve(values.provider),
    out: resolve(values.out),
    baseReceipt: resolve(values.base),
    headReceipt: resolve(values.head),
  });
}
