/** Pure framing, statistics and Markdown for `content-mapper-pair.ts`. */

export type BurstSample = {
  wallMs: number;
  /** JSON-RPC error replies in the burst. */
  errors?: number;
  cpuSeconds?: number;
  peakMiB?: number;
  outputBytes: number;
  outputSha: string;
};

export type SerialSample = { latencies: number[]; outputSha: string };

export type BuildSummary = {
  id: string;
  /** Timed beside the builds; its answers are not compared. */
  reference?: boolean;
  errors?: number;
  burstWallMs: number;
  burstWallMin: number;
  burstWallMax: number;
  burstCpuSeconds?: number;
  burstPeakMiB?: number;
  serialP50Ms: number;
  serialP95Ms: number;
  serialSessionP50Ms: number;
  serialSessionP95Ms: number;
  serialTotalMs: number;
  serialSessions: number;
  serialRawSamples: number;
  outputBytes: number;
};

export type Cell = {
  label: string;
  sfcs: number;
  serialFiles: number;
  replicates: number;
  orderSeed: number;
  ordersPerReplicate: number;
  burstSamplesPerBuild: number;
  builds: BuildSummary[];
  outputsIdentical: boolean;
  failure?: string;
};

/** Fixed across hosts so every replicate gets the same deterministic AB/BA order. */
export const PAIR_ORDER_SEED = 0x13018;

/**
 * Two-build comparisons use AB+BA. Larger comparisons add every rotation and
 * its inverse so each build occupies every position and relative order equally.
 */
export function balancedOrders<T>(
  values: readonly T[],
  replicate: number,
  seed = PAIR_ORDER_SEED,
): T[][] {
  let mixed = (seed + Math.imul(replicate + 1, 0x9e3779b9)) >>> 0;
  mixed ^= mixed >>> 16;
  mixed = Math.imul(mixed, 0x21f0aaad) >>> 0;
  mixed ^= mixed >>> 15;
  const first = (mixed & 1) === 0 ? [...values] : [...values].reverse();
  if (first.length <= 2) return [first, [...first].reverse()];
  const rotations = (order: readonly T[]) =>
    order.map((_, offset) => [...order.slice(offset), ...order.slice(0, offset)]);
  return [...rotations(first), ...rotations([...first].reverse())];
}

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
  replicates: number;
  orderSeed?: number;
  builds: string[];
  references?: string[];
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
  const references = input.references ?? [];
  const builds = [...input.builds, ...references].map((id): BuildSummary => {
    const bursts = input.bursts.get(id) ?? [];
    const serials = input.serials.get(id) ?? [];
    const walls = bursts.map((sample) => sample.wallMs);
    const cpu = defined(bursts.map((sample) => sample.cpuSeconds));
    const peak = defined(bursts.map((sample) => sample.peakMiB));
    const latencies = serials.flatMap((sample) => sample.latencies);
    const sessionP50s = serials.map((sample) => quantile(sample.latencies, 0.5));
    const sessionP95s = serials.map((sample) => quantile(sample.latencies, 0.95));
    return {
      id,
      ...(references.includes(id)
        ? { reference: true, errors: Math.max(0, ...bursts.map((sample) => sample.errors ?? 0)) }
        : {}),
      burstWallMs: median(walls),
      burstWallMin: Math.min(...walls),
      burstWallMax: Math.max(...walls),
      burstCpuSeconds: cpu.length ? median(cpu) : undefined,
      burstPeakMiB: peak.length ? median(peak) : undefined,
      serialP50Ms: quantile(latencies, 0.5),
      serialP95Ms: quantile(latencies, 0.95),
      serialSessionP50Ms: median(sessionP50s),
      serialSessionP95Ms: median(sessionP95s),
      serialTotalMs: median(serials.map((sample) => sample.latencies.reduce((a, b) => a + b, 0))),
      serialSessions: serials.length,
      serialRawSamples: latencies.length,
      outputBytes: bursts[0]?.outputBytes ?? 0,
    };
  });
  return {
    label: input.label,
    sfcs: input.sfcs,
    serialFiles: input.serialFiles,
    replicates: input.replicates,
    orderSeed: input.orderSeed ?? PAIR_ORDER_SEED,
    ordersPerReplicate:
      (input.bursts.get(input.builds[0] ?? "")?.length ?? 0) / input.replicates,
    burstSamplesPerBuild: input.bursts.get(input.builds[0] ?? "")?.length ?? 0,
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
    const schedule =
      cell.ordersPerReplicate === 2
        ? "AB+BA"
        : `${cell.ordersPerReplicate} position-balanced orders`;
    lines.push(
      `**${cell.sfcs} SFCs**, ${cell.replicates} balanced replicates ` +
        `(${schedule}, seed 0x${cell.orderSeed.toString(16)}): ` +
        `${cell.burstSamplesPerBuild} burst samples/build; ` +
        `${cell.builds[0]?.serialSessions ?? 0} serial sessions/build × ${cell.serialFiles} transforms ` +
        `= ${cell.builds[0]?.serialRawSamples ?? 0} raw latency samples/build. ` +
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
      "| build | burst wall ms (min–max) | burst CPU s | peak MiB | serial pooled p50/p95 ms | per-session median p50/p95 ms | session total median ms | response MB |",
      "|---|---|---|---|---|---|---|---|",
    );
    for (const build of cell.builds) {
      const base = build === first ? undefined : first;
      lines.push(
        `| ${build.reference ? `${build.id} (reference${build.errors ? `, ${build.errors} errors` : ""})` : build.id} | ${fixed(build.burstWallMs, 0)}${delta(build.burstWallMs, base?.burstWallMs)} (${fixed(build.burstWallMin, 0)}–${fixed(build.burstWallMax, 0)}) ` +
          `| ${fixed(build.burstCpuSeconds, 2)}${delta(build.burstCpuSeconds, base?.burstCpuSeconds)} ` +
          `| ${fixed(build.burstPeakMiB, 1)}${delta(build.burstPeakMiB, base?.burstPeakMiB)} ` +
          `| ${fixed(build.serialP50Ms, 2)}${delta(build.serialP50Ms, base?.serialP50Ms)} / ${fixed(build.serialP95Ms, 2)}${delta(build.serialP95Ms, base?.serialP95Ms)} ` +
          `| ${fixed(build.serialSessionP50Ms, 2)}${delta(build.serialSessionP50Ms, base?.serialSessionP50Ms)} / ${fixed(build.serialSessionP95Ms, 2)}${delta(build.serialSessionP95Ms, base?.serialSessionP95Ms)} ` +
          `| ${fixed(build.serialTotalMs, 0)}${delta(build.serialTotalMs, base?.serialTotalMs)} ` +
          `| ${fixed(build.outputBytes / 1e6, 2)} |`,
      );
    }
    lines.push("");
  }
  return `${lines.join("\n")}\n`;
}
