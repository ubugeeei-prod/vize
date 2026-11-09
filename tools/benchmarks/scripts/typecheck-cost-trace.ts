/**
 * Sampled stacks from a tsgo `--generateTrace` run. tsgo writes a complete
 * (`X`) event for a short check step only when it spans a 1 ms boundary, and
 * B/E pairs always, so at every 1 ms tick the recorded events covering it are
 * exactly the steps running then: a 1 ms sampling profile of each checker.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

export const TICK_US = 1000;

export type TraceArgs = {
  checkerId?: number;
  path?: string;
  pos?: number;
  end?: number;
  sourceId?: number;
  targetId?: number;
  id?: number;
};
export type TraceEvent = {
  ph: string;
  name: string;
  ts: number;
  dur?: number;
  pid?: number;
  tid?: number;
  args?: TraceArgs;
};
export type Frame = { name: string; start: number; end: number; args: TraceArgs };
export type LegendEntry = { tracePath: string; typesPath: string; checkerId: number };

export function readLegend(dir: string): LegendEntry[] {
  const legend = JSON.parse(readFileSync(join(dir, "legend.json"), "utf8")) as LegendEntry[];
  // The legend records absolute paths of the machine that traced; files sit beside it.
  return legend.map((entry) => ({
    ...entry,
    tracePath: join(dir, entry.tracePath.split(/[\\/]/u).pop() as string),
    typesPath: join(dir, entry.typesPath.split(/[\\/]/u).pop() as string),
  }));
}

/** Check-phase frames per checker id, B/E pairs matched per thread. */
export function checkerFrames(events: readonly TraceEvent[]): Map<number, Frame[]> {
  const frames = new Map<number, Frame[]>();
  const open = new Map<string, TraceEvent[]>();
  const add = (event: TraceEvent, start: number, end: number) => {
    const checker = event.args?.checkerId;
    if (checker == null) return;
    const list = frames.get(checker) ?? [];
    list.push({ name: event.name, start, end, args: event.args ?? {} });
    frames.set(checker, list);
  };
  for (const event of events) {
    if (event.ph === "X") add(event, event.ts, event.ts + (event.dur ?? 0));
    const thread = `${event.pid}:${event.tid}`;
    if (event.ph === "B") open.set(thread, [...(open.get(thread) ?? []), event]);
    if (event.ph === "E") {
      const begin = open.get(thread)?.pop();
      if (begin) add({ ...begin, args: { ...begin.args, ...event.args } }, begin.ts, event.ts);
    }
  }
  return frames;
}

/**
 * The stack (outermost first) at every tick inside a `checkSourceFile`.
 * A frame covers tick k when start < k·TICK ≤ end, tsgo's sampling rule.
 */
export function tickStacks(frames: readonly Frame[]): Map<number, Frame[]> {
  const sorted = [...frames].sort((a, b) => a.start - b.start || b.end - a.end);
  const stacks = new Map<number, Frame[]>();
  for (const frame of sorted) {
    const first = Math.floor(frame.start / TICK_US) + 1;
    const last = Math.floor(frame.end / TICK_US);
    for (let tick = first; tick <= last; tick++) {
      const stack = stacks.get(tick);
      if (stack) stack.push(frame);
      else if (frame.name === "checkSourceFile") stacks.set(tick, [frame]);
    }
  }
  return stacks;
}

export const isPositionFrame = (frame: Frame): boolean =>
  frame.args.pos != null && frame.args.path != null;

/**
 * Recorded statement-level frames directly inside each `checkSourceFile` or
 * position frame of the same file, by start: the checks before and after an
 * unframed sample bracket the source range it was checking.
 */
export function statementChildren(frames: readonly Frame[]): Map<Frame, Frame[]> {
  const sorted = [...frames].sort((a, b) => a.start - b.start || b.end - a.end);
  const children = new Map<Frame, Frame[]>();
  const open: Frame[] = [];
  for (const frame of sorted) {
    while (open.length > 0 && (open[open.length - 1] as Frame).end <= frame.start) open.pop();
    if (isPositionFrame(frame)) {
      for (let index = open.length - 1; index >= 0; index--) {
        const parent = open[index] as Frame;
        if (parent.name !== "checkSourceFile" && !isPositionFrame(parent)) continue;
        const list = children.get(parent);
        if (parent.args.path !== frame.args.path) break;
        if (list) list.push(frame);
        else children.set(parent, [frame]);
        break;
      }
    }
    open.push(frame);
  }
  return children;
}

/** Outermost variance measurements per checker, with exact durations (B/E pairs). */
export function varianceSpans(frames: readonly Frame[]): Frame[] {
  const spans = frames
    .filter((frame) => frame.name === "getVariancesWorker")
    .sort((a, b) => a.start - b.start);
  const outer: Frame[] = [];
  for (const span of spans) {
    const last = outer[outer.length - 1];
    if (!last || span.start >= last.end) outer.push(span);
  }
  return outer;
}

export const isTypeFrame = (frame: Frame): boolean =>
  frame.name === "structuredTypeRelatedTo" || frame.name === "getVariancesWorker";

export const frameTypeIds = (frame: Frame): number[] =>
  [frame.args.sourceId, frame.args.targetId, frame.args.id].filter(
    (id): id is number => typeof id === "number",
  );
