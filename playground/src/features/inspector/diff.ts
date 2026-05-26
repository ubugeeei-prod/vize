import type { DiffLine, DiffStats } from "./types";

function splitLines(value: string): string[] {
  if (!value) return [];
  return value.replace(/\r\n/g, "\n").split("\n");
}

export function buildLineDiff(left: string, right: string): DiffLine[] {
  const leftLines = splitLines(left);
  const rightLines = splitLines(right);
  const rows = leftLines.length + 1;
  const cols = rightLines.length + 1;
  const table: number[][] = Array.from({ length: rows }, () => Array(cols).fill(0));

  for (let leftIndex = leftLines.length - 1; leftIndex >= 0; leftIndex -= 1) {
    for (let rightIndex = rightLines.length - 1; rightIndex >= 0; rightIndex -= 1) {
      table[leftIndex]![rightIndex] =
        leftLines[leftIndex] === rightLines[rightIndex]
          ? table[leftIndex + 1]![rightIndex + 1]! + 1
          : Math.max(table[leftIndex + 1]![rightIndex]!, table[leftIndex]![rightIndex + 1]!);
    }
  }

  const diff: DiffLine[] = [];
  let leftIndex = 0;
  let rightIndex = 0;

  while (leftIndex < leftLines.length && rightIndex < rightLines.length) {
    if (leftLines[leftIndex] === rightLines[rightIndex]) {
      diff.push({
        kind: "same",
        leftLine: leftIndex + 1,
        rightLine: rightIndex + 1,
        text: leftLines[leftIndex]!,
      });
      leftIndex += 1;
      rightIndex += 1;
    } else if (table[leftIndex + 1]![rightIndex]! >= table[leftIndex]![rightIndex + 1]!) {
      diff.push({
        kind: "remove",
        leftLine: leftIndex + 1,
        rightLine: null,
        text: leftLines[leftIndex]!,
      });
      leftIndex += 1;
    } else {
      diff.push({
        kind: "add",
        leftLine: null,
        rightLine: rightIndex + 1,
        text: rightLines[rightIndex]!,
      });
      rightIndex += 1;
    }
  }

  while (leftIndex < leftLines.length) {
    diff.push({
      kind: "remove",
      leftLine: leftIndex + 1,
      rightLine: null,
      text: leftLines[leftIndex]!,
    });
    leftIndex += 1;
  }

  while (rightIndex < rightLines.length) {
    diff.push({
      kind: "add",
      leftLine: null,
      rightLine: rightIndex + 1,
      text: rightLines[rightIndex]!,
    });
    rightIndex += 1;
  }

  return diff;
}

export function getDiffStats(diff: DiffLine[]): DiffStats {
  return diff.reduce<DiffStats>(
    (stats, line) => {
      if (line.kind === "add") stats.additions += 1;
      if (line.kind === "remove") stats.removals += 1;
      if (line.kind === "same") stats.unchanged += 1;
      return stats;
    },
    { additions: 0, removals: 0, unchanged: 0 },
  );
}
