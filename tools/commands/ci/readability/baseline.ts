// The readability baseline and its shrink-only comparison.
//
// The baseline is a TSV with one row per file that has a nonzero counter.
// The comparison is per area (a crate, package or top-level tool directory):
//
// - `lines` is per file. A file over the limit may not grow, and a new file
//   over the limit must take the row of a deleted file at least as long
//   (a move or rename).
// - Every other counter is an area total. It may not exceed the area's
//   baseline total, so a split or a move inside an area is free, and a new
//   violation must be paid for by removing one. The budget of deleted files
//   may pay for growth in another area, so a cross-area rename is free too.
//
// `--write` records the current counts, which can only lower the totals,
// because it refuses to run while the comparison fails.

import { counterNames, emptyCounts, type CounterName, type Counts } from "./counters.ts";

export type FileCounts = ReadonlyMap<string, Counts>;

export const baselineHeader = [
  "# Readability ratchet baseline (shrink-only). Generated; do not edit by hand.",
  "# Lower it with `node tools/commands/ci/readability-ratchet.ts --write`.",
  "# Counters are defined in tools/commands/ci/readability/counters.ts.",
];

export function parseBaseline(text: string): Map<string, Counts> {
  const rows = new Map<string, Counts>();
  let columns: CounterName[] | undefined;
  for (const [index, line] of text.split("\n").entries()) {
    if (!line || line.startsWith("#")) continue;
    const cells = line.split("\t");
    if (!columns) {
      if (cells[0] !== "path") throw new Error(`baseline line ${index + 1}: expected a header`);
      columns = cells.slice(1) as CounterName[];
      for (const column of columns)
        if (!counterNames.includes(column))
          throw new Error(`baseline line ${index + 1}: unknown counter ${column}`);
      continue;
    }
    const counts = emptyCounts();
    for (const [offset, column] of columns.entries()) {
      const value = Number(cells[offset + 1]);
      if (!Number.isSafeInteger(value) || value < 0)
        throw new Error(`baseline line ${index + 1}: ${column} is not a count`);
      counts[column] = value;
    }
    if (rows.has(cells[0])) throw new Error(`baseline line ${index + 1}: duplicate ${cells[0]}`);
    rows.set(cells[0], counts);
  }
  return rows;
}

const isZero = (counts: Counts) => counterNames.every((name) => counts[name] === 0);

export function renderBaseline(files: FileCounts): string {
  const rows = [...files]
    .filter(([, counts]) => !isZero(counts))
    .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
    .map(([path, counts]) => [path, ...counterNames.map((name) => counts[name])].join("\t"));
  return [...baselineHeader, ["path", ...counterNames].join("\t"), ...rows, ""].join("\n");
}

/** `crates/vize_x/…` → `crates/vize_x`; a root file is its own area. */
export function areaOf(path: string): string {
  const parts = path.split("/");
  return parts.length > 2 ? `${parts[0]}/${parts[1]}` : parts[0];
}

export interface Finding {
  readonly area: string;
  readonly counter: CounterName;
  readonly baseline: number;
  readonly current: number;
  /** Files whose own count is above their baseline row, worst first. */
  readonly files: readonly { path: string; baseline: number; current: number }[];
}

export interface Comparison {
  readonly findings: readonly Finding[];
  /** Per counter: baseline and current totals over the whole tree. */
  readonly totals: Readonly<Record<CounterName, { baseline: number; current: number }>>;
  /** Rows `--write` would lower or drop. */
  readonly lowerable: number;
}

function sum(rows: Iterable<Counts>, counter: CounterName): number {
  let total = 0;
  for (const counts of rows) total += counts[counter];
  return total;
}

export function compare(baseline: FileCounts, current: FileCounts): Comparison {
  const findings: Finding[] = [];
  const zero = emptyCounts();
  const deleted = [...baseline.keys()].filter((path) => !current.has(path));
  const totals = Object.fromEntries(
    counterNames.map((name) => [
      name,
      { baseline: sum(baseline.values(), name), current: sum(current.values(), name) },
    ]),
  ) as Comparison["totals"];

  // `lines`: per file, with deleted rows free to be claimed by moved files.
  const unclaimed = new Map(deleted.map((path) => [path, baseline.get(path)!.lines]));
  const byLength = [...current]
    .filter(([, counts]) => counts.lines > 0)
    .sort(([, left], [, right]) => right.lines - left.lines);
  for (const [path, counts] of byLength) {
    const limit = baseline.get(path)?.lines ?? 0;
    if (counts.lines <= limit) continue;
    if (limit === 0) {
      const name = path.split("/").at(-1);
      const candidates = [...unclaimed].filter(([, lines]) => lines >= counts.lines);
      const claim = candidates.find(([other]) => other.split("/").at(-1) === name) ?? candidates[0];
      if (claim) {
        unclaimed.delete(claim[0]);
        continue;
      }
    }
    findings.push({
      area: areaOf(path),
      counter: "lines",
      baseline: limit,
      current: counts.lines,
      files: [{ path, baseline: limit, current: counts.lines }],
    });
  }

  // Every other counter: area totals, with deleted files' budget transferable.
  const areaTotals = new Map<string, { base: Counts; now: Counts; freed: Counts }>();
  const totalsOf = (path: string) => {
    const area = areaOf(path);
    let entry = areaTotals.get(area);
    if (!entry) {
      entry = { base: emptyCounts(), now: emptyCounts(), freed: emptyCounts() };
      areaTotals.set(area, entry);
    }
    return entry;
  };
  for (const [path, counts] of baseline)
    for (const name of counterNames) {
      totalsOf(path).base[name] += counts[name];
      if (!current.has(path)) totalsOf(path).freed[name] += counts[name];
    }
  for (const [path, counts] of current)
    for (const name of counterNames) totalsOf(path).now[name] += counts[name];
  for (const counter of counterNames.filter((name) => name !== "lines")) {
    const excess = new Map<string, { baseline: number; current: number }>();
    let transferable = 0;
    for (const [area, { base, now, freed }] of areaTotals) {
      if (now[counter] > base[counter])
        excess.set(area, { baseline: base[counter], current: now[counter] });
      else transferable += Math.min(freed[counter], base[counter] - now[counter]);
    }
    for (const [area, total] of [...excess].sort(
      ([, left], [, right]) => left.current - left.baseline - (right.current - right.baseline),
    )) {
      const over = total.current - total.baseline;
      if (over <= transferable) {
        transferable -= over;
        continue;
      }
      const files = [...current]
        .filter(([path, counts]) => areaOf(path) === area && counts[counter] > 0)
        .map(([path, counts]) => ({
          path,
          baseline: (baseline.get(path) ?? zero)[counter],
          current: counts[counter],
        }))
        .filter((file) => file.current > file.baseline)
        .sort((left, right) => right.current - right.baseline - (left.current - left.baseline));
      findings.push({ area, counter, ...total, files });
    }
  }

  let lowerable = deleted.length;
  for (const [path, counts] of baseline) {
    const now = current.get(path);
    if (now && counterNames.some((name) => now[name] < counts[name])) lowerable++;
  }
  return {
    findings: findings.sort((left, right) =>
      left.area === right.area
        ? counterNames.indexOf(left.counter) - counterNames.indexOf(right.counter)
        : left.area < right.area
          ? -1
          : 1,
    ),
    totals,
    lowerable,
  };
}
