/**
 * Markdown for the type-check profile: per corpus and lane, where the CPU of
 * the whole process tree went (role, DSO, phase x cost kind, functions,
 * syscalls), perf stat counters, tsgo's pprof view and, with a base, the
 * function-level diff. Pure functions over the driver's lane results.
 */

import type { FunctionCost, Profile } from "./typecheck-profile-fold.ts";
import type { PprofTop } from "./typecheck-profile-pprof.ts";

export type Counter = { value: number | null; unit: string };

export type LaneResult = {
  corpus: string;
  lane: string;
  label: string;
  side: "base" | "head";
  sha: string;
  callGraph: string;
  frequency: number;
  wallMs: number;
  diagnostics: number;
  counters: Record<string, Counter>;
  hostStats: Array<[string, string]>;
  profile: Profile;
  pprof: PprofTop | null;
  notes: string[];
};

const CLOCK_EVENTS = /^(?:cpu-clock|task-clock)/u;

/** Milliseconds of one profile's period: exact for clock events, scaled by task-clock otherwise. */
export function toMs(result: LaneResult, period: number): number {
  if (CLOCK_EVENTS.test(result.profile.event)) return period / 1e6;
  const taskClock = result.counters["task-clock"]?.value ?? null;
  if (taskClock === null || result.profile.period === 0) return Number.NaN;
  return (period / result.profile.period) * taskClock;
}

const ms = (value: number) =>
  Number.isNaN(value)
    ? "n/a"
    : value >= 1000
      ? `${(value / 1000).toFixed(2)} s`
      : `${value.toFixed(0)} ms`;
const pct = (part: number, whole: number) =>
  whole > 0 ? `${((part / whole) * 100).toFixed(1)}%` : "n/a";
const delta = (base: number, head: number) => {
  if (!Number.isFinite(base) || !Number.isFinite(head)) return "n/a";
  const diff = head - base;
  const rel = base !== 0 ? ` (${diff >= 0 ? "+" : ""}${((diff / base) * 100).toFixed(1)}%)` : "";
  return `${diff >= 0 ? "+" : "-"}${ms(Math.abs(diff))}${rel}`;
};
const cell = (text: string) => text.replaceAll("|", "\\|").replaceAll("`", "'");
const code = (text: string, max = 110) =>
  `\`${cell(text.length > max ? `${text.slice(0, max - 1)}…` : text)}\``;

type Keyed = Record<string, number>;

/** Rows of `key | [base] | head | [delta] | head share`, largest head first. */
function keyedTable(
  title: string,
  head: LaneResult,
  headValues: Keyed,
  base: LaneResult | null,
  baseValues: Keyed | null,
  limit = 20,
): string[] {
  const keys = [...new Set([...Object.keys(headValues), ...Object.keys(baseValues ?? {})])]
    .sort((a, b) => (headValues[b] ?? 0) - (headValues[a] ?? 0))
    .slice(0, limit);
  const total = head.profile.period;
  const rows = keys.map((key) => {
    const h = toMs(head, headValues[key] ?? 0);
    const share = pct(headValues[key] ?? 0, total);
    if (!base || !baseValues) return `| ${cell(key)} | ${ms(h)} | ${share} |`;
    const b = toMs(base, baseValues[key] ?? 0);
    return `| ${cell(key)} | ${ms(b)} | ${ms(h)} | ${delta(b, h)} | ${share} |`;
  });
  const header = base
    ? [`| ${title} | base | head | Δ | head share |`, "| --- | ---: | ---: | ---: | ---: |"]
    : [`| ${title} | CPU | share |`, "| --- | ---: | ---: |"];
  return [...header, ...rows, ""];
}

const functionKey = (cost: FunctionCost) => `${cost.role}\t${cost.dso}\t${cost.name}`;

function functionTable(
  head: LaneResult,
  base: LaneResult | null,
  by: "self" | "total",
  limit: number,
): string[] {
  const baseCosts = new Map(base?.profile.functions.map((cost) => [functionKey(cost), cost]));
  const total = head.profile.period;
  const rows = [...head.profile.functions]
    .sort((a, b) => b[by] - a[by])
    .slice(0, limit)
    .map((cost, index) => {
      const before = baseCosts.get(functionKey(cost));
      const change = base ? ` | ${delta(toMs(base, before?.[by] ?? 0), toMs(head, cost[by]))}` : "";
      return `| ${index + 1} | ${code(cost.name)} | ${cell(cost.dso)} | ${cell(cost.role)} | ${pct(cost.self, total)} | ${pct(cost.total, total)}${change} |`;
    });
  return [
    `| # | Function | DSO | Process | Self | Inclusive${base ? ` | Δ ${by}` : ""} |`,
    `| ---: | --- | --- | --- | ---: | ---:${base ? " | ---:" : ""} |`,
    ...rows,
    "",
  ];
}

/** Functions whose self (or inclusive) CPU moved most between base and head. */
export function functionDiff(
  base: LaneResult,
  head: LaneResult,
  by: "self" | "total",
  limit = 15,
): Array<{ cost: FunctionCost; baseMs: number; headMs: number }> {
  const keys = new Map<string, { base?: FunctionCost; head?: FunctionCost }>();
  for (const cost of base.profile.functions) keys.set(functionKey(cost), { base: cost });
  for (const cost of head.profile.functions) {
    keys.set(functionKey(cost), { ...keys.get(functionKey(cost)), head: cost });
  }
  return [...keys.values()]
    .map((pair) => ({
      cost: (pair.head ?? pair.base)!,
      baseMs: toMs(base, pair.base?.[by] ?? 0),
      headMs: toMs(head, pair.head?.[by] ?? 0),
    }))
    .sort((a, b) => Math.abs(b.headMs - b.baseMs) - Math.abs(a.headMs - a.baseMs))
    .slice(0, limit);
}

function phaseRows(result: LaneResult): Keyed {
  const out: Keyed = {};
  for (const phases of Object.values(result.profile.phases)) {
    for (const [phase, kinds] of Object.entries(phases)) {
      out[phase] = (out[phase] ?? 0) + Object.values(kinds).reduce((a, b) => a + b, 0);
    }
  }
  return out;
}

/** Phase rows with the three largest cost kinds inside each. */
function phaseKindTable(result: LaneResult, limit = 25): string[] {
  const total = result.profile.period;
  const rows: Array<[string, number, string]> = [];
  for (const phases of Object.values(result.profile.phases)) {
    for (const [phase, kinds] of Object.entries(phases)) {
      const sum = Object.values(kinds).reduce((a, b) => a + b, 0);
      const top = Object.entries(kinds)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 3)
        .map(([kind, value]) => `${kind} ${pct(value, sum)}`)
        .join(", ");
      rows.push([phase, sum, top]);
    }
  }
  return [
    "| Phase | CPU | share | largest cost kinds inside |",
    "| --- | ---: | ---: | --- |",
    ...rows
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit)
      .map(
        ([phase, sum, top]) =>
          `| ${cell(phase)} | ${ms(toMs(result, sum))} | ${pct(sum, total)} | ${cell(top)} |`,
      ),
    "",
  ];
}

const COUNTERS = [
  "task-clock",
  "duration_time",
  "user_time",
  "system_time",
  "cycles",
  "instructions",
  "branch-misses",
  "cache-references",
  "cache-misses",
  "L1-dcache-load-misses",
  "dTLB-load-misses",
  "context-switches",
  "cpu-migrations",
  "page-faults",
  "major-faults",
];

const counterText = (counter: Counter | undefined) =>
  !counter || counter.value === null
    ? "not counted"
    : counter.unit === "msec"
      ? ms(counter.value)
      : counter.value >= 1e6
        ? `${(counter.value / 1e9).toFixed(3)}G`
        : String(counter.value);

function counterTable(head: LaneResult, base: LaneResult | null): string[] {
  const names = COUNTERS.filter((name) => name in head.counters);
  const rows = names.map((name) => {
    const h = head.counters[name];
    if (!base) return `| ${name} | ${counterText(h)} |`;
    const b = base.counters[name];
    const change =
      h?.value != null && b?.value != null && b.value !== 0
        ? `${(((h.value - b.value) / b.value) * 100).toFixed(1)}%`
        : "n/a";
    return `| ${name} | ${counterText(b)} | ${counterText(h)} | ${change} |`;
  });
  const ipc = (r: LaneResult) => {
    const cycles = r.counters.cycles?.value;
    const instructions = r.counters.instructions?.value;
    return cycles && instructions ? (instructions / cycles).toFixed(2) : "n/a";
  };
  if (head.counters.cycles?.value) {
    rows.push(base ? `| IPC | ${ipc(base)} | ${ipc(head)} | |` : `| IPC | ${ipc(head)} |`);
  }
  // Some runner VMs accept hardware events but never count them.
  const noPmu = head.counters.cycles?.value === 0;
  return [
    base ? "| perf stat | base | head | Δ |" : "| perf stat | head |",
    base ? "| --- | ---: | ---: | ---: |" : "| --- | ---: |",
    ...rows,
    "",
    ...(noPmu ? ["> Hardware counters read 0: this runner does not expose its PMU.", ""] : []),
  ];
}

/**
 * Busy cores per process role over the run, in at most `slices` rows, with
 * the phase that held the most cores in each: where the mapper overlaps the
 * host's parse, and where GC or one checker runs alone.
 */
export function timelineTable(result: LaneResult, slices = 16): string[] {
  const { timeline, event } = result.profile;
  if (!timeline || !CLOCK_EVENTS.test(event)) return [];
  const length = Math.max(0, ...Object.values(timeline.roles).map((values) => values.length));
  if (length === 0) return [];
  const per = Math.max(1, Math.ceil(length / slices));
  const roles = Object.entries(result.profile.roles)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4)
    .map(([role]) => role);
  const sum = (values: number[] | undefined, from: number) =>
    (values ?? []).slice(from, from + per).reduce((a, b) => a + b, 0);
  const rows: string[] = [];
  for (let from = 0; from < length; from += per) {
    const width = Math.min(per, length - from) * timeline.bucketMs * 1e6;
    const cores = (period: number) => (period / width).toFixed(1);
    const top = Object.entries(timeline.phases)
      .map(([phase, values]) => [phase, sum(values, from)] as const)
      .sort((a, b) => b[1] - a[1])[0];
    const start = from * timeline.bucketMs;
    rows.push(
      `| ${start}–${start + per * timeline.bucketMs} | ${roles.map((role) => cores(sum(timeline.roles[role], from))).join(" | ")} | ${top && top[1] > 0 ? `${cell(top[0])} ${cores(top[1])}` : ""} |`,
    );
  }
  return [
    "Busy cores over the run (from the first sample):",
    "",
    `| ms | ${roles.map(cell).join(" | ")} | busiest phase |`,
    `| --- | ${roles.map(() => "---:").join(" | ")} | --- |`,
    ...rows,
    "",
  ];
}

function qualityLines(result: LaneResult): string[] {
  return Object.entries(result.profile.quality).map(([role, quality]) => {
    const roots = quality.roots
      .slice(0, 3)
      .map(([root, value]) => `${code(root, 60)} ${pct(value, quality.samples)}`)
      .join(", ");
    return `- ${role}: ${pct(quality.complete, quality.samples)} of samples unwind to a thread/process entry${roots ? `; other outermost frames: ${roots}` : ""}`;
  });
}

function pprofLines(head: LaneResult, base: LaneResult | null, limit = 20): string[] {
  if (!head.pprof) return [];
  const before = new Map(base?.pprof?.functions.map((fn) => [fn.name, fn]));
  const top = [...head.pprof.functions].sort((a, b) => b.flat - a.flat).slice(0, limit);
  const total = head.pprof.total;
  return [
    `tsgo \`--pprofDir\` CPU profile (separate untimed run, ${ms(total / 1e6)} sampled, inlined frames attributed):`,
    "",
    `| # | Go function | flat | cum${base?.pprof ? " | Δ flat" : ""} |`,
    `| ---: | --- | ---: | ---:${base?.pprof ? " | ---:" : ""} |`,
    ...top.map((fn, index) => {
      const b = before.get(fn.name);
      const change = base?.pprof ? ` | ${delta((b?.flat ?? 0) / 1e6, fn.flat / 1e6)}` : "";
      return `| ${index + 1} | ${code(fn.name)} | ${pct(fn.flat, total)} | ${pct(fn.cum, total)}${change} |`;
    }),
    "",
  ];
}

const HOST_STATS = /time$|^Mapper|mapper|wait/iu;

function hostStatLines(head: LaneResult, base: LaneResult | null): string[] {
  const stats = head.hostStats.filter(([key]) => HOST_STATS.test(key));
  if (stats.length === 0) return [];
  const before = new Map(base?.hostStats);
  return [
    "Host `--extendedDiagnostics` (from the perf stat run):",
    "",
    base ? "| Statistic | base | head |" : "| Statistic | head |",
    base ? "| --- | ---: | ---: |" : "| --- | ---: |",
    ...stats.map(([key, value]) =>
      base
        ? `| ${cell(key)} | ${before.get(key) ?? "n/a"} | ${value} |`
        : `| ${cell(key)} | ${value} |`,
    ),
    "",
  ];
}

/** One lane of one corpus: head tables, with base columns and diffs when a base ran. */
export function renderLane(head: LaneResult, base: LaneResult | null, top = 20): string {
  const p = head.profile;
  const lines = [
    `### ${head.corpus}: ${head.label}`,
    "",
    `${p.samples} samples of \`${p.event}\` at ${head.frequency} Hz, \`--call-graph ${head.callGraph}\`, ${ms(toMs(head, p.period))} of CPU in the process tree, wall ${ms(head.wallMs)}, ${head.diagnostics} diagnostics.${base ? ` Base: ${ms(toMs(base, base.profile.period))} CPU, wall ${ms(base.wallMs)}.` : ""}`,
    "",
    ...head.notes.map((note) => `> ${note}`),
    ...(head.notes.length ? [""] : []),
    ...counterTable(head, base),
    ...hostStatLines(head, base),
    "Process roles:",
    "",
    ...keyedTable("Process", head, p.roles, base, base?.profile.roles ?? null),
    ...timelineTable(head),
    "Self time by DSO:",
    "",
    ...keyedTable("DSO", head, p.dsos, base, base?.profile.dsos ?? null),
    "Phases (whole-stack rules, typecheck-profile-rules.ts):",
    "",
    ...phaseKindTable(head),
    ...(base
      ? ["Phase change:", "", ...keyedTable("Phase", head, phaseRows(head), base, phaseRows(base))]
      : []),
    "Cost kind of the leaf frame:",
    "",
    ...keyedTable("Kind", head, p.kinds, base, base?.profile.kinds ?? null),
    "Syscalls (inclusive):",
    "",
    ...keyedTable("Syscall", head, p.syscalls, base, base?.profile.syscalls ?? null, 12),
    `Top ${top} functions by self time:`,
    "",
    ...functionTable(head, base, "self", top),
    `Top ${top} functions by inclusive time:`,
    "",
    ...functionTable(head, base, "total", top),
  ];
  if (base) {
    for (const by of ["self", "total"] as const) {
      lines.push(`Largest ${by}-time moves, base → head:`, "");
      lines.push(
        "| Function | DSO | Process | base | head | Δ |",
        "| --- | --- | --- | ---: | ---: | ---: |",
      );
      for (const { cost, baseMs, headMs } of functionDiff(base, head, by)) {
        lines.push(
          `| ${code(cost.name)} | ${cell(cost.dso)} | ${cell(cost.role)} | ${ms(baseMs)} | ${ms(headMs)} | ${delta(baseMs, headMs)} |`,
        );
      }
      lines.push("");
    }
  }
  lines.push(...pprofLines(head, base));
  lines.push("Stack unwinding:", "", ...qualityLines(head), "");
  const unresolved = Object.entries(p.unresolved).sort((a, b) => b[1] - a[1]);
  if (unresolved.length) {
    lines.push(
      `Samples with unnamed frames, by DSO: ${unresolved.map(([dso, value]) => `${dso} ${pct(value, p.period)}`).join(", ")}`,
      "",
    );
  }
  const { resolved, missed, examples } = p.symbolization ?? {
    resolved: {},
    missed: 0,
    examples: [],
  };
  const named = Object.entries(resolved);
  if (named.length || missed) {
    lines.push(
      `Stripped-binary frames named from their own tables: ${named.map(([how, count]) => `${count} via ${how}`).join(", ") || "none"}; ${missed} missed${examples.length ? ` (${examples.slice(0, 3).join("; ")})` : ""}.`,
      "",
    );
  }
  return lines.join("\n");
}

/** One row per corpus and lane: CPU, wall, the two largest roles, libc and kernel shares. */
export function renderOverview(results: LaneResult[]): string {
  const heads = results.filter((result) => result.side === "head");
  const baseOf = (head: LaneResult) =>
    results.find((r) => r.side === "base" && r.corpus === head.corpus && r.lane === head.lane) ??
    null;
  const lines = [
    "| Corpus | Lane | CPU (head) | Δ CPU vs base | wall | largest processes | libc | kernel | largest phase |",
    "| --- | --- | ---: | ---: | ---: | --- | ---: | ---: | --- |",
  ];
  for (const head of heads) {
    const base = baseOf(head);
    const p = head.profile;
    const cpu = toMs(head, p.period);
    const roles = Object.entries(p.roles)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 2)
      .map(([role, value]) => `${role} ${pct(value, p.period)}`)
      .join(", ");
    const phase = Object.entries(phaseRows(head)).sort((a, b) => b[1] - a[1])[0];
    const change = base ? delta(toMs(base, base.profile.period), cpu) : "";
    lines.push(
      `| ${head.corpus} | ${cell(head.label)} | ${ms(cpu)} | ${change} | ${ms(head.wallMs)} | ${cell(roles)} | ${pct(p.dsos.libc ?? 0, p.period)} | ${pct(p.dsos.kernel ?? 0, p.period)} | ${phase ? `${cell(phase[0])} ${pct(phase[1], p.period)}` : ""} |`,
    );
  }
  return [...lines, ""].join("\n");
}
