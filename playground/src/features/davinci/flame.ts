// The flame view (C-4): profiler spans, pass x stage x block, as a flame
// graph - and the same graph against a pinned baseline run. Frames come
// straight from the P0-11 export's attributed totals; nothing is re-timed.
// Like a classic flame graph, width is time and siblings are sorted by name
// (the export ranks spans by cost and carries no run order). The same view
// opens any `--profile-json` file, negotiated like the live export.

import {
  negotiateProfileExport,
  type ProfileExport,
  type ProfileNegotiation,
} from "../../wasm/types/profile";

/** The attribution dimensions, outermost first. */
const LEVELS = ["stage", "pass", "block"] as const;

export interface FlameFrame {
  /** The frame's attribution path, outermost first. */
  path: string[];
  /** 0 = stage, 1 = pass, 2 = block. */
  depth: number;
  /** Offset from the left edge, in nanoseconds of the whole graph. */
  start: number;
  nanos: number;
  /** Calls aggregated into the frame. */
  count: number;
  /** The same frame's time in the baseline run; null when it did not run. */
  baseline: number | null;
}

export interface Flame {
  total: number;
  frames: FlameFrame[];
}

export type FlameTrend = "slower" | "faster" | "same" | "new";

interface Node {
  nanos: number;
  count: number;
  children: Map<string, Node>;
}

const node = (): Node => ({ nanos: 0, count: 0, children: new Map() });

/** The spans recorded under `key`, folded into a stage > pass > block tree. */
function fold(profile: ProfileExport, key: string): Node {
  const root = node();
  for (const span of profile.spans) {
    if (span.key !== key) continue;
    let at = root;
    for (const level of LEVELS) {
      const name = span.attribution?.[level] ?? "(unattributed)";
      const child = at.children.get(name) ?? node();
      at.children.set(name, child);
      child.nanos += span.wall_ns.total;
      child.count += span.count;
      at = child;
    }
  }
  return root;
}

/**
 * The flame graph of the spans recorded under `key`, compared frame by frame
 * with `baseline` when one is pinned.
 */
export function flameGraph(
  profile: ProfileExport,
  key: string,
  baseline: ProfileExport | null = null,
): Flame {
  const frames: FlameFrame[] = [];
  const place = (at: Node, before: Node | null | undefined, path: string[], start: number) => {
    let offset = start;
    for (const name of [...at.children.keys()].sort()) {
      const child = at.children.get(name)!;
      const was = before?.children.get(name);
      const framePath = [...path, name];
      frames.push({
        path: framePath,
        depth: path.length,
        start: offset,
        nanos: child.nanos,
        count: child.count,
        baseline: baseline ? (was?.nanos ?? null) : null,
      });
      place(child, was ?? null, framePath, offset);
      offset += child.nanos;
    }
  };
  const root = fold(profile, key);
  place(root, baseline ? fold(baseline, key) : null, [], 0);
  const total = [...root.children.values()].reduce((sum, child) => sum + child.nanos, 0);
  return { total, frames };
}

/** The span keys in `profile`, heaviest (summed total) first. */
export function flameKeys(profile: ProfileExport): string[] {
  const totals = new Map<string, number>();
  for (const span of profile.spans) {
    totals.set(span.key, (totals.get(span.key) ?? 0) + span.wall_ns.total);
  }
  return [...totals].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).map(([key]) => key);
}

/** A `--profile-json` file's text, negotiated like the live export. */
export function readProfileExport(text: string): ProfileNegotiation {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { ok: false, error: "The file is not JSON." };
  }
  return negotiateProfileExport(raw);
}

/** How a frame moved against the baseline: a tenth either way is noise. */
export function flameTrend(frame: FlameFrame): FlameTrend {
  if (frame.baseline === null) return "new";
  if (frame.baseline === 0) return frame.nanos === 0 ? "same" : "slower";
  const ratio = frame.nanos / frame.baseline;
  if (ratio > 1.1) return "slower";
  if (ratio < 0.9) return "faster";
  return "same";
}
