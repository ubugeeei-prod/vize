/** Hosted, post-exit memory-profile reader; never launches a compiler or a measured process. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { decodeMemoryProfile, memoryProfileLimits } from "./cold-typechecker-scale-memory-proto.ts";

const hash = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const typeUnits = [
  "alloc_objects/count",
  "alloc_space/bytes",
  "inuse_objects/count",
  "inuse_space/bytes",
];

export function parseRawMemorySamples(raw: string) {
  const lines = raw.split("\n"),
    start = lines.indexOf("Samples:"),
    end = lines.indexOf("Locations");
  assert(start >= 0 && end > start + 1);
  assert.equal(lines.filter((line) => line === "Samples:").length, 1);
  assert.equal(lines.filter((line) => line === "Locations").length, 1);
  const sampleTypes = lines[start + 1]
    .trim()
    .split(/\s+/u)
    .map((value) => value.replace(/\[dflt\]$/u, ""));
  assert.deepEqual([...sampleTypes].sort(), [...typeUnits].sort());
  const totals = [0n, 0n, 0n, 0n];
  const samples: { values: string[]; locations: string[]; rawLabels: string[] }[] = [];
  for (const line of lines.slice(start + 2, end)) {
    const match = /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+):\s*((?:\d+\s*)*)$/u.exec(line);
    if (match) {
      const values = match.slice(1, 5).map((value) => BigInt(value));
      values.forEach((value, index) => {
        totals[index] += value;
      });
      samples.push({
        values: values.map(String),
        locations: match[5].trim().split(/\s+/u).filter(Boolean),
        rawLabels: [],
      });
    } else {
      assert(
        samples.length > 0 && /^ {16}[^\s:]+:.*$/u.test(line),
        "unknown Go memory raw sample format",
      );
      samples[samples.length - 1].rawLabels.push(line);
    }
  }
  assert(samples.length > 0);
  for (const required of ["alloc_objects/count", "alloc_space/bytes"])
    assert(
      totals[sampleTypes.indexOf(required)] > 0n,
      "positive recorded allocation denominator required",
    );
  return { sampleTypes, totals: totals.map(String), samples };
}

/** Requires the unchanged CPU reader's successful Go/image/source custody receipt first. */
export function reportMemoryProfile(capture: string) {
  assert.equal(process.platform, "linux");
  assert.equal(fs.realpathSync(capture), capture);
  const read = (name: string) => JSON.parse(fs.readFileSync(path.join(capture, name), "utf8"));
  const write = (directory: string, name: string, value: unknown) =>
    fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n", {
      flag: "wx",
    });
  const out = path.join(capture, "native-memory-report");
  fs.mkdirSync(out);
  const status = read("native-profile.status.json"),
    validated = read("native-profile.validation.json");
  assert(Number.isSafeInteger(status.invokedPid) && status.invokedPid > 0);
  assert.equal(validated.passed, true);
  const name = `${status.invokedPid}-memprofile.pb.gz`,
    file = path.join(capture, "native-profile-data", name);
  const records = read("native-profile.files.json");
  assert(Array.isArray(records) && records.length === 2);
  const matching = records.filter((row) => row.path === file);
  assert.equal(matching.length, 1);
  assert.equal(matching[0].kind, "sampled-allocations-not-live-heap");
  assert.deepEqual(records, validated.profileFiles);
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && !stat.isSymbolicLink() && stat.nlink === 1);
  // The capture already retained the original file; an oversized reader input is not expanded.
  write(out, "request.json", {
    profile: file,
    bytes: stat.size,
    captureSha256: matching[0].sha256,
    limits: memoryProfileLimits,
    rawRetainedRemote: true,
    timingEligible: false,
  });
  assert.equal(stat.size, matching[0].bytes);
  assert(
    stat.size > 0 && stat.size <= memoryProfileLimits.compressedBytes,
    "remote memory reader compressed budget",
  );
  const body = fs.readFileSync(file);
  assert.equal(body.length, stat.size);
  assert.equal(hash(body), matching[0].sha256);
  const decoded = decodeMemoryProfile(body);
  assert.equal(decoded.uncompressedBytes, matching[0].uncompressedBytes);
  write(out, "memory-full-decoded.json", decoded);
  const cpu = read("native-profile-report/reader-and-denominator.json");
  assert.equal(cpu.goFullRawMatchesSampleDenominator, true);
  assert.equal(cpu.networkSymbolization, "disabled");
  assert.equal(records[0].kind, "cpu");
  assert.equal(records[0].sha256, cpu.profileSha256);
  assert.equal(
    records[0].path,
    path.join(capture, "native-profile-data", `${status.invokedPid}-cpuprofile.pb.gz`),
  );
  assert.match(cpu.reader.version, /^go version go1\.27\.2 /u);
  const go = cpu.reader.executable as string;
  assert(path.isAbsolute(go) && fs.realpathSync(go) === go);
  const readerHash = hash(fs.readFileSync(go));
  assert.equal(readerHash, cpu.reader.sha256);
  const env = {
    ...process.env,
    GOTOOLCHAIN: "local",
    GOPROXY: "off",
    GOSUMDB: "off",
    GOTELEMETRY: "off",
    GOWORK: "off",
    PPROF_TMPDIR: out,
  };
  const commands: unknown[] = [];
  const invoke = (label: string, args: string[]) => {
    const result = spawnSync(go, args, { env, maxBuffer: 64 * 1024 * 1024, timeout: 120_000 });
    for (const stream of ["stdout", "stderr"] as const)
      if (Buffer.isBuffer(result[stream]))
        fs.writeFileSync(path.join(out, label + "." + stream + ".bin"), result[stream], {
          flag: "wx",
        });
    commands.push({
      executable: go,
      args,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      pid: result.pid ?? null,
      stdoutAvailable: Buffer.isBuffer(result.stdout),
      stderrAvailable: Buffer.isBuffer(result.stderr),
    });
    write(out, `${label}.process.json`, commands.at(-1));
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0);
    assert(Buffer.isBuffer(result.stdout) && Buffer.isBuffer(result.stderr));
    return new TextDecoder("utf-8", { fatal: true }).decode(result.stdout);
  };
  const raw = parseRawMemorySamples(
    invoke("memory-full-raw", ["tool", "pprof", "-raw", "-symbolize=none", file]),
  );
  assert.deepEqual(
    raw.sampleTypes,
    decoded.sampleTypes.map((row) => `${row.type}/${row.unit}`),
  );
  assert.deepEqual(raw.totals, decoded.sampleTotals);
  assert.deepEqual(
    raw.samples.map(({ values, locations }) => ({ values, locations })),
    decoded.samples.map(({ values, locations }) => ({ values, locations })),
    "full ordered Go sample weights/location IDs disagree",
  );
  for (const type of decoded.sampleTypes) {
    const flags = [
      "tool",
      "pprof",
      "-symbolize=none",
      "-nodefraction=0",
      "-edgefraction=0",
      "-nodecount=0",
      `-sample_index=${type.type}`,
      "-top",
    ];
    invoke(type.type + "-all-flat", [...flags, file]);
    invoke(type.type + "-all-cumulative", [...flags, "-cum", file]);
  }
  assert.equal(hash(fs.readFileSync(go)), readerHash);
  assert.equal(hash(fs.readFileSync(file)), hash(body));
  const receipt = {
    reader: cpu.reader,
    profileSha256: hash(body),
    commands,
    limits: memoryProfileLimits,
    sampleRows: decoded.samples.length,
    sampleTypes: decoded.sampleTypes,
    exactSampleTotals: decoded.sampleTotals,
    period: decoded.period,
    periodType: decoded.periodType,
    durationNanos: decoded.durationNanos,
    allSamplesCountedOnce: true,
    allTablesUnbounded: true,
    goFullRawOrderedSamplesEqual: true,
    networkSymbolization: "disabled",
    instrumentationExcludedFromTiming: true,
    inuseScope: decoded.inuseScope,
    weightScope: decoded.weightScope,
    sourceScope: decoded.sourceScope,
  };
  write(out, "reader-and-denominators.json", receipt);
  return receipt;
}
