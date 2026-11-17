/**
 * Report schema and Markdown summary for
 * `tools/support/github/rust-frontend-threads-bench.ts`.
 */

export interface Sample {
  readonly scenario: string;
  readonly measure: string;
  readonly config: string;
  readonly round: number;
  readonly wallSeconds: number;
  readonly cpuSeconds: number;
  readonly maxRssMiB: number;
  readonly status: number;
  readonly sha256?: string;
}

export interface Report {
  readonly host: { cpus: number; memoryGiB: number; platform: string; loadAverage: number[] };
  readonly samples: Sample[];
  readonly determinism?: Record<string, unknown>;
}

/** Nearest-rank percentile; with three samples p95 is the slowest run. */
export function percentile(values: readonly number[], fraction: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.max(0, Math.ceil(fraction * sorted.length) - 1)]!;
}

const order = (config: string) =>
  config === "stable" ? -1 : Number(config.replace("nightly-j", ""));

const delta = (value: number, base: number | undefined) =>
  base === undefined ? "" : `${(((value - base) / base) * 100).toFixed(1)}%`;

/** One row per measure and configuration, ordered stable, j1, j2, ... */
export function summarize(reports: readonly Report[]): string {
  const groups = new Map<string, Map<string, Sample[]>>();
  for (const sample of reports.flatMap((report) => report.samples)) {
    if (sample.status !== 0) continue;
    const configs = groups.get(sample.measure) ?? new Map<string, Sample[]>();
    configs.set(sample.config, [...(configs.get(sample.config) ?? []), sample]);
    groups.set(sample.measure, configs);
  }
  const lines = [
    "| Measure | Config | n | p50 wall s | p95 wall s | vs stable | vs j1 | CPU s (p50) | peak RSS MiB |",
    "| --- | --- | --: | --: | --: | --: | --: | --: | --: |",
  ];
  for (const [measure, configs] of groups) {
    const p50 = (config: string) => {
      const group = configs.get(config);
      return (
        group &&
        percentile(
          group.map((sample) => sample.wallSeconds),
          0.5,
        )
      );
    };
    for (const config of [...configs.keys()].sort((left, right) => order(left) - order(right))) {
      const group = configs.get(config)!;
      const wall = group.map((sample) => sample.wallSeconds);
      const median = percentile(wall, 0.5);
      const cpu = percentile(
        group.map((sample) => sample.cpuSeconds),
        0.5,
      );
      const rss = Math.max(...group.map((sample) => sample.maxRssMiB));
      const vsStable = config === "stable" ? "" : delta(median, p50("stable"));
      const vsSerial =
        config === "stable" || config === "nightly-j1" ? "" : delta(median, p50("nightly-j1"));
      lines.push(
        `| ${measure} | ${config} | ${group.length} | ${median.toFixed(1)} | ${percentile(wall, 0.95).toFixed(1)} | ${vsStable} | ${vsSerial} | ${cpu.toFixed(1)} | ${rss.toFixed(0)} |`,
      );
    }
  }
  const hashes = reports
    .flatMap((report) => report.samples)
    .filter((sample) => sample.sha256 !== undefined)
    .map(
      (sample) =>
        `- ${sample.measure} ${sample.config}#${sample.round}: \`${sample.sha256!.slice(0, 16)}\``,
    );
  const determinism = reports.flatMap((report) => (report.determinism ? [report.determinism] : []));
  const hosts = reports.map(
    ({ host }) =>
      `${host.platform} ${host.cpus} CPUs ${host.memoryGiB.toFixed(0)} GiB load ${host.loadAverage.map((load) => load.toFixed(1)).join("/")}`,
  );
  return [
    `Hosts: ${[...new Set(hosts)].join("; ")}`,
    "",
    ...lines,
    "",
    ...(hashes.length > 0 ? ["Binary hashes:", ...hashes, ""] : []),
    ...(determinism.length > 0
      ? ["Determinism:", "```json", JSON.stringify(determinism, null, 2), "```", ""]
      : []),
  ].join("\n");
}
