/**
 * Size curves for `lsp-vs-volar.ts`: fixtures that differ only in unrelated
 * project size (same edited SFC, anchors and expectations), each metric per
 * editor against SFC count, and a least-squares slope per 1k SFCs. Per-edit
 * work must not grow with the project, so a per-edit slope that is clearly
 * above zero is flagged. Pure over session results.
 */

import type { FixtureSpec } from "./lsp-vs-volar-fixtures.ts";
import { type EditorSummary, type Latency, summarize } from "./lsp-vs-volar-report.ts";
import type { SessionResult } from "./lsp-vs-volar-session.ts";

/** One benchmark job per fixture, except that a curve's fixtures share one machine. */
export function planJobs(specs: FixtureSpec[]): Array<{ name: string; ids: string }> {
  const jobs = new Map<string, string[]>();
  for (const spec of specs) {
    const name = spec.curve?.name ?? spec.id;
    jobs.set(name, [...(jobs.get(name) ?? []), spec.id]);
  }
  return [...jobs].map(([name, ids]) => ({ name, ids: ids.join(" ") }));
}

export interface Fit {
  slope: number;
  intercept: number;
  /** Half-width of the slope's 95% interval; `null` without a residual degree of freedom. */
  halfWidth: number | null;
  n: number;
}

/** Two-sided 95% Student t quantiles for 1-30 degrees of freedom. */
const T95 = [
  12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.16, 2.145,
  2.131, 2.12, 2.11, 2.101, 2.093, 2.086, 2.08, 2.074, 2.069, 2.064, 2.06, 2.056, 2.052, 2.048,
  2.045, 2.042,
];

/** Ordinary least squares of `y` on `x`; `null` unless `x` takes two values. */
export function fitLine(points: ReadonlyArray<readonly [number, number]>): Fit | null {
  const n = points.length;
  if (n < 2) return null;
  const meanX = points.reduce((sum, [x]) => sum + x, 0) / n;
  const meanY = points.reduce((sum, [, y]) => sum + y, 0) / n;
  let sxx = 0;
  let sxy = 0;
  for (const [x, y] of points) {
    sxx += (x - meanX) ** 2;
    sxy += (x - meanX) * (y - meanY);
  }
  if (sxx === 0) return null;
  const slope = sxy / sxx;
  const intercept = meanY - slope * meanX;
  const residuals = points.reduce((sum, [x, y]) => sum + (y - intercept - slope * x) ** 2, 0);
  const df = n - 2;
  const halfWidth = df < 1 ? null : (T95[df - 1] ?? 2) * Math.sqrt(residuals / df / sxx);
  return { slope, intercept, halfWidth, n };
}

type Unit = "ms" | "s" | "MiB";

interface Metric {
  label: string;
  unit: Unit;
  /** Must not grow with project size. */
  perEdit: boolean;
  value: (summary: EditorSummary) => number | null;
  /** Asked and answered wrong or not at all, every time. */
  failed?: (summary: EditorSummary) => boolean;
}

function latencies(label: string, pick: (summary: EditorSummary) => Latency): Metric[] {
  const failed = (summary: EditorSummary) => pick(summary).n > 0 && pick(summary).ok === 0;
  return [
    { label: `${label} p50 ms`, unit: "ms", perEdit: true, value: (s) => pick(s).p50, failed },
    { label: `${label} p99 ms`, unit: "ms", perEdit: true, value: (s) => pick(s).p99, failed },
  ];
}

export const CURVE_METRICS: Metric[] = [
  { label: "cold start ms", unit: "ms", perEdit: false, value: (s) => s.cold.totalMs },
  { label: "session CPU s", unit: "s", perEdit: false, value: (s) => s.warm.cpuSeconds },
  { label: "peak PSS MiB", unit: "MiB", perEdit: false, value: (s) => s.warm.peakPssMiB },
  { label: "CPU s per edit", unit: "s", perEdit: true, value: (s) => s.warm.cpuSecondsPerEdit },
  ...latencies("diagnostics after script edit", (s) => s.diagnostics.script),
  ...latencies("diagnostics after template edit", (s) => s.diagnostics.template),
  ...latencies("hover", (s) => s.requests.hover),
  ...latencies("completion", (s) => s.requests.completion),
  ...latencies("definition", (s) => s.requests.definition),
];

/** Growth below these is under the clock's and the sampler's resolution. */
const FLOOR: Record<Unit, number> = { ms: 1, s: 0.005, MiB: 1 };
/** Growth across the measured sizes below this share of the smallest size's value is noise. */
const RELATIVE = 0.05;

export interface MetricCurve {
  label: string;
  unit: Unit;
  perEdit: boolean;
  /** The fixture sections' values, one per size. */
  values: Array<number | null>;
  /** Fit over every session, x in thousands of SFCs. */
  fit: Fit | null;
  /** Why this per-edit metric grows with project size, or `null`. */
  grows: string | null;
}

export interface CurvePoint {
  id: string;
  sfcs: number;
  sessions: SessionResult[];
}

/** The verdict for one per-edit metric, from its fit and the sizes where it answered. */
export function growth(
  metric: Pick<Metric, "unit" | "perEdit">,
  fit: Fit | null,
  thousands: number[],
  failedAt: boolean[],
  values: Array<number | null>,
): string | null {
  if (!metric.perEdit) return null;
  const answered = values.findIndex((value) => value != null);
  const stops = failedAt.findIndex((failed, index) => failed && answered >= 0 && index > answered);
  if (stops >= 0) return `stops answering at ${thousands[stops]}k SFCs`;
  if (!fit || fit.halfWidth == null || fit.slope - fit.halfWidth <= 0) return null;
  const low = Math.min(...thousands);
  const rise = fit.slope * (Math.max(...thousands) - low);
  const base = Math.abs(fit.intercept + fit.slope * low);
  return rise >= Math.max(FLOOR[metric.unit], RELATIVE * base) ? "slope above zero" : null;
}

export interface EditorCurve {
  editor: string;
  metrics: MetricCurve[];
}

/** Every editor's metrics across the curve's sizes, ascending. */
export function sizeCurve(points: CurvePoint[]): { sfcs: number[]; editors: EditorCurve[] } {
  const sorted = [...points].sort((left, right) => left.sfcs - right.sfcs);
  const thousands = sorted.map((point) => point.sfcs / 1000);
  const labels = [...new Set(sorted.flatMap((point) => point.sessions.map((s) => s.editor)))];
  const editors = labels.map((editor) => {
    const mine = sorted.map((point) => point.sessions.filter((s) => s.editor === editor));
    const pooled = mine.map((sessions) => (sessions.length ? summarize(sessions)[0] : null));
    const single = mine.map((sessions) => sessions.map((session) => summarize([session])[0]));
    const metrics = CURVE_METRICS.map((metric): MetricCurve => {
      const values = pooled.map((summary) => (summary ? metric.value(summary) : null));
      const xy = single.flatMap((summaries, index) =>
        summaries
          .map((summary) => metric.value(summary))
          .filter((value): value is number => value != null)
          .map((value) => [thousands[index], value] as const),
      );
      const fit = fitLine(xy);
      const failedAt = pooled.map((summary) =>
        summary ? (metric.failed?.(summary) ?? false) : false,
      );
      const { label, unit, perEdit } = metric;
      return {
        label,
        unit,
        perEdit,
        values,
        fit,
        grows: growth(metric, fit, thousands, failedAt, values),
      };
    });
    return { editor, metrics };
  });
  return { sfcs: sorted.map((point) => point.sfcs), editors };
}

function number(value: number | null, unit: Unit): string {
  if (value == null) return "–";
  if (unit === "s") return value.toFixed(Math.abs(value) < 1 ? 3 : 2);
  return Math.abs(value) < 10 ? value.toFixed(1) : value.toFixed(0);
}

function slope(fit: Fit | null, unit: Unit): string {
  if (!fit) return "–";
  const sign = fit.slope >= 0 ? "+" : "−";
  const width = fit.halfWidth == null ? "" : ` ± ${number(fit.halfWidth, unit)}`;
  return `${sign}${number(Math.abs(fit.slope), unit)}${width} ${unit}`;
}

export function curveMarkdown(
  name: string,
  ids: string[],
  curve: ReturnType<typeof sizeCurve>,
): string {
  const sizes = curve.sfcs.map((sfcs) => `${sfcs.toLocaleString("en-US")} SFCs`);
  const lines = [
    `## Size curve: ${name}`,
    "",
    `Fixtures ${ids.join(", ")}: the same edited SFC, anchors and expectations at every size; only unrelated project size grows.`,
    "Cells are the fixture sections' values. The slope is a least-squares fit over every session, per 1k SFCs, ± its 95% interval.",
    `A per-edit row is flagged when its slope is above zero beyond that interval and the fitted rise across the sizes is at least ${RELATIVE * 100}% of the smallest size's value (and at least ${FLOOR.ms} ms or ${FLOOR.s} CPU s), or when it stops answering at a larger size.`,
    "",
  ];
  const flagged: string[] = [];
  for (const editor of curve.editors) {
    lines.push(`### ${editor.editor}`, "");
    lines.push(`| metric | ${sizes.join(" | ")} | slope per 1k SFCs | per edit |`);
    lines.push(`| --- | ${sizes.map(() => "---:").join(" | ")} | ---: | --- |`);
    for (const metric of editor.metrics) {
      const verdict = !metric.perEdit ? "" : metric.grows ? `**grows** (${metric.grows})` : "flat";
      const cells = metric.values.map((value) => number(value, metric.unit));
      lines.push(
        `| ${metric.label} | ${cells.join(" | ")} | ${slope(metric.fit, metric.unit)} | ${verdict} |`,
      );
      if (metric.grows)
        flagged.push(
          `- ${editor.editor}: ${metric.label}, ${slope(metric.fit, metric.unit)} per 1k SFCs (${metric.grows})`,
        );
    }
    lines.push("");
  }
  lines.push(
    flagged.length
      ? "Per-edit work that grows with project size:"
      : "No per-edit metric grows with project size.",
    "",
  );
  if (flagged.length) lines.push(...flagged, "");
  return lines.join("\n");
}

/** Each curve with at least two of `specs` on it. */
export function curvesOf(specs: FixtureSpec[], sessions: (id: string) => SessionResult[]) {
  const groups = new Map<string, FixtureSpec[]>();
  for (const spec of specs) {
    if (spec.curve) groups.set(spec.curve.name, [...(groups.get(spec.curve.name) ?? []), spec]);
  }
  return [...groups]
    .filter(([, members]) => members.length >= 2)
    .map(([name, members]) => ({
      name,
      ids: members.map((member) => member.id),
      curve: sizeCurve(
        members.map((member) => ({
          id: member.id,
          sfcs: member.curve!.sfcs,
          sessions: sessions(member.id),
        })),
      ),
    }));
}
