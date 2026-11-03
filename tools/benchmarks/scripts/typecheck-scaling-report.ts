/**
 * Medians, per-SFC cost and superlinearity verdicts of the type-check scaling
 * lane, and its Markdown summary. Pure: reads the results the driver wrote.
 *
 * A tool is flagged when its per-SFC CPU or peak RSS at the largest size
 * exceeds GROWTH_LIMIT × its value at the smallest size, or when the marginal
 * cost per added SFC between the two largest sizes exceeds GROWTH_LIMIT × the
 * marginal cost between the two smallest (a step in the curve).
 */

import { PLANT_MULTIPLICITY_NOTES } from "./typecheck-scaling-gate.ts";
import type { Provenance, ToolId } from "./typecheck-scaling-tools.ts";

export const GROWTH_LIMIT = 1.5;

export type Sample = {
  wallMs: number;
  cpuMs: number | null;
  maxRssKb: number | null;
  errors: number;
  warnings: number;
  fingerprint: string;
  types: number | null;
  instantiations: number | null;
  checkTimeS: number | null;
};
export type Cell = {
  tool: ToolId;
  label: string;
  vize: boolean;
  kind: "project" | "monorepo";
  sfcs: number;
  plants: number;
  mode: string;
  cold: Sample | null;
  runs: Sample[];
  refused: string | null;
};
export type ScalingResults = {
  schemaVersion: 1;
  sha: string;
  runner: string;
  machine: string;
  measured: boolean;
  runs: number;
  generatedAt: string;
  runUrl: string | null;
  nodeOptions: string;
  gnuTime: string | null;
  provenance: Provenance;
  cells: Cell[];
};

export function median(values: number[]): number {
  if (values.length === 0) throw new Error("median of no values");
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[mid] as number)
    : ((sorted[mid - 1] as number) + (sorted[mid] as number)) / 2;
}

const medianOf = (samples: Sample[], key: keyof Sample): number | null => {
  const values = samples.map((sample) => sample[key]).filter((v) => typeof v === "number");
  return values.length === samples.length && values.length > 0 ? median(values as number[]) : null;
};

export type Row = {
  cell: Cell;
  wallMs: number | null;
  cpuMs: number | null;
  rssKb: number | null;
  types: number | null;
  instantiations: number | null;
  checkTimeS: number | null;
};

/** Warm medians; the cold run never enters them. A refused cell has no numbers. */
export function rowOf(cell: Cell): Row {
  const runs = cell.refused ? [] : cell.runs;
  const of = (key: keyof Sample) => (runs.length > 0 ? medianOf(runs, key) : null);
  return {
    cell,
    wallMs: of("wallMs"),
    cpuMs: of("cpuMs"),
    rssKb: of("maxRssKb"),
    types: of("types"),
    instantiations: of("instantiations"),
    checkTimeS: of("checkTimeS"),
  };
}

type Metric = "wallMs" | "cpuMs" | "rssKb";
export type Growth = {
  tool: ToolId;
  label: string;
  vize: boolean;
  sizes: number[];
  /** Per-SFC value at the largest size over the smallest. */
  perSfc: Record<Metric, number | null>;
  /** Marginal cost per added SFC, last interval over the first; null with fewer than three sizes. */
  step: Record<Metric, number | null>;
  flags: string[];
};

const ratio = (a: number | null, b: number | null): number | null =>
  a == null || b == null || !(b > 0) ? null : a / b;

/** Per-tool growth over the single-project sizes that measured. */
export function growthOf(rows: Row[]): Growth[] {
  const tools = [...new Set(rows.map((row) => row.cell.tool))];
  return tools.flatMap((tool) => {
    const series = rows
      .filter((row) => row.cell.tool === tool && row.cell.kind === "project" && row.cpuMs != null)
      .sort((a, b) => a.cell.sfcs - b.cell.sfcs);
    const first = series[0];
    const last = series.at(-1);
    if (!first || !last || series.length < 2) return [];
    const perSfc = (metric: Metric) =>
      ratio(
        last[metric] == null ? null : (last[metric] as number) / last.cell.sfcs,
        first[metric] == null ? null : (first[metric] as number) / first.cell.sfcs,
      );
    const slope = (a: Row, b: Row, metric: Metric) =>
      a[metric] == null || b[metric] == null
        ? null
        : ((b[metric] as number) - (a[metric] as number)) / (b.cell.sfcs - a.cell.sfcs);
    const step = (metric: Metric) => {
      if (series.length < 3) return null;
      const [a, b] = series as [Row, Row];
      const [y, z] = series.slice(-2) as [Row, Row];
      return ratio(slope(y, z, metric), slope(a, b, metric));
    };
    const growth: Growth = {
      tool,
      label: first.cell.label,
      vize: first.cell.vize,
      sizes: series.map((row) => row.cell.sfcs),
      perSfc: { wallMs: perSfc("wallMs"), cpuMs: perSfc("cpuMs"), rssKb: perSfc("rssKb") },
      step: { wallMs: step("wallMs"), cpuMs: step("cpuMs"), rssKb: step("rssKb") },
      flags: [],
    };
    for (const [metric, name] of [
      ["cpuMs", "CPU"],
      ["rssKb", "RSS"],
    ] as const) {
      const value = growth.perSfc[metric];
      if (value != null && value > GROWTH_LIMIT) {
        growth.flags.push(
          `per-SFC ${name} ×${value.toFixed(2)} from ${first.cell.sfcs} to ${last.cell.sfcs}`,
        );
      }
      const jump = growth.step[metric];
      if (jump != null && jump > GROWTH_LIMIT) {
        growth.flags.push(`marginal ${name} per SFC steps ×${jump.toFixed(2)}`);
      }
    }
    return [growth];
  });
}

const fixed = (value: number | null, digits: number, scale = 1) =>
  value == null ? "n/a" : (value / scale).toFixed(digits);
const count = (value: number | null) =>
  value == null ? "n/a" : Math.round(value).toLocaleString("en-US");
const times = (value: number | null) => (value == null ? "n/a" : `×${value.toFixed(2)}`);

export function renderSummary(results: ScalingResults): string {
  const rows = results.cells.map(rowOf);
  const growth = growthOf(rows);
  const refused = results.cells.filter((cell) => cell.refused);
  const lines = [
    "# Type-check scaling",
    "",
    `Commit \`${results.sha}\` on \`${results.runner}\` (${results.machine}), ${results.generatedAt}. Median of ${results.runs} warm run(s); the first (cold) run is listed separately and never enters a median.${results.runUrl ? ` [Run](${results.runUrl}).` : ""}`,
    "",
    "CPU is user+sys of the whole process tree (reaped children's cutime+cstime from /proc/self/stat around each spawn). Peak RSS is GNU time `%M`, the largest single process of the tree. Every timed run reported exactly the planted errors; each run starts with an empty cache directory (`XDG_CACHE_HOME`), so no checker replays an earlier run's results, and build-mode runs start from deleted outputs with `--force`.",
    "",
  ];
  for (const note of PLANT_MULTIPLICITY_NOTES) lines.push(`- ${note}`);
  if (PLANT_MULTIPLICITY_NOTES.length > 0) lines.push("");
  if (!results.measured) lines.push("**Smoke run: not on Linux, so nothing was timed.**", "");
  if (refused.length > 0) {
    lines.push(
      `## Refused (${refused.length})`,
      "",
      "These cells did not report exactly the planted errors, so they have no timing.",
      "",
    );
    for (const cell of refused) {
      lines.push(
        `- **${cell.label}**, ${cell.kind} ${cell.sfcs} SFCs:`,
        "",
        "```text",
        cell.refused ?? "",
        "```",
        "",
      );
    }
  }
  const header = (kind: string) => [
    `| Tool | ${kind === "monorepo" ? "Mode" : "SFCs"} | Wall s | CPU s | Peak RSS MiB | Wall ms/SFC | CPU ms/SFC | RSS KiB/SFC | Types | Instantiations | Check s | Cold wall s | Cold CPU s |`,
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  ];
  const line = (row: Row) => {
    const { cell } = row;
    const per = (value: number | null, digits: number) =>
      value == null ? "n/a" : (value / cell.sfcs).toFixed(digits);
    return `| ${cell.label} | ${cell.kind === "monorepo" ? `${cell.mode}, ${cell.sfcs}` : cell.sfcs} | ${fixed(row.wallMs, 2, 1000)} | ${fixed(row.cpuMs, 2, 1000)} | ${fixed(row.rssKb, 0, 1024)} | ${per(row.wallMs, 3)} | ${per(row.cpuMs, 3)} | ${per(row.rssKb, 1)} | ${count(row.types)} | ${count(row.instantiations)} | ${fixed(row.checkTimeS, 2)} | ${cell.refused ? "refused" : fixed(cell.cold?.wallMs ?? null, 2, 1000)} | ${cell.refused ? "refused" : fixed(cell.cold?.cpuMs ?? null, 2, 1000)} |`;
  };
  const project = rows.filter((row) => row.cell.kind === "project");
  if (project.length > 0) {
    lines.push("## Single project", "", ...header("project"));
    for (const row of [...project].sort(
      (a, b) => a.cell.tool.localeCompare(b.cell.tool) || a.cell.sfcs - b.cell.sfcs,
    )) {
      lines.push(line(row));
    }
    lines.push("");
  }
  const monorepo = rows.filter((row) => row.cell.kind === "monorepo");
  if (monorepo.length > 0) {
    lines.push(
      "## Project-references monorepo",
      "",
      ...header("monorepo"),
      ...monorepo.map(line),
      "",
    );
  }
  if (growth.length > 0) {
    lines.push(
      "## Growth",
      "",
      `Per-SFC cost at the largest size over the smallest, and the marginal cost per added SFC of the last interval over the first. Above ×${GROWTH_LIMIT} is flagged.`,
      "",
      "| Tool | Sizes | CPU/SFC | RSS/SFC | Wall/SFC | CPU step | RSS step | Verdict |",
      "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |",
      ...growth.map(
        (g) =>
          `| ${g.label} | ${g.sizes.join(", ")} | ${times(g.perSfc.cpuMs)} | ${times(g.perSfc.rssKb)} | ${times(g.perSfc.wallMs)} | ${times(g.step.cpuMs)} | ${times(g.step.rssKb)} | ${g.flags.length > 0 ? `**superlinear**: ${g.flags.join("; ")}` : "linear"} |`,
      ),
      "",
    );
  }
  lines.push(
    "<details><summary>Tool versions and binary checksums</summary>",
    "",
    "| Tool | Version | Binary SHA-256 |",
    "| --- | --- | --- |",
    ...Object.entries(results.provenance).map(
      ([name, entry]) =>
        `| ${name} | ${entry.version ?? "n/a"} | ${entry.sha256 ? `\`${entry.sha256}\`` : "n/a"} |`,
    ),
    "",
    `NODE_OPTIONS=\`${results.nodeOptions}\`; ${results.gnuTime ?? "no GNU time"}.`,
    "",
    "</details>",
    "",
  );
  return `${lines.join("\n")}\n`;
}
