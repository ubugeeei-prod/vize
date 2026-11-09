/** Pure statistics, strace attribution and Markdown for `typecheck-cold-pair.ts`. */

export type CheckSample = {
  wallMs: number;
  /** User + system seconds of the whole process tree (GNU time) or of vize alone (Windows). */
  cpuSeconds?: number;
  /** Peak resident set of the largest process in the tree (GNU time) or of vize (Windows). */
  peakMiB?: number;
  /** Sampled high-water marks: vize itself and the sum over its checker children. */
  vizeHwmMiB?: number;
  childHwmMiB?: number;
  exitCode: number;
  outputBytes: number;
  outputSha: string;
  mirrorFiles: number;
  mirrorBytes: number;
};

export type BuildSummary = {
  id: string;
  wallMs: number;
  wallMin: number;
  wallMax: number;
  cpuSeconds?: number;
  peakMiB?: number;
  vizeHwmMiB?: number;
  childHwmMiB?: number;
  mirrorFiles: number;
  mirrorBytes: number;
  exitCode: number;
};

export type Cell = {
  label: string;
  project: string;
  runs: number;
  builds: BuildSummary[];
  outputsIdentical: boolean;
  failure?: string;
  samples?: Record<string, CheckSample[]>;
};

export function median(values: readonly number[]): number {
  if (values.length === 0) return NaN;
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[middle] ?? NaN)
    : ((sorted[middle - 1] ?? NaN) + (sorted[middle] ?? NaN)) / 2;
}

const defined = (values: (number | undefined)[]): number[] =>
  values.filter((value): value is number => value !== undefined && Number.isFinite(value));
const medianOf = (values: (number | undefined)[]): number | undefined => {
  const present = defined(values);
  return present.length ? median(present) : undefined;
};

export function summarizeCell(input: {
  label: string;
  project: string;
  builds: string[];
  samples: Map<string, CheckSample[]>;
  allowOutputChange: boolean;
}): Cell {
  const shas = new Set<string>();
  let unstable: string | undefined;
  for (const id of input.builds) {
    const own = new Set((input.samples.get(id) ?? []).map((s) => `${s.exitCode}:${s.outputSha}`));
    if (own.size !== 1) unstable ??= id;
    for (const sha of own) shas.add(sha);
  }
  const outputsIdentical = shas.size === 1;
  const failure = unstable
    ? `${unstable} reported runs differently`
    : !outputsIdentical && !input.allowOutputChange
      ? "builds reported different diagnostics"
      : undefined;
  const builds = input.builds.map((id): BuildSummary => {
    const samples = input.samples.get(id) ?? [];
    const walls = samples.map((sample) => sample.wallMs);
    return {
      id,
      wallMs: median(walls),
      wallMin: Math.min(...walls),
      wallMax: Math.max(...walls),
      cpuSeconds: medianOf(samples.map((s) => s.cpuSeconds)),
      peakMiB: medianOf(samples.map((s) => s.peakMiB)),
      vizeHwmMiB: medianOf(samples.map((s) => s.vizeHwmMiB)),
      childHwmMiB: medianOf(samples.map((s) => s.childHwmMiB)),
      mirrorFiles: samples[0]?.mirrorFiles ?? 0,
      mirrorBytes: samples[0]?.mirrorBytes ?? 0,
      exitCode: samples[0]?.exitCode ?? -1,
    };
  });
  return {
    label: input.label,
    project: input.project,
    runs: input.samples.get(input.builds[0] ?? "")?.length ?? 0,
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

export function markdown(label: string, cells: readonly Cell[], windows = false): string {
  const lines = [`### vize check pair: ${label}`, ""];
  if (windows)
    lines.push("CPU is vize.exe alone; HWM columns are sampled peak working sets (250 ms).", "");
  for (const cell of cells) {
    lines.push(
      `**${cell.project}**, ${cell.runs} alternating cold runs (fresh cache root each). ` +
        (cell.failure
          ? `**No timing published: ${cell.failure}.**`
          : cell.outputsIdentical
            ? "Diagnostics byte-identical across builds and runs."
            : "Diagnostics differ between builds (allowed)."),
      "",
    );
    if (cell.failure) continue;
    const first = cell.builds[0];
    lines.push(
      "| build | wall ms (min–max) | CPU s | peak MiB | vize HWM MiB | checker HWM MiB | mirror files | mirror MB |",
      "|---|---|---|---|---|---|---|---|",
    );
    for (const build of cell.builds) {
      const base = build === first ? undefined : first;
      lines.push(
        `| ${build.id} | ${fixed(build.wallMs, 0)}${delta(build.wallMs, base?.wallMs)} (${fixed(build.wallMin, 0)}–${fixed(build.wallMax, 0)}) ` +
          `| ${fixed(build.cpuSeconds, 2)}${delta(build.cpuSeconds, base?.cpuSeconds)} ` +
          `| ${fixed(build.peakMiB, 1)}${delta(build.peakMiB, base?.peakMiB)} ` +
          `| ${fixed(build.vizeHwmMiB, 1)}${delta(build.vizeHwmMiB, base?.vizeHwmMiB)} ` +
          `| ${fixed(build.childHwmMiB, 1)}${delta(build.childHwmMiB, base?.childHwmMiB)} ` +
          `| ${build.mirrorFiles}${delta(build.mirrorFiles, base?.mirrorFiles)} ` +
          `| ${fixed(build.mirrorBytes / 1e6, 2)} |`,
      );
    }
    lines.push("");
  }
  return `${lines.join("\n")}\n`;
}

/** Path classes a traced filesystem call is attributed to, most specific first. */
export type PathRoots = { mirror: string; modules: string; project: string; checker: string };
export type Attribution = Map<string, number>;

const under = (path: string, root: string) => path === root || path.startsWith(`${root}/`);

export function classifyPath(path: string | undefined, roots: PathRoots): string {
  if (path === undefined) return "fd";
  if (under(path, roots.mirror)) return "mirror";
  if (under(path, roots.checker)) return "checker-install";
  if (under(path, roots.modules)) return "node_modules";
  if (under(path, roots.project)) return "source";
  if (/^\/(proc|sys|dev)\//u.test(path)) return "procfs";
  return "other";
}

const LINE = /^(\d+)\s+(?:<\.\.\. (\w+) resumed>|(\w+)\()(.*)$/u;
const RESULT = /\)\s+=\s+(-?\d+|\?)(?:\s+(E[A-Z]+))?/u;
const QUOTED = /(?:<(\/[^>]*)>)?,?\s*"((?:[^"\\]|\\.)*)"/u;
const DECORATED = /<(\/[^>]*)>/u;
/** Calls whose first argument is a descriptor: the path is its `-y` decoration. */
const FD_CALLS = new Set([
  "read",
  "pread64",
  "readv",
  "write",
  "pwrite64",
  "writev",
  "getdents64",
  "close",
  "fstat",
  "fsync",
  "fdatasync",
  "ftruncate",
  "fchmod",
  "fchown",
  "flock",
  "lseek",
  "copy_file_range",
]);

/** The path a traced call names: its descriptor's, or its string resolved against a dirfd. */
export function tracedPath(call: string, args: string): string | undefined {
  if (FD_CALLS.has(call)) return DECORATED.exec(args)?.[1];
  const quoted = QUOTED.exec(args);
  if (!quoted) return DECORATED.exec(args)?.[1];
  const [, dir, name = ""] = quoted;
  if (name.startsWith("/") || !dir) return name || dir;
  return name ? `${dir}/${name}` : dir;
}

/**
 * Count `strace -f -y` lines by process (the executable each pid last ran),
 * syscall, path class and outcome. Threads and children inherit their
 * creator's process until they execute another program; lines of a pid seen
 * before its creator's `clone` returns wait for that answer.
 */
export class StraceAttribution {
  readonly counts: Attribution = new Map();
  private readonly owners = new Map<string, string>();
  private readonly pending = new Map<string, { call: string; path?: string }>();
  private readonly orphans = new Map<string, string[]>();
  private readonly roots: PathRoots;

  constructor(roots: PathRoots) {
    this.roots = roots;
  }

  push(raw: string): void {
    const match = LINE.exec(raw);
    if (!match) return;
    const [, pid = "", resumed, started, rest = ""] = match;
    let call = started ?? resumed ?? "";
    let path: string | undefined;
    if (started) {
      path = tracedPath(call, rest);
      if (rest.includes("<unfinished ...>")) {
        this.pending.set(pid, { call, path });
        return;
      }
    } else {
      const open = this.pending.get(pid);
      this.pending.delete(pid);
      call = open?.call ?? call;
      path = open?.path;
    }
    const result = RESULT.exec(rest);
    const value = result?.[1] ?? "?";
    if (/^(clone|clone3|fork|vfork)$/u.test(call)) {
      if (/^\d+$/u.test(value) && value !== "0") this.adopt(value, this.owner(pid));
      return;
    }
    // The traced command's own first line names the root process.
    if (this.owners.size === 0) this.owners.set(pid, "vize");
    if (call === "execve" && value === "0" && path) {
      this.adopt(pid, path.split("/").pop() ?? path);
    }
    const outcome = result?.[2] ?? (value.startsWith("-") ? "error" : "ok");
    const key = `${call}\t${classifyPath(path, this.roots)}\t${outcome}`;
    if (this.owners.has(pid)) {
      this.bump(`${this.owner(pid)}\t${key}`);
      return;
    }
    const waiting = this.orphans.get(pid) ?? [];
    waiting.push(key);
    this.orphans.set(pid, waiting);
  }

  /** Lines whose pid never met its creator count as `unknown`. */
  finish(): Attribution {
    for (const keys of this.orphans.values()) {
      for (const key of keys) this.bump(`unknown\t${key}`);
    }
    this.orphans.clear();
    return this.counts;
  }

  private owner(pid: string): string {
    return this.owners.get(pid) ?? "unknown";
  }

  private adopt(pid: string, owner: string): void {
    this.owners.set(pid, owner);
    for (const key of this.orphans.get(pid) ?? []) this.bump(`${owner}\t${key}`);
    this.orphans.delete(pid);
  }

  private bump(key: string): void {
    this.counts.set(key, (this.counts.get(key) ?? 0) + 1);
  }
}

export function attributeStrace(log: string, roots: PathRoots): Attribution {
  const attribution = new StraceAttribution(roots);
  for (const line of log.split("\n")) attribution.push(line);
  return attribution.finish();
}

/** Rows (process, syscall, path class) of every build, failures split out, largest first. */
export function attributionTable(builds: readonly [string, Attribution][], limit = 80): string {
  const ids = builds.map(([id]) => id);
  const rows = new Map<string, Map<string, { calls: number; failed: number }>>();
  for (const [id, counts] of builds) {
    for (const [key, count] of counts) {
      const [owner = "", call = "", where = "", outcome = ""] = key.split("\t");
      const rowKey = `${owner}\t${call}\t${where}`;
      const row = rows.get(rowKey) ?? new Map<string, { calls: number; failed: number }>();
      const cell = row.get(id) ?? { calls: 0, failed: 0 };
      cell.calls += count;
      if (outcome !== "ok") cell.failed += count;
      row.set(id, cell);
      rows.set(rowKey, row);
    }
  }
  const weight = (row: Map<string, { calls: number }>) =>
    Math.max(...ids.map((id) => row.get(id)?.calls ?? 0));
  const sorted = [...rows].sort((a, b) => weight(b[1]) - weight(a[1])).slice(0, limit);
  const lines = [
    `| process | syscall | path | ${ids.map((id) => `${id} calls (failed)`).join(" | ")} |`,
    `|---|---|---|${ids.map(() => "---|").join("")}`,
  ];
  for (const [key, row] of sorted) {
    const [owner, call, where] = key.split("\t");
    const cells = ids.map((id) => {
      const cell = row.get(id) ?? { calls: 0, failed: 0 };
      return `${cell.calls} (${cell.failed})`;
    });
    lines.push(`| ${owner} | ${call} | ${where} | ${cells.join(" | ")} |`);
  }
  return `${lines.join("\n")}\n`;
}

/** Per-process totals of each build, summed over syscalls and path classes. */
export function processTotals(builds: readonly [string, Attribution][]): string {
  const ids = builds.map(([id]) => id);
  const totals = new Map<string, Map<string, number>>();
  for (const [id, counts] of builds) {
    for (const [key, count] of counts) {
      const owner = key.split("\t")[0] ?? "";
      const row = totals.get(owner) ?? new Map<string, number>();
      row.set(id, (row.get(id) ?? 0) + count);
      totals.set(owner, row);
    }
  }
  const lines = [`| process | ${ids.join(" | ")} |`, `|---|${ids.map(() => "---|").join("")}`];
  for (const [owner, row] of totals) {
    lines.push(`| ${owner} | ${ids.map((id) => row.get(id) ?? 0).join(" | ")} |`);
  }
  return `${lines.join("\n")}\n`;
}
