/** Remote-only full CPU profile decode. No checker execution or network symbolization. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { decodeCpuProfile } from "./cold-typechecker-scale-cpu-proto.ts";

export function parseRawCpuSamples(raw: string) {
  const lines = raw.split("\n"),
    start = lines.indexOf("Samples:"),
    end = lines.indexOf("Locations");
  assert(start >= 0 && end > start + 1);
  assert.equal(lines.filter((line) => line === "Samples:").length, 1);
  const sampleTypes = lines[start + 1]
    .trim()
    .split(/\s+/u)
    .map((type) => type.replace(/\[dflt\]$/u, ""));
  assert.deepEqual(sampleTypes, ["samples/count", "cpu/nanoseconds"]);
  const totals = [0n, 0n],
    samples: { values: string[]; locations: string[]; rawLabels: string[] }[] = [];
  for (const line of lines.slice(start + 2, end)) {
    const match = /^\s*(\d+)\s+(\d+):\s*((?:\d+\s*)*)$/u.exec(line);
    if (match) {
      const values = [BigInt(match[1]), BigInt(match[2])];
      values.forEach((v, i) => {
        totals[i] += v;
      });
      samples.push({
        values: values.map(String),
        locations: match[3].trim() ? match[3].trim().split(/\s+/u) : [],
        rawLabels: [],
      });
    } else {
      assert(
        samples.length > 0 && /^ {16}\S.*$/u.test(line),
        "unknown Go -raw sample format; preserve raw and refuse qualification",
      );
      samples[samples.length - 1].rawLabels.push(line);
    }
  }
  assert(samples.length > 0);
  assert(totals[0] > 0n && totals[1] > 0n, "CPU attribution requires positive measured samples");
  return { sampleTypes, totals: totals.map(String), samples };
}

export function reportCpuProfile(capture: string) {
  const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
  const read = (name: string) =>
    JSON.parse(fs.readFileSync(path.join(capture, name), "utf8")) as Record<string, unknown>;
  const out = path.join(capture, "native-profile-report");
  fs.mkdirSync(out);
  const write = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
  const status = read("native-profile.status.json");
  assert(Number.isInteger(status.invokedPid) && Number(status.invokedPid) > 0);
  const cpu = path.join(capture, "native-profile-data", `${status.invokedPid}-cpuprofile.pb.gz`);
  const decoded = decodeCpuProfile(fs.readFileSync(cpu));
  write("cpu-full-decoded.json", decoded);
  const goPath = (process.env.PATH ?? "")
    .split(path.delimiter)
    .map((p) => path.join(p, "go"))
    .find((p) => path.isAbsolute(p) && fs.existsSync(p));
  assert(goPath);
  const go = fs.realpathSync(goPath);
  const readerBefore = hash(go);
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
  function invoke(name: string, args: string[]) {
    const result = spawnSync(go, args, {
      env,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      timeout: 120_000,
    });
    fs.writeFileSync(path.join(out, name + ".stdout.txt"), result.stdout ?? "", { flag: "wx" });
    fs.writeFileSync(path.join(out, name + ".stderr.txt"), result.stderr ?? "", { flag: "wx" });
    commands.push({
      executable: go,
      args,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
    });
    fs.writeFileSync(
      path.join(out, "reader-commands.json"),
      JSON.stringify(commands, null, 2) + "\n",
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0);
    return result.stdout;
  }
  const version = invoke("reader-version", ["version"]);
  assert.match(version, /^go version go1\.27\.2 /u);
  const request = read("request.json"),
    spec = request.spec as Record<string, unknown>;
  assert(typeof spec.backend === "string");
  assert.equal(hash(spec.backend), spec.backendSha256);
  const imageMetadata = invoke("native-image-go-build", ["version", "-m", spec.backend]);
  assert(
    /vcs\.modified=true/u.test(imageMetadata),
    "current same-image dirty-source limit must remain explicit",
  );
  const raw = invoke("cpu-full-raw", ["tool", "pprof", "-raw", "-symbolize=none", cpu]);
  const rawSamples = parseRawCpuSamples(raw);
  assert.deepEqual(rawSamples.totals, decoded.sampleTotals);
  assert.equal(
    rawSamples.samples.length,
    decoded.samples.length,
    "Go reader changed sample population",
  );
  assert.deepEqual(
    rawSamples.samples.map(({ values, locations }) => ({ values, locations })),
    decoded.samples.map(({ values, locations }) => ({ values, locations })),
    "Go reader changed ordered sample weights/locations",
  );
  const tableFlags = [
    "tool",
    "pprof",
    "-symbolize=none",
    "-nodefraction=0",
    "-edgefraction=0",
    "-nodecount=0",
    "-sample_index=cpu",
    "-top",
  ];
  invoke("cpu-all-flat", [...tableFlags, cpu]);
  invoke("cpu-all-cumulative", [...tableFlags, "-cum", cpu]);
  const validation = read("native-profile.validation.json");
  assert.equal(validation.passed, true);
  assert.equal(hash(go), readerBefore, "profile reader changed during decode");
  write("reader-and-denominator.json", {
    reader: { executable: go, sha256: readerBefore, unchangedAfterDecode: true, version },
    commands,
    nativeImageSha256: spec.backendSha256,
    profileSha256: hash(cpu),
    sampleRows: decoded.samples.length,
    sampleTypes: decoded.sampleTypes,
    exactSampleTotals: decoded.sampleTotals,
    period: decoded.period,
    periodType: decoded.periodType,
    timeNanos: decoded.timeNanos,
    durationNanos: decoded.durationNanos,
    functions: decoded.functions.length,
    locations: decoded.locations.length,
    mappings: decoded.mappings.length,
    inlineFrameRows: decoded.locations.reduce((sum, location) => sum + location.lines.length, 0),
    unknownLeafCpuNs: decoded.unknownLeafCpuNs,
    samplesWithAnyUnknownFrameCpuNs: decoded.samplesWithAnyUnknownFrameCpuNs,
    namedLeafCpuNs: (BigInt(decoded.sampleTotals[1]) - BigInt(decoded.unknownLeafCpuNs)).toString(),
    knownLostProfileEventCpuNs: decoded.knownLostProfileEventCpuNs,
    lossScope: decoded.lossScope,
    goFullRawMatchesSampleDenominator: true,
    allTablesUnbounded: true,
    networkSymbolization: "disabled",
    sourceLineScope: decoded.sourceLineScope,
    cpuScope: decoded.cpuScope,
    instrumentationExcludedFromTiming: true,
  });
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  assert(process.argv[2]);
  reportCpuProfile(fs.realpathSync(process.argv[2]));
}
