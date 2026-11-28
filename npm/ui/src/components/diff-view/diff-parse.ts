/** One classified line of a unified diff. */
export type DiffLineKind = "context" | "add" | "remove" | "meta";

/** A line before it receives a stable render key. */
export interface DiffLineInput {
  readonly kind: DiffLineKind;
  readonly text: string;
}

/** A classified line with a key safe for `v-for`. */
export interface DiffLine extends DiffLineInput {
  readonly key: string;
}

function kindFor(line: string): DiffLineKind {
  if (
    line.startsWith("diff ") ||
    line.startsWith("index ") ||
    line.startsWith("---") ||
    line.startsWith("+++") ||
    line.startsWith("@@")
  ) {
    return "meta";
  }
  if (line.startsWith("+")) return "add";
  if (line.startsWith("-")) return "remove";
  return "context";
}

function textFor(line: string, kind: DiffLineKind): string {
  if (kind === "meta") return line;
  if (line.startsWith("+") || line.startsWith("-") || line.startsWith(" ")) return line.slice(1);
  return line;
}

/** Parse a unified diff. File headers and hunk ranges stay as `meta` lines. */
export function parseUnifiedDiff(source: string): readonly DiffLine[] {
  const raw = source.split(/\r?\n/);
  const lines: DiffLine[] = [];
  for (const [index, line] of raw.entries()) {
    if (line.length === 0 && index === raw.length - 1) continue;
    const kind = kindFor(line);
    const text = textFor(line, kind);
    lines.push({ kind, text, key: `${index}:${kind}:${text}` });
  }
  return lines;
}

/** Assign stable keys to structured lines a caller already classified. */
export function keyDiffLines(lines: readonly DiffLineInput[]): readonly DiffLine[] {
  return lines.map((line, index) => ({
    kind: line.kind,
    text: line.text,
    key: `${index}:${line.kind}:${line.text}`,
  }));
}
