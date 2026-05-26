import type { ThemedCodeToken } from "../../shared/codeHighlighting";
import type { DiffLine } from "./types";

export type HighlightedDiffLine = DiffLine & {
  tokens: ThemedCodeToken[];
};

export interface SplitDiffSide {
  kind: "same" | "remove" | "add" | "empty";
  line: number | null;
  tokens: ThemedCodeToken[];
}

export interface SplitDiffRow {
  left: SplitDiffSide;
  right: SplitDiffSide;
}

type ChangedPair = {
  removal: HighlightedDiffLine | null;
  addition: HighlightedDiffLine | null;
};

type AlignAction = "pair" | "remove" | "add";

const MIN_CHANGED_LINE_SCORE = 0.45;
const PAIR_BONUS = 0.06;

export function diffLineText(line: DiffLine): string {
  return line.text || " ";
}

export function renderPlainDiffLines(lines: DiffLine[]): HighlightedDiffLine[] {
  return lines.map((line) => ({
    ...line,
    tokens: [
      {
        content: diffLineText(line),
        darkColor: undefined,
        lightColor: undefined,
      },
    ],
  }));
}

export function buildSplitDiffRows(lines: HighlightedDiffLine[]): SplitDiffRow[] {
  const rows: SplitDiffRow[] = [];

  for (let index = 0; index < lines.length; ) {
    const line = lines[index]!;
    if (line.kind === "same") {
      rows.push({
        left: toSplitSide(line, "left"),
        right: toSplitSide(line, "right"),
      });
      index += 1;
      continue;
    }

    const removals: HighlightedDiffLine[] = [];
    const additions: HighlightedDiffLine[] = [];
    while (index < lines.length && lines[index]!.kind !== "same") {
      const changedLine = lines[index]!;
      if (changedLine.kind === "remove") {
        removals.push(changedLine);
      } else {
        additions.push(changedLine);
      }
      index += 1;
    }

    for (const pair of alignChangedLines(removals, additions)) {
      rows.push({
        left: pair.removal ? toSplitSide(pair.removal, "left") : emptySplitSide(),
        right: pair.addition ? toSplitSide(pair.addition, "right") : emptySplitSide(),
      });
    }
  }

  return rows;
}

function alignChangedLines(
  removals: HighlightedDiffLine[],
  additions: HighlightedDiffLine[],
): ChangedPair[] {
  const rowCount = removals.length + 1;
  const colCount = additions.length + 1;
  const scores = Array.from({ length: rowCount }, () => Array<number>(colCount).fill(0));
  const actions = Array.from({ length: rowCount }, () =>
    Array<AlignAction | null>(colCount).fill(null),
  );

  for (let leftIndex = removals.length; leftIndex >= 0; leftIndex -= 1) {
    for (let rightIndex = additions.length; rightIndex >= 0; rightIndex -= 1) {
      if (leftIndex === removals.length && rightIndex === additions.length) {
        continue;
      }

      let bestScore = Number.NEGATIVE_INFINITY;
      let bestAction: AlignAction | null = null;

      if (leftIndex < removals.length) {
        bestScore = scores[leftIndex + 1]![rightIndex]!;
        bestAction = "remove";
      }

      if (rightIndex < additions.length) {
        const addScore = scores[leftIndex]![rightIndex + 1]!;
        if (addScore > bestScore) {
          bestScore = addScore;
          bestAction = "add";
        }
      }

      if (leftIndex < removals.length && rightIndex < additions.length) {
        const lineScore = diffLineSimilarity(
          removals[leftIndex]!.text,
          additions[rightIndex]!.text,
        );
        const pairScore =
          lineScore >= MIN_CHANGED_LINE_SCORE
            ? scores[leftIndex + 1]![rightIndex + 1]! + lineScore + PAIR_BONUS
            : Number.NEGATIVE_INFINITY;
        if (pairScore > bestScore) {
          bestScore = pairScore;
          bestAction = "pair";
        }
      }

      scores[leftIndex]![rightIndex] = bestScore;
      actions[leftIndex]![rightIndex] = bestAction;
    }
  }

  const pairs: ChangedPair[] = [];
  let leftIndex = 0;
  let rightIndex = 0;

  while (leftIndex < removals.length || rightIndex < additions.length) {
    const action = actions[leftIndex]?.[rightIndex];
    if (action === "pair") {
      pairs.push({
        removal: removals[leftIndex]!,
        addition: additions[rightIndex]!,
      });
      leftIndex += 1;
      rightIndex += 1;
    } else if (action === "add") {
      pairs.push({
        removal: null,
        addition: additions[rightIndex]!,
      });
      rightIndex += 1;
    } else {
      pairs.push({
        removal: removals[leftIndex]!,
        addition: null,
      });
      leftIndex += 1;
    }
  }

  return pairs;
}

function emptySplitSide(): SplitDiffSide {
  return {
    kind: "empty",
    line: null,
    tokens: [
      {
        content: "",
        darkColor: undefined,
        lightColor: undefined,
      },
    ],
  };
}

function toSplitSide(line: HighlightedDiffLine, side: "left" | "right"): SplitDiffSide {
  return {
    kind: line.kind,
    line: side === "left" ? line.leftLine : line.rightLine,
    tokens: line.tokens,
  };
}

function diffLineSimilarity(left: string, right: string): number {
  const normalizedLeft = normalizeLineForSimilarity(left);
  const normalizedRight = normalizeLineForSimilarity(right);

  if (!normalizedLeft || !normalizedRight) {
    return normalizedLeft === normalizedRight ? 1 : 0;
  }

  if (normalizedLeft === normalizedRight) {
    return 1;
  }

  const compactLeft = normalizedLeft.replace(/\s+/g, "");
  const compactRight = normalizedRight.replace(/\s+/g, "");
  if (compactLeft === compactRight) {
    return 0.96;
  }

  const lengthRatio =
    Math.min(compactLeft.length, compactRight.length) /
    Math.max(compactLeft.length, compactRight.length);
  if (Math.max(compactLeft.length, compactRight.length) >= 16 && lengthRatio < 0.42) {
    return 0;
  }

  const charScore = diceScore(compactLeft, compactRight);
  const tokenScore = diceScore(
    tokenizeSimilarityParts(normalizedLeft),
    tokenizeSimilarityParts(normalizedRight),
  );

  return Math.min(1, charScore * 0.58 + tokenScore * 0.42);
}

function normalizeLineForSimilarity(value: string): string {
  return value
    .trim()
    .replace(/\b_ctx\./g, "")
    .replace(/\b_([A-Za-z]\w*)/g, "$1")
    .replace(/\s+/g, " ");
}

function tokenizeSimilarityParts(value: string): string[] {
  return value.match(/[A-Za-z_$][\w$]*|\d+|[^\s\w$]/g) ?? [];
}

function diceScore(left: string | string[], right: string | string[]): number {
  const leftParts = Array.isArray(left) ? left : bigrams(left);
  const rightParts = Array.isArray(right) ? right : bigrams(right);

  if (leftParts.length === 0 || rightParts.length === 0) {
    return leftParts.length === rightParts.length ? 1 : 0;
  }

  const rightCounts = new Map<string, number>();
  for (const part of rightParts) {
    rightCounts.set(part, (rightCounts.get(part) ?? 0) + 1);
  }

  let intersection = 0;
  for (const part of leftParts) {
    const count = rightCounts.get(part) ?? 0;
    if (count > 0) {
      intersection += 1;
      rightCounts.set(part, count - 1);
    }
  }

  return (2 * intersection) / (leftParts.length + rightParts.length);
}

function bigrams(value: string): string[] {
  if (value.length <= 1) {
    return value ? [value] : [];
  }

  const result: string[] = [];
  for (let index = 0; index < value.length - 1; index += 1) {
    result.push(value.slice(index, index + 2));
  }
  return result;
}
