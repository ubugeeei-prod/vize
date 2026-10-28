/** Independent ELF symbol-only self samples remain usable without source-line IPC. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { boundedCommand, digest } from "./source-symbolizer-render.ts";
import { parseFlatSymbolReport } from "./source-symbolizer-protocol.ts";
import type { SymbolCpuRow } from "./source-symbolizer-protocol.ts";

export function symbolSelfCensus(rows: SymbolCpuRow[], totalSamples: number, targetDso: string) {
  const known =
    rows.length > 0 &&
    rows.every((row) => row.samples !== null && row.dso.length > 0 && row.symbol.length > 0);
  const groups = new Map<string, { samples: number; resolvedSymbolSamples: number }>();
  for (const row of rows) {
    const group = groups.get(row.dso) ?? { samples: 0, resolvedSymbolSamples: 0 };
    group.samples += row.samples ?? 0;
    if (
      !/^(?:\[[^\]]+\]\s*)?(?:0x[0-9a-f]+|\[unknown\])$/iu.test(row.symbol) &&
      row.symbol.length > 0
    )
      group.resolvedSymbolSamples += row.samples ?? 0;
    groups.set(row.dso, group);
  }
  const allSelfSamples = known ? rows.reduce((sum, row) => sum + (row.samples ?? 0), 0) : null;
  const allDsos = [...groups].map(([dso, group]) => ({
    dso,
    samples: known ? group.samples : null,
    resolvedSymbolSamples: known ? group.resolvedSymbolSamples : null,
    unresolvedSymbolSamples: known ? group.samples - group.resolvedSymbolSamples : null,
  }));
  const target = groups.get(targetDso);
  return {
    qualified:
      known &&
      Number.isSafeInteger(totalSamples) &&
      totalSamples > 0 &&
      allSelfSamples === totalSamples &&
      (target?.resolvedSymbolSamples ?? 0) > 0,
    allSelfSamples,
    totalSamples,
    allDsos,
    targetDso,
    sourceLineQualified: false,
    hardwareInstructions: null,
  };
}
export function renderSymbolCpu(options: {
  binary: string;
  data: string;
  perf: string;
  out: string;
  totalSamples: number;
  identity: Record<string, unknown>;
}) {
  assert.ok(!existsSync(options.out));
  mkdirSync(options.out, { recursive: true });
  const sections = boundedCommand(
    join(options.out, "sections"),
    "readelf",
    ["--section-headers", "--wide", options.binary],
    10_000,
  );
  assert.equal(sections.status, 0);
  assert.match(sections.stdout, /\s\.symtab\s/u);
  const notes = boundedCommand(
    join(options.out, "build-id"),
    "readelf",
    ["--notes", options.binary],
    10_000,
  );
  assert.equal(notes.status, 0);
  const version = boundedCommand(
    join(options.out, "perf-version"),
    options.perf,
    ["--version"],
    3000,
  );
  assert.equal(version.status, 0);
  const before = digest(options.binary),
    dataSha256 = digest(options.data);
  const command = boundedCommand(
    join(options.out, "report"),
    options.perf,
    [
      "report",
      "--stdio",
      "--no-children",
      "--call-graph",
      "none",
      "--no-inline",
      "--show-nr-samples",
      "--show-total-period",
      "--fields",
      "overhead,sample,period,dso,symbol",
      "--field-separator",
      "\t",
      "--percent-limit",
      "0",
      "--percentage",
      "absolute",
      "--sort",
      "dso,symbol",
      "--input",
      options.data,
    ],
    30_000,
  );
  const parsed = parseFlatSymbolReport(command.stdout);
  const census = symbolSelfCensus(parsed.rows, options.totalSamples, basename(options.binary));
  const unchanged = digest(options.binary) === before && digest(options.data) === dataSha256;
  const result = {
    schemaVersion: 1,
    metric: "software CPU self samples by ELF symbol",
    qualified:
      unchanged &&
      command.status === 0 &&
      !command.error &&
      parsed.qualifiedFormat &&
      census.qualified,
    binarySha256: before,
    dataSha256,
    unchanged,
    identity: options.identity,
    elfBuildId: /Build ID:\s*([0-9a-f]+)/iu.exec(notes.stdout)?.[1] ?? null,
    perfVersion: version.stdout.trim(),
    command: {
      executable: command.executable,
      args: command.args,
      status: command.status,
      error: command.error,
      wallMs: command.wallMs,
    },
    census,
    rows: parsed.rows,
    sourceLineQualified: false,
    hardwareInstructions: null,
    inclusiveCosts: null,
    scope:
      "exact recorded self events; no addr2line, source-line attribution, or instruction-count claim",
    postProcessingOutsideMeasuredWindow: true,
  };
  writeFileSync(join(options.out, "symbol-cpu.json"), JSON.stringify(result, null, 2) + "\n");
  return result;
}
