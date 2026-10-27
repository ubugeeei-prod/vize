/** Small original carrier first; optional replay only decodes retained normal samples. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { calibrateSourcePmu } from "./source-pmu-calibration.ts";
import { captureSourceOperationPerf } from "./source-operation-perf.ts";
import { frozenSourceLocations } from "./source-symbolizer-protocol.ts";
import { renderSymbolCpu } from "./source-symbolizer-symbols.ts";
import {
  boundedCommand,
  digest,
  createSentinelAdapter,
  probeDecoders,
  renderSourceCpu,
} from "./source-symbolizer-render.ts";

const json = (file: string) => JSON.parse(readFileSync(file, "utf8"));
export function runCarrierSmoke(options: { perf: string; out: string; decoders: string[] }) {
  assert.ok(!existsSync(options.out), "fresh evidence output required");
  mkdirSync(options.out, { recursive: true });
  const binary = join(options.out, "carrier");
  const hardware = calibrateSourcePmu({
    binary,
    perf: options.perf,
    out: join(options.out, "hardware-admission"),
    rounds: 1,
  });
  const llvm = options.decoders.find(
    (decoder) => decoder.includes("llvm-addr2line") && existsSync(decoder),
  );
  const adapter = llvm ? createSentinelAdapter(join(options.out, "adapter"), llvm) : null;
  const probes = probeDecoders(binary, join(options.out, "protocol-probes"), [
    ...(adapter ? [adapter] : []),
    ...options.decoders,
  ]);
  writeFileSync(join(options.out, "probes.json"), JSON.stringify(probes, null, 2) + "\n");
  const selected = probes.find((probe) => probe.qualified)?.decoder;
  assert.ok(selected, "no decoder passed the exact sentinel plus original Rust source witness");
  const args = ["instructions", "10000000", "64"];
  const normals = [0, 1, 2].map((ordinal) =>
    boundedCommand(join(options.out, "normal-" + ordinal), binary, args, 30_000),
  );
  for (const run of normals) assert.equal(run.status, 0);
  const packets = normals.map((run) =>
    json(join(options.out, "normal-" + normals.indexOf(run), "stdout.txt")),
  );
  for (const packet of packets) {
    assert.equal(packet.schema, "vize-pmu-calibration");
    assert.equal(packet.completed, true);
    assert.equal(packet.mode, "instructions");
    assert.equal(packet.iterations, 10000000);
    assert.deepEqual(packet, packets[0]);
  }
  const capture = captureSourceOperationPerf({
    mode: "record",
    perfExecutable: options.perf,
    addr2lineExecutable: selected,
    outputDir: join(options.out, "capture"),
    timeoutMs: 30_000,
    reportTimeoutMs: 30_000,
    command: {
      executable: binary,
      args,
      cwd: options.out,
      env: { LC_ALL: "C", LLVM_ADDR2LINE_OPTS: "", LLVM_SYMBOLIZER_OPTS: "" },
    },
    identity: {
      sourceSha256: hardware.sourceSha256,
      binarySha256: hardware.binarySha256,
      build: hardware.build,
      scope: "original Rust carrier; single process, no Vize speed claim",
    },
  });
  assert.equal(capture.collection, "collected");
  assert.equal(capture.perf.status, 0);
  assert.deepEqual(json(join(options.out, "capture", "stdout.txt")), packets[0]);
  assert.ok(capture.sampleCount !== null && capture.sampleCount > 0);
  const source = renderSourceCpu({
    binary,
    data: join(options.out, "capture", "perf.data"),
    decoder: selected,
    perf: options.perf,
    out: join(options.out, "decoded"),
    totalSamples: capture.sampleCount,
    requiredSource: "pmu_calibration.rs",
  });
  const medianNormalMs = normals.map((run) => run.wallMs).sort((a, b) => a - b)[1];
  const captureMs = Date.parse(capture.captureFinishedAt) - Date.parse(capture.startedAt);
  const result = {
    schemaVersion: 1,
    qualified: source.qualified,
    selectedDecoder: selected,
    sourceSha256: hardware.sourceSha256,
    binarySha256: hardware.binarySha256,
    hardware: {
      qualified: hardware.qualified,
      groups: hardware.groups,
      counters: "raw admission files retained separately",
    },
    cpuClock: source,
    profilerOverhead: {
      normalFreshWallMs: normals.map((run) => run.wallMs),
      medianNormalMs,
      captureMs,
      captureToNormalRatio: captureMs / medianNormalMs,
      scope:
        "one software capture including perf startup/drain; excludes report and setup; diagnostic overhead estimate, not a Vize speed measurement",
    },
  };
  writeFileSync(join(options.out, "result.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(
    result.qualified,
    "source samples did not qualify; inspect raw protocol and source reports",
  );
  return result;
}

export function retainedNormalIdentity(root: string) {
  const original = json(join(root, "record-head/direct-process/receipt.json"));
  const build = json(join(root, "builds/normal-head.json"));
  const binary = join(root, "binaries/normal-head/vize");
  assert.equal(original.mode, "record");
  assert.equal(original.metric, "software-user-cpu-clock-samples");
  assert.equal(original.samplingHz, 99);
  assert.equal(original.dwarfStackBytes, 4096);
  assert.equal(
    original.command.executable,
    binary,
    "restore the exact captured DSO path before replay",
  );
  assert.equal(build.role, "normal-head");
  assert.equal(build.binary, binary);
  assert.equal(statSync(binary).size, build.binaryBytes);
  assert.equal(
    statSync(join(root, "record-head/direct-process/perf.data")).size,
    original.dataBytes,
  );
  assert.equal(build.sourceSha, original.identity.source);
  assert.equal(build.binarySha256, original.binarySha256);
  assert.equal(original.binarySha256After, original.binarySha256);
  assert.equal(digest(binary), original.binarySha256);
  assert.equal(build.targetStartedEmpty, true);
  assert.equal(build.cargoArtifact.fresh, false);
  assert.equal(build.allocation, false);
  assert.ok(!build.cargoArtifact.features.includes("profiling"));
  assert.ok(Number.isSafeInteger(original.sampleCount) && original.sampleCount > 0);
  return { original, build, binary };
}

export function replayNormal(options: {
  perf: string;
  out: string;
  decoder: string;
  root: string;
}) {
  assert.ok(!existsSync(options.out), "fresh replay evidence required");
  const { original, build, binary } = retainedNormalIdentity(options.root);
  const dwarf = boundedCommand(
    join(options.out, "sections"),
    "readelf",
    ["--section-headers", "--wide", binary],
    10_000,
  );
  assert.equal(dwarf.status, 0);
  assert.ok(dwarf.stdout.includes(".debug_line"), "actual normal binary has no line tables");
  const symbols = renderSymbolCpu({
    binary,
    data: join(options.root, "record-head/direct-process/perf.data"),
    perf: options.perf,
    out: join(options.out, "symbols"),
    totalSamples: original.sampleCount,
    identity: { compiledSource: build.sourceSha, build, originalCapture: original },
  });
  assert.ok(symbols.qualified, "exact ELF symbol-only self census failed");
  const normalProbes = probeDecoders(binary, join(options.out, "normal-query"), [options.decoder], {
    symbol: "generate_vue_virtual_ts",
    source: ".rs:",
    timeoutMs: 10_000,
  });
  writeFileSync(
    join(options.out, "normal-probes.json"),
    JSON.stringify(normalProbes, null, 2) + "\n",
  );
  assert.ok(normalProbes[0]?.qualified, "normal first source query failed before full replay");
  const decoded = renderSourceCpu({
    binary,
    data: join(options.root, "record-head/direct-process/perf.data"),
    decoder: options.decoder,
    perf: options.perf,
    out: join(options.out, "decoded"),
    totalSamples: original.sampleCount,
    reportTimeoutMs: 30_000,
  });
  const result = {
    schemaVersion: 1,
    qualified: decoded.qualified,
    symbols,
    compiledSource: build.sourceSha,
    observerSource: process.env.GITHUB_SHA ?? null,
    sourceLocations: frozenSourceLocations(
      decoded.reports.self.rows,
      build.sourceRoot,
      build.sourceSha,
    ),
    actualBinary: basename(binary),
    binarySha256: original.binarySha256,
    originalCapture: original,
    build,
    decoded,
    scope:
      "post-processing of the exact retained normal check capture; no new timing or hardware instruction claim",
  };
  writeFileSync(join(options.out, "normal-replay.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(result.qualified, "normal Rust source samples remain unqualified");
  return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      perf: { type: "string" },
      out: { type: "string" },
      decoders: {
        type: "string",
        default: "/usr/bin/addr2line,/usr/bin/llvm-addr2line,/usr/bin/eu-addr2line",
      },
      "replay-root": { type: "string" },
      "carrier-result": { type: "string" },
      "bounded-result": { type: "string" },
    },
  });
  assert.ok(values.perf && values.out);
  if (values["replay-root"]) {
    assert.ok(values["carrier-result"]);
    const carrier = json(resolve(values["carrier-result"]));
    assert.equal(carrier.qualified, true);
    assert.ok(values["bounded-result"], "1/10/100 controls must precede retained full replay");
    const bounded = json(resolve(values["bounded-result"]));
    const retained = retainedNormalIdentity(resolve(values["replay-root"]));
    assert.equal(bounded.qualified, true);
    assert.equal(bounded.binarySha256, retained.original.binarySha256);
    assert.equal(bounded.backendSha256, retained.original.identity.backendSha256);
    assert.deepEqual(
      bounded.stages.map((stage: { sfcCount: number }) => stage.sfcCount),
      [1, 10, 100],
    );
    assert.ok(
      bounded.stages.every(
        (stage: { accuracyQualified: boolean; captureCompleted: boolean }) =>
          stage.accuracyQualified && stage.captureCompleted,
      ),
    );
    replayNormal({
      perf: values.perf,
      out: resolve(values.out),
      decoder: carrier.selectedDecoder,
      root: resolve(values["replay-root"]),
    });
  } else
    runCarrierSmoke({
      perf: values.perf,
      out: resolve(values.out),
      decoders: values.decoders!.split(","),
    });
}
