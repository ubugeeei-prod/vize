import { graphemes } from "../utils/text.js";

/** Zero-based logical location in a multiline grapheme string. */
export interface TextAreaPoint {
  readonly line: number;
  readonly column: number;
}

function boundedPosition(value: string, position: number): number {
  const length = graphemes(value).length;
  return Math.max(0, Math.min(Number.isFinite(position) ? Math.trunc(position) : 0, length));
}

/** Convert a global grapheme position into its line and column. */
export function positionToTextAreaPoint(value: string, position: number): TextAreaPoint {
  const parts = graphemes(value);
  const end = boundedPosition(value, position);
  let line = 0;
  let column = 0;
  for (let index = 0; index < end; index++) {
    if (parts[index] === "\n") {
      line++;
      column = 0;
    } else {
      column++;
    }
  }
  return { line, column };
}

/** Convert a line and column into a clamped global grapheme position. */
export function textAreaPointToPosition(value: string, line: number, column: number): number {
  const targetLine = Math.max(0, Number.isFinite(line) ? Math.trunc(line) : 0);
  const targetColumn = Math.max(0, Number.isFinite(column) ? Math.trunc(column) : 0);
  const parts = graphemes(value);
  let currentLine = 0;
  let currentColumn = 0;

  for (let index = 0; index < parts.length; index++) {
    if (currentLine === targetLine && currentColumn >= targetColumn) return index;
    if (parts[index] === "\n") {
      if (currentLine === targetLine) return index;
      currentLine++;
      currentColumn = 0;
    } else {
      currentColumn++;
    }
  }
  return parts.length;
}

/** Find the start or end position of the line containing a global position. */
export function lineBoundary(value: string, position: number, edge: "start" | "end"): number {
  const point = positionToTextAreaPoint(value, position);
  return textAreaPointToPosition(value, point.line, edge === "start" ? 0 : Number.MAX_SAFE_INTEGER);
}

/** Move between logical lines while retaining the caller's preferred column. */
export function moveTextAreaCursorVertically(
  value: string,
  position: number,
  lineDelta: number,
  preferredColumn: number,
): number {
  const point = positionToTextAreaPoint(value, position);
  const lastLine = value.split("\n").length - 1;
  const targetLine = Math.max(0, Math.min(lastLine, point.line + Math.trunc(lineDelta)));
  return textAreaPointToPosition(value, targetLine, preferredColumn);
}
