/** A/A noise and one separate allocation profile over the existing complete corpus. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { captureColdScale, coldScaleSupportScriptFiles } from "./cold-typechecker-scale.ts";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";

const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
type Build = {
  sourceSha: string;
  treeSha: string;
  binary: string;
  binarySha256: string;
  allocation: boolean;
  cargoArtifact: { features: string[] };
  cargo: { args: string[] };
  rustc: string;
};
export function nearestRank(values: number[], p: number): number {
  assert.ok(values.length > 0 && values.every(Number.isFinite) && p > 0 && p <= 1);
  return [...values].sort((a, b) => a - b)[Math.ceil(values.length * p) - 1];
}
export function linuxPeakRss(stderr: string): number {
  const matches = [...stderr.matchAll(/^\s*Maximum resident set size \(kbytes\): (\d+)$/gmu)];
  assert.equal(matches.length, 1, "one GNU time RSS record required");
  const bytes = Number(matches[0][1]) * 1024;
  assert.ok(Number.isSafeInteger(bytes) && bytes > 0);
  return bytes;
}

export function phaseBaseline(options: {
  provider: string;
  out: string;
  source: string;
  normalReceipt: string;
  allocationReceipt: string;
}) {
  const { provider, out, source } = options;
  const readBuild = (file: string) => JSON.parse(readFileSync(file, "utf8")) as Build;
  const normal = readBuild(options.normalReceipt),
    allocation = readBuild(options.allocationReceipt);
  assert.equal(normal.sourceSha, source);
  assert.equal(allocation.sourceSha, source);
  assert.equal(normal.treeSha, allocation.treeSha);
  assert.equal(normal.allocation, false);
  assert.equal(allocation.allocation, true);
  assert.equal(normal.cargoArtifact.features.includes("profiling"), false);
  assert.equal(allocation.cargoArtifact.features.includes("profiling"), true);
  assert.equal(normal.rustc, allocation.rustc);
  assert.deepEqual(allocation.cargo.args, [...normal.cargo.args, "--features", "profiling"]);
  assert.equal(hash(normal.binary), normal.binarySha256);
  assert.equal(hash(allocation.binary), allocation.binarySha256);
  const providerLock = hash(join(provider, "package-lock.json"));
  const files = 2500 + coldScaleSupportScriptFiles;
  const samples: {
    pair: number;
    role: string;
    order: number;
    wallMs: number;
    peakRssBytes: number;
    corpusSha256: string;
  }[] = [];
  const view = (directory: string) => {
    const corpus = JSON.parse(readFileSync(join(directory, "corpus.json"), "utf8")) as {
      app: string;
      entries: { path: string }[];
    };
    const packet: unknown = JSON.parse(readFileSync(join(directory, "stdout.txt"), "utf8"));
    return coldScalePacketView(packet, { ...corpus, root: join(directory, "project") });
  };
  const allocationDirectory = join(out, "allocation");
  const allocationMeasurement = captureColdScale({
    provider,
    out: allocationDirectory,
    files,
    tool: "profile",
    invalid: false,
    binary: allocation.binary,
  });
  const profile = JSON.parse(readFileSync(join(allocationDirectory, "profile.json"), "utf8")) as {
    allocation: unknown;
    spans: unknown[];
    counters: unknown[];
  };
  assert.ok(
    profile.allocation &&
      Array.isArray(profile.spans) &&
      profile.spans.length > 0 &&
      Array.isArray(profile.counters),
  );
  const wholePacket = view(allocationDirectory);
  writeFileSync(
    join(out, "allocation-phase.json"),
    JSON.stringify(
      { source, normal, allocation, profile, corpusSha256: allocationMeasurement.corpusSha256 },
      null,
      2,
    ) + "\n",
  );
  for (const tool of ["mapper", "vue-tsc"])
    captureColdScale({ provider, out: join(out, `valid-${tool}`), files, tool, invalid: false });
  for (const tool of ["cli", "mapper", "vue-tsc"])
    captureColdScale({
      provider,
      out: join(out, `invalid-${tool}`),
      files,
      tool,
      invalid: true,
      ...(tool === "cli" ? { binary: normal.binary } : {}),
    });
  for (let pair = 1; pair <= 4; pair++) {
    const order = pair % 2 === 1 ? ["A1", "A2"] : ["A2", "A1"];
    for (const [index, role] of order.entries()) {
      const directory = join(out, `${pair}-${role}`);
      const measurement = captureColdScale({
        provider,
        out: directory,
        files,
        tool: "cli",
        invalid: false,
        binary: normal.binary,
      });
      assert.deepEqual(
        view(directory),
        wholePacket,
        "complete A/A and allocation JSON packet changed",
      );
      assert.equal(measurement.corpusSha256, allocationMeasurement.corpusSha256);
      samples.push({
        pair,
        role,
        order: index + 1,
        wallMs: measurement.wallMs,
        peakRssBytes: linuxPeakRss(readFileSync(join(directory, "stderr.txt"), "utf8")),
        corpusSha256: measurement.corpusSha256,
      });
    }
  }
  assert.equal(new Set(samples.map((sample) => sample.corpusSha256)).size, 1);
  assert.equal(hash(join(provider, "package-lock.json")), providerLock);
  assert.equal(hash(normal.binary), normal.binarySha256);
  assert.equal(hash(allocation.binary), allocation.binarySha256);
  const summary = {
    role: "same-source A/A noise and separate allocation/phase baseline; no optimization claim",
    source,
    tree: normal.treeSha,
    files,
    sfcs: 2500,
    providerLock,
    normal,
    allocation,
    samples,
    wholeOutputSha256: createHash("sha256").update(JSON.stringify(wholePacket)).digest("hex"),
    wholeOutputComparison:
      "only declared filename identities mapped through each exact authored census; every other field and order exact",
    wallMs: {
      p50: nearestRank(
        samples.map((sample) => sample.wallMs),
        0.5,
      ),
      p95: nearestRank(
        samples.map((sample) => sample.wallMs),
        0.95,
      ),
      tailLimit: "eight samples are a limited noise description",
    },
    peakRssBytes: {
      p50: nearestRank(
        samples.map((sample) => sample.peakRssBytes),
        0.5,
      ),
      p95: nearestRank(
        samples.map((sample) => sample.peakRssBytes),
        0.95,
      ),
      scope: "GNU time child high-water RSS; not simultaneous process-tree RSS",
    },
    phaseAllocation: profile,
    affinity: process.env.VIZE_BENCH_CPU_SET ?? null,
    correctness: "complete existing authored graph and all independent valid/invalid guards passed",
    improvementQualified: false,
    coldCacheQualified: false,
    instrumentedTimingUsable: false,
  };
  mkdirSync(out, { recursive: true });
  writeFileSync(join(out, "phase-baseline.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(
    JSON.stringify({
      source,
      samples: samples.length,
      wallMs: summary.wallMs,
      peakRssBytes: summary.peakRssBytes,
      allocation: profile.allocation,
    }),
  );
  return summary;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      source: { type: "string" },
      normal: { type: "string" },
      allocation: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out && values.source && values.normal && values.allocation);
  assert.match(values.source, /^[a-f0-9]{40}$/u);
  phaseBaseline({
    provider: resolve(values.provider),
    out: resolve(values.out),
    source: values.source,
    normalReceipt: resolve(values.normal),
    allocationReceipt: resolve(values.allocation),
  });
}
