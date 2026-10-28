/** Attribute every captured checker self sample using only its bound SDK pclntab. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { boundedCommand, digest } from "./source-symbolizer-render.ts";
import { bindCheckerProgramCounters } from "./native-checker-cpu-pcs.ts";

export function parseGoAddressFrames(output: string, pcs: string[]) {
  const lines = output.trimEnd().replaceAll("\r\n", "\n").split("\n");
  assert.equal(lines.length, (pcs.length + 2) * 2, "two output lines required for every input PC");
  for (const index of [0, pcs.length + 1]) {
    assert.equal(lines[index * 2], "?", "out-of-image negative PC resolved unexpectedly");
    assert.equal(lines[index * 2 + 1], "?:0");
  }
  return pcs.map((pc, index) => {
    const name = lines[(index + 1) * 2],
      location = lines[(index + 1) * 2 + 1];
    const match = /^(.*):(\d+)$/u.exec(location);
    assert.ok(match, "malformed Go file:line response");
    const line = Number(match[2]);
    assert.ok(Number.isSafeInteger(line));
    if (name === "?") assert.equal(location, "?:0");
    return {
      elfPc: pc,
      function: name,
      location,
      line,
      functionResolved: name.length > 0 && name !== "?",
      imageLineInfoResolved: name.length > 0 && name !== "?" && match[1] !== "?" && line > 0,
      sourceTreeOwnershipVerified: false,
      sourceUrl: null,
    };
  });
}
export function decodeNativeCheckerCpu(options: {
  sdk: string;
  sdkSha256: string;
  data: string;
  dataSha256: string;
  trace: string;
  elf: string;
  out: string;
  go: string;
  identity: Record<string, unknown>;
}) {
  assert.ok(!existsSync(options.out));
  mkdirSync(options.out, { recursive: true });
  assert.equal(digest(options.sdk), options.sdkSha256);
  assert.equal(digest(options.data), options.dataSha256);
  const mapping = bindCheckerProgramCounters(
    readFileSync(options.trace, "utf8"),
    readFileSync(options.elf, "utf8"),
    options.sdk,
  );
  writeFileSync(join(options.out, "pc-mapping.json"), JSON.stringify(mapping, null, 2) + "\n");
  assert.ok(mapping.qualified, "complete MMAP/PT_LOAD/dsoff binding failed");
  const pcs = mapping.uniqueElfPcs;
  const input = ["0x0", ...pcs, "0xdeadbeefbadc0ffe"].join("\n") + "\n";
  writeFileSync(join(options.out, "elf-pcs.txt"), input);
  const tool = boundedCommand(
    join(options.out, "go-addresses"),
    options.go,
    ["tool", "addr2line", options.sdk],
    30_000,
    input,
    { env: { GOTOOLCHAIN: "local", GOTELEMETRY: "off", GOWORK: "off" } },
  );
  assert.equal(tool.status, 0, tool.stderr);
  const decoded = parseGoAddressFrames(tool.stdout, pcs);
  const byPc = new Map(decoded.map((entry) => [entry.elfPc, entry]));
  const functions = new Map<string, { function: string; samples: number; periodSumRaw: bigint }>();
  let resolvedSamples = 0,
    lineInfoSamples = 0,
    periodSum = 0n;
  for (const row of mapping.rows) {
    const entry = byPc.get(row.elfPc);
    assert.ok(entry);
    const group = functions.get(entry.function) ?? {
      function: entry.function,
      samples: 0,
      periodSumRaw: 0n,
    };
    group.samples++;
    group.periodSumRaw += BigInt(row.period);
    functions.set(entry.function, group);
    periodSum += BigInt(row.period);
    if (entry.functionResolved) resolvedSamples++;
    if (entry.imageLineInfoResolved) lineInfoSamples++;
  }
  const functionRows = [...functions.values()]
    .map((entry) => ({ ...entry, periodSumRaw: entry.periodSumRaw.toString() }))
    .sort((left, right) => right.samples - left.samples);
  assert.equal(
    functionRows.reduce((sum, row) => sum + row.samples, 0),
    1380,
  );
  const unchanged =
    digest(options.sdk) === options.sdkSha256 && digest(options.data) === options.dataSha256;
  const result = {
    schemaVersion: 1,
    metric: "same-image Go function software CPU self samples",
    qualified: unchanged && resolvedSamples > 0,
    goFunctionsQualified: unchanged && resolvedSamples > 0,
    allSampleCount: mapping.allSampleCount,
    checkerSampleCount: mapping.checkerSampleCount,
    resolvedFunctionSamples: resolvedSamples,
    unresolvedFunctionSamples: 1380 - resolvedSamples,
    imageLineInfoSamples: lineInfoSamples,
    uniqueElfPcs: pcs.length,
    periodSumRaw: periodSum.toString(),
    periodScope: "recorded user cpu-clock event periods",
    sdkSha256: options.sdkSha256,
    dataSha256: options.dataSha256,
    traceSha256: digest(options.trace),
    elfHeadersSha256: digest(options.elf),
    goToolSha256: digest(options.go),
    identity: options.identity,
    unchanged,
    conversion: mapping.conversion,
    observedLoadBiases: [...new Set(mapping.rows.map((row) => row.loadBias))],
    decodedPcs: decoded,
    functions: functionRows,
    sourceTreeOwnershipVerified: false,
    hardwareInstructions: null,
    inclusiveCosts: null,
    callCounts: null,
    coldQualified: false,
    latencyMeasured: false,
    scope:
      "all original checker self samples accounted; file:line hints come from the identical SDK image; source text and ownership remain unverified",
    postProcessingOutsideMeasuredWindow: true,
  };
  writeFileSync(join(options.out, "go-cpu.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(result.qualified);
  return result;
}
