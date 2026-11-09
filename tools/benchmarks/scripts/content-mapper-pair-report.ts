/** Pure framing, statistics and Markdown for `content-mapper-pair.ts`. */

export type BurstSample = {
  wallMs: number;
  cpuSeconds?: number;
  peakMiB?: number;
  outputBytes: number;
  outputSha: string;
};

export type SerialSample = { latencies: number[]; outputSha: string };

export type BuildSummary = {
  id: string;
  burstWallMs: number;
  burstWallMin: number;
  burstWallMax: number;
  burstCpuSeconds?: number;
  burstPeakMiB?: number;
  serialP50Ms: number;
  serialP95Ms: number;
  serialTotalMs: number;
  outputBytes: number;
};

export type Cell = {
  label: string;
  sfcs: number;
  serialFiles: number;
  runs: number;
  builds: BuildSummary[];
  outputsIdentical: boolean;
  failure?: string;
};

/** One JSON-RPC message with its `Content-Length` header. */
export function frame(message: unknown): Buffer {
  const body = Buffer.from(JSON.stringify(message));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
}

export function median(values: readonly number[]): number {
  if (values.length === 0) return NaN;
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[middle] ?? NaN)
    : ((sorted[middle - 1] ?? NaN) + (sorted[middle] ?? NaN)) / 2;
}

export function quantile(values: readonly number[], q: number): number {
  if (values.length === 0) return NaN;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * q))] ?? NaN;
}

const defined = (values: (number | undefined)[]): number[] =>
  values.filter((value): value is number => value !== undefined && Number.isFinite(value));

export function summarizeCell(input: {
  label: string;
  sfcs: number;
  serialFiles: number;
  builds: string[];
  bursts: Map<string, BurstSample[]>;
  serials: Map<string, SerialSample[]>;
  allowOutputChange: boolean;
}): Cell {
  const shas = new Set<string>();
  const perBuild = new Map<string, Set<string>>();
  for (const id of input.builds) {
    const own = new Set<string>();
    for (const sample of input.bursts.get(id) ?? []) own.add(`burst:${sample.outputSha}`);
    for (const sample of input.serials.get(id) ?? []) own.add(`serial:${sample.outputSha}`);
    perBuild.set(id, own);
    for (const sha of own) shas.add(sha);
  }
  // Each build must answer every run identically; builds may differ only when allowed.
  const unstable = [...perBuild].find(([, own]) => own.size !== 2);
  const outputsIdentical = shas.size === 2;
  const failure = unstable
    ? `${unstable[0]} answered runs differently`
    : !outputsIdentical && !input.allowOutputChange
      ? "builds answered differently"
      : undefined;
  const builds = input.builds.map((id): BuildSummary => {
    const bursts = input.bursts.get(id) ?? [];
    const serials = input.serials.get(id) ?? [];
    const walls = bursts.map((sample) => sample.wallMs);
    const cpu = defined(bursts.map((sample) => sample.cpuSeconds));
    const peak = defined(bursts.map((sample) => sample.peakMiB));
    const latencies = serials.flatMap((sample) => sample.latencies);
    return {
      id,
      burstWallMs: median(walls),
      burstWallMin: Math.min(...walls),
      burstWallMax: Math.max(...walls),
      burstCpuSeconds: cpu.length ? median(cpu) : undefined,
      burstPeakMiB: peak.length ? median(peak) : undefined,
      serialP50Ms: quantile(latencies, 0.5),
      serialP95Ms: quantile(latencies, 0.95),
      serialTotalMs: median(serials.map((sample) => sample.latencies.reduce((a, b) => a + b, 0))),
      outputBytes: bursts[0]?.outputBytes ?? 0,
    };
  });
  return {
    label: input.label,
    sfcs: input.sfcs,
    serialFiles: input.serialFiles,
    runs: input.bursts.get(input.builds[0] ?? "")?.length ?? 0,
    builds,
    outputsIdentical,
    failure,
  };
}

const fixed = (value: number | undefined, digits: number): string =>
  value === undefined || !Number.isFinite(value) ? "n/a" : value.toFixed(digits);

/** Head relative to the first build, as a signed percentage. */
const delta = (value: number | undefined, base: number | undefined): string =>
  value === undefined || base === undefined || !Number.isFinite(value) || !base
    ? ""
    : ` (${value >= base ? "+" : ""}${((100 * (value - base)) / base).toFixed(1)}%)`;

export function markdown(label: string, cells: readonly Cell[]): string {
  const lines = [`### Content mapper pair: ${label}`, ""];
  for (const cell of cells) {
    lines.push(
      `**${cell.sfcs} SFCs**, ${cell.runs} alternating runs; serial: ${cell.serialFiles} transforms. ` +
        (cell.failure
          ? `**No timing published: ${cell.failure}.**`
          : cell.outputsIdentical
            ? "Responses byte-identical across builds and runs."
            : "Responses differ between builds (allowed)."),
      "",
    );
    if (cell.failure) continue;
    const first = cell.builds[0];
    lines.push(
      "| build | burst wall ms (min–max) | burst CPU s | peak MiB | serial p50 ms | serial p95 ms | serial total ms | response MB |",
      "|---|---|---|---|---|---|---|---|",
    );
    for (const build of cell.builds) {
      const base = build === first ? undefined : first;
      lines.push(
        `| ${build.id} | ${fixed(build.burstWallMs, 0)}${delta(build.burstWallMs, base?.burstWallMs)} (${fixed(build.burstWallMin, 0)}–${fixed(build.burstWallMax, 0)}) ` +
          `| ${fixed(build.burstCpuSeconds, 2)}${delta(build.burstCpuSeconds, base?.burstCpuSeconds)} ` +
          `| ${fixed(build.burstPeakMiB, 1)}${delta(build.burstPeakMiB, base?.burstPeakMiB)} ` +
          `| ${fixed(build.serialP50Ms, 2)}${delta(build.serialP50Ms, base?.serialP50Ms)} ` +
          `| ${fixed(build.serialP95Ms, 2)}${delta(build.serialP95Ms, base?.serialP95Ms)} ` +
          `| ${fixed(build.serialTotalMs, 0)}${delta(build.serialTotalMs, base?.serialTotalMs)} ` +
          `| ${fixed(build.outputBytes / 1e6, 2)} |`,
      );
    }
    lines.push("");
  }
  return `${lines.join("\n")}\n`;
}
