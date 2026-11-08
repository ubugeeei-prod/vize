/** Internal, dependency-free line and token diffing for the Diff component. */

export type DiffLineKind = "context" | "added" | "removed";

export interface DiffSegment {
  text: string;
  changed: boolean;
}

export interface DiffLine {
  kind: DiffLineKind;
  text: string;
  oldLine?: number;
  newLine?: number;
  segments: DiffSegment[];
}

export interface DiffHunkHeader {
  kind: "hunk";
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  text: string;
}

export type UnifiedDiffRow = DiffLine | DiffHunkHeader;

export interface SplitDiffLine {
  kind: "split";
  oldLine?: DiffLine;
  newLine?: DiffLine;
}

export type SplitDiffRow = SplitDiffLine | DiffHunkHeader;

interface SequenceEdit<Value> {
  kind: "equal" | "added" | "removed";
  value: Value;
}

function diagonalValue(diagonals: ReadonlyMap<number, number>, diagonal: number): number {
  return diagonals.get(diagonal) ?? Number.NEGATIVE_INFINITY;
}

/** Compute a shortest edit script with Myers' O((N + M)D) algorithm. */
function diffSequence<Value>(
  previous: readonly Value[],
  next: readonly Value[],
): SequenceEdit<Value>[] {
  if (previous.length === 0) return next.map((value) => ({ kind: "added", value }));
  if (next.length === 0) return previous.map((value) => ({ kind: "removed", value }));

  const maximumDistance = previous.length + next.length;
  const trace: Map<number, number>[] = [];
  const diagonals = new Map<number, number>([[1, 0]]);

  for (let distance = 0; distance <= maximumDistance; distance += 1) {
    trace.push(new Map(diagonals));
    for (let diagonal = -distance; diagonal <= distance; diagonal += 2) {
      const followsInsertion =
        diagonal === -distance ||
        (diagonal !== distance &&
          diagonalValue(diagonals, diagonal - 1) < diagonalValue(diagonals, diagonal + 1));
      let previousIndex = followsInsertion
        ? diagonalValue(diagonals, diagonal + 1)
        : diagonalValue(diagonals, diagonal - 1) + 1;
      if (!Number.isFinite(previousIndex)) previousIndex = 0;
      let nextIndex = previousIndex - diagonal;

      while (
        previousIndex < previous.length &&
        nextIndex < next.length &&
        Object.is(previous[previousIndex], next[nextIndex])
      ) {
        previousIndex += 1;
        nextIndex += 1;
      }
      diagonals.set(diagonal, previousIndex);

      if (previousIndex >= previous.length && nextIndex >= next.length) {
        return backtrack(trace, previous, next, distance);
      }
    }
  }

  return [];
}

function backtrack<Value>(
  trace: readonly ReadonlyMap<number, number>[],
  previous: readonly Value[],
  next: readonly Value[],
  distance: number,
): SequenceEdit<Value>[] {
  const edits: SequenceEdit<Value>[] = [];
  let previousIndex = previous.length;
  let nextIndex = next.length;

  for (let step = distance; step > 0; step -= 1) {
    const diagonals = trace[step];
    if (!diagonals) break;
    const diagonal = previousIndex - nextIndex;
    const followsInsertion =
      diagonal === -step ||
      (diagonal !== step &&
        diagonalValue(diagonals, diagonal - 1) < diagonalValue(diagonals, diagonal + 1));
    const previousDiagonal = followsInsertion ? diagonal + 1 : diagonal - 1;
    const previousSnakeIndex = diagonals.get(previousDiagonal) ?? 0;
    const nextSnakeIndex = previousSnakeIndex - previousDiagonal;

    while (previousIndex > previousSnakeIndex && nextIndex > nextSnakeIndex) {
      previousIndex -= 1;
      nextIndex -= 1;
      edits.push({ kind: "equal", value: previous[previousIndex] as Value });
    }

    if (followsInsertion) {
      nextIndex -= 1;
      edits.push({ kind: "added", value: next[nextIndex] as Value });
    } else {
      previousIndex -= 1;
      edits.push({ kind: "removed", value: previous[previousIndex] as Value });
    }
  }

  while (previousIndex > 0 && nextIndex > 0) {
    previousIndex -= 1;
    nextIndex -= 1;
    edits.push({ kind: "equal", value: previous[previousIndex] as Value });
  }
  while (previousIndex > 0) {
    previousIndex -= 1;
    edits.push({ kind: "removed", value: previous[previousIndex] as Value });
  }
  while (nextIndex > 0) {
    nextIndex -= 1;
    edits.push({ kind: "added", value: next[nextIndex] as Value });
  }

  return edits.reverse();
}

function splitLines(value: string): string[] {
  if (value.length === 0) return [];
  return value.replaceAll("\r\n", "\n").replaceAll("\r", "\n").split("\n");
}

function tokenize(value: string): string[] {
  return value.match(/\s+|[\p{L}\p{N}_]+|[^\s\p{L}\p{N}_]+/gu) ?? [];
}

function mergeSegments(segments: DiffSegment[]): DiffSegment[] {
  const merged: DiffSegment[] = [];
  for (const segment of segments) {
    const last = merged.at(-1);
    if (last?.changed === segment.changed) last.text += segment.text;
    else merged.push({ ...segment });
  }
  return merged;
}

function wordSegments(previous: string, next: string): [DiffSegment[], DiffSegment[]] {
  const edits = diffSequence(tokenize(previous), tokenize(next));
  const removed: DiffSegment[] = [];
  const added: DiffSegment[] = [];

  for (const edit of edits) {
    if (edit.kind !== "added") {
      removed.push({ text: edit.value, changed: edit.kind === "removed" });
    }
    if (edit.kind !== "removed") {
      added.push({ text: edit.value, changed: edit.kind === "added" });
    }
  }
  return [mergeSegments(removed), mergeSegments(added)];
}

function pairChangedLines(lines: DiffLine[]): void {
  for (let index = 0; index < lines.length;) {
    if (lines[index]?.kind === "context") {
      index += 1;
      continue;
    }
    const blockStart = index;
    while (index < lines.length && lines[index]?.kind !== "context") index += 1;
    const block = lines.slice(blockStart, index);
    const removed = block.filter((line) => line.kind === "removed");
    const added = block.filter((line) => line.kind === "added");

    for (let pairIndex = 0; pairIndex < Math.min(removed.length, added.length); pairIndex += 1) {
      const previous = removed[pairIndex];
      const next = added[pairIndex];
      if (!previous || !next) continue;
      [previous.segments, next.segments] = wordSegments(previous.text, next.text);
    }
  }
}

function normalizeChangeBlocks(lines: DiffLine[]): DiffLine[] {
  const normalized: DiffLine[] = [];
  for (let index = 0; index < lines.length;) {
    if (lines[index]?.kind === "context") {
      normalized.push(lines[index] as DiffLine);
      index += 1;
      continue;
    }
    const block: DiffLine[] = [];
    while (index < lines.length && lines[index]?.kind !== "context") {
      block.push(lines[index] as DiffLine);
      index += 1;
    }
    normalized.push(...block.filter((line) => line.kind === "removed"));
    normalized.push(...block.filter((line) => line.kind === "added"));
  }
  return normalized;
}

/** Build numbered line edits and word-level highlight segments. */
export function createDiffLines(previous: string, next: string): DiffLine[] {
  let oldLine = 1;
  let newLine = 1;
  const lines = diffSequence(splitLines(previous), splitLines(next)).map((edit): DiffLine => {
    if (edit.kind === "equal") {
      const line: DiffLine = {
        kind: "context",
        text: edit.value,
        oldLine,
        newLine,
        segments: [{ text: edit.value, changed: false }],
      };
      oldLine += 1;
      newLine += 1;
      return line;
    }
    if (edit.kind === "removed") {
      const line: DiffLine = {
        kind: "removed",
        text: edit.value,
        oldLine,
        segments: [{ text: edit.value, changed: true }],
      };
      oldLine += 1;
      return line;
    }
    const line: DiffLine = {
      kind: "added",
      text: edit.value,
      newLine,
      segments: [{ text: edit.value, changed: true }],
    };
    newLine += 1;
    return line;
  });
  const normalized = normalizeChangeBlocks(lines);
  pairChangedLines(normalized);
  return normalized;
}

function hunkHeader(lines: readonly DiffLine[], start: number, end: number): DiffHunkHeader {
  const prior = lines.slice(0, start);
  const body = lines.slice(start, end);
  const oldStart = prior.filter((line) => line.oldLine !== undefined).length + 1;
  const newStart = prior.filter((line) => line.newLine !== undefined).length + 1;
  const oldCount = body.filter((line) => line.oldLine !== undefined).length;
  const newCount = body.filter((line) => line.newLine !== undefined).length;
  return {
    kind: "hunk",
    oldStart,
    oldCount,
    newStart,
    newCount,
    text: `@@ -${oldStart},${oldCount} +${newStart},${newCount} @@`,
  };
}

/** Retain only requested context around changes and insert stable hunk headers. */
export function createUnifiedRows(
  previous: string,
  next: string,
  context: number,
): UnifiedDiffRow[] {
  const lines = createDiffLines(previous, next);
  const changed = lines.flatMap((line, index) => (line.kind === "context" ? [] : [index]));
  if (changed.length === 0) return lines;

  const retainedContext = Number.isFinite(context) ? Math.max(0, Math.trunc(context)) : 3;
  const ranges: Array<{ start: number; end: number }> = [];
  for (const index of changed) {
    const start = Math.max(0, index - retainedContext);
    const end = Math.min(lines.length, index + retainedContext + 1);
    const previousRange = ranges.at(-1);
    if (previousRange && start <= previousRange.end)
      previousRange.end = Math.max(previousRange.end, end);
    else ranges.push({ start, end });
  }

  return ranges.flatMap(({ start, end }) => [
    hunkHeader(lines, start, end),
    ...lines.slice(start, end),
  ]);
}

/** Pair removed and added lines for a side-by-side view. */
export function createSplitRows(rows: readonly UnifiedDiffRow[]): SplitDiffRow[] {
  const split: SplitDiffRow[] = [];
  for (let index = 0; index < rows.length;) {
    const row = rows[index];
    if (!row) break;
    if (row.kind === "hunk") {
      split.push(row);
      index += 1;
      continue;
    }
    if (row.kind === "context") {
      split.push({ kind: "split", oldLine: row, newLine: row });
      index += 1;
      continue;
    }

    const changed: DiffLine[] = [];
    while (index < rows.length && rows[index]?.kind !== "context" && rows[index]?.kind !== "hunk") {
      changed.push(rows[index] as DiffLine);
      index += 1;
    }
    const removed = changed.filter((line) => line.kind === "removed");
    const added = changed.filter((line) => line.kind === "added");
    for (let pairIndex = 0; pairIndex < Math.max(removed.length, added.length); pairIndex += 1) {
      split.push({ kind: "split", oldLine: removed[pairIndex], newLine: added[pairIndex] });
    }
  }
  return split;
}
