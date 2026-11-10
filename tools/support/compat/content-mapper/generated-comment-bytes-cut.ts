/**
 * The generated-comment byte cut as a pure text edit, for certifying captured
 * projections: debug-only `@vize-map: expr|prop|handler|component` lines leave,
 * and the reasons the emitters stopped writing leave their `@ts-ignore` lines.
 * Kept `@vize-map: a:b -> c:d` comments carry generated UTF-8 byte offsets, so
 * they move exactly as the emitter would place them.
 */

/** Reasons the emitters no longer write; helper declarations keep theirs. */
const removedReasons = new Set([
  "Inference-only callback prop; mapped prop owner checks diagnostics.",
  "Inference-only constructor; authored props checks own diagnostics.",
  "Inference-only listener value; the typed event scope owns diagnostics.",
  "Inference-only callback branch.",
  "Inference-only nested callback parameter.",
  "TS2694/TS2307: a `vue` without `ComponentPublicInstance` must degrade template context extras to unchecked, never error.",
  "TS6196: SFC generic may be used by emits or slots.",
  "TS2345: the authored withDefaults call reports an invalid argument.",
  "Inference-only call; the authored prop checker owns diagnostics.",
  "Inference-only guard and call; the root's mapped prop checks own diagnostics.",
  "Inference-only call; the root's mapped prop checks own diagnostics.",
  "Inference-only callback prop; the mapped prop owner checks diagnostics.",
  "Inference-only factory; called before the guard below.",
  "Probe leaf errors are owned by the canonical component-props call.",
  "Inference-only guard; authored binding checks own diagnostics.",
  "Inference-only dynamic target/props; the canonical component-props call owns diagnostics.",
  "Inference-only prop; the canonical component-props call owns diagnostics.",
  "Navigation-only reference; the authored v-bind statement owns diagnostics.",
  "Inference-only alias; authored v-if and v-bind statements own diagnostics.",
  "Inference-only alias; the authored v-bind statement owns diagnostics.",
  "Inference-only guard; the authored v-if statement owns diagnostics.",
  "Inference-only call; the usage's mapped prop checks own diagnostics.",
  "Inference-only guard; the authored binding checks own diagnostics.",
]);
/** `{purpose}; authored v-if checks own diagnostics.` for every guard purpose. */
const guardReason = /^[A-Za-z -]+; authored v-if checks own diagnostics\.$/u;
const token =
  /(?<map>^[ \t]*\/\/ @vize-map: (?:expr|prop|handler|component) -> \d+:\d+\r?\n)|(?<ignore>\/\/ @ts-ignore (?<reason>[^\n\r]*))|(?<gen>\/\/ @vize-map: (?<a>\d+):(?<b>\d+) -> )/gmu;

type Edit = {
  /** UTF-16 range in the previous text. */
  start: number;
  end: number;
  /** UTF-8 range in the previous text. */
  byteStart: number;
  byteEnd: number;
  text: string;
  replacement: (relocate: (offset: number) => number) => string;
};

export type CommentCut = {
  text: string;
  /** Previous UTF-16 offset to its successor offset; removed text maps to its cut point. */
  utf16: (offset: number) => number;
  /** Previous UTF-8 byte offset to its successor offset, like `utf16`. */
  bytes: (offset: number) => number;
  /** The byte edits on the previous text, in order. */
  edits: Array<{ start: number; end: number; replacement: Buffer }>;
};

export function generatedCommentBytesCut(previous: string): CommentCut {
  const edits: Edit[] = [];
  let unitCursor = 0;
  let byteCursor = 0;
  for (const match of previous.matchAll(token)) {
    byteCursor += Buffer.byteLength(previous.slice(unitCursor, match.index));
    const whole = match[0];
    const groups = match.groups!;
    const byteStart = byteCursor;
    byteCursor += Buffer.byteLength(whole);
    unitCursor = match.index + whole.length;
    let replacement: Edit["replacement"];
    if (groups.map !== undefined) replacement = () => "";
    else if (groups.ignore !== undefined) {
      const reason = groups.reason!;
      if (!removedReasons.has(reason) && !guardReason.test(reason)) continue;
      replacement = () => "// @ts-ignore";
    } else {
      const start = Number(groups.a);
      const end = Number(groups.b);
      replacement = (relocate) => `// @vize-map: ${relocate(start)}:${relocate(end)} -> `;
    }
    edits.push({
      start: match.index,
      end: unitCursor,
      byteStart,
      byteEnd: byteCursor,
      text: whole,
      replacement,
    });
  }
  // A relocated offset can change a map comment's own width, which moves every
  // later offset, so iterate to the fixed point (widths settle immediately).
  let texts = edits.map((edit) => (edit.text.startsWith("// @vize-map: ") ? edit.text : ""));
  for (let round = 0; ; round++) {
    const relocate = (offset: number) => {
      let delta = 0;
      edits.forEach((edit, index) => {
        if (edit.byteEnd <= offset)
          delta += Buffer.byteLength(texts[index]!) - (edit.byteEnd - edit.byteStart);
        else if (edit.byteStart < offset)
          throw new Error("a map offset points inside a cut comment");
      });
      return offset + delta;
    };
    const next = edits.map((edit) => edit.replacement(relocate));
    if (next.every((text, index) => text === texts[index])) break;
    if (round > 8) throw new Error("map comment widths did not settle");
    texts = next;
  }
  let text = "";
  let cursor = 0;
  edits.forEach((edit, index) => {
    text += previous.slice(cursor, edit.start) + texts[index];
    cursor = edit.end;
  });
  text += previous.slice(cursor);
  const utf16 = (offset: number) => {
    let delta = 0;
    for (const [index, edit] of edits.entries()) {
      if (edit.end <= offset) delta += texts[index]!.length - (edit.end - edit.start);
      else if (edit.start < offset) return edit.start + delta;
    }
    return offset + delta;
  };
  const bytes = (offset: number) => {
    let delta = 0;
    for (const [index, edit] of edits.entries()) {
      if (edit.byteEnd <= offset)
        delta += Buffer.byteLength(texts[index]!) - (edit.byteEnd - edit.byteStart);
      else if (edit.byteStart < offset) return edit.byteStart + delta;
    }
    return offset + delta;
  };
  return {
    text,
    utf16,
    bytes,
    edits: edits
      .map((edit, index) => ({
        start: edit.byteStart,
        end: edit.byteEnd,
        replacement: Buffer.from(texts[index]!),
      }))
      .filter(
        (edit) => !edit.replacement.equals(Buffer.from(previous).subarray(edit.start, edit.end)),
      ),
  };
}
