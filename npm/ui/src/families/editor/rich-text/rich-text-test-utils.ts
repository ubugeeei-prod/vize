import { createDefaultRichTextSchema } from "./rich-text-schema.ts";
import type { RichTextDefaultSchema } from "./rich-text-schema.ts";
import { assertRichTextDoc, textContent } from "./rich-text-model.ts";
import type {
  RichTextDoc,
  RichTextPosition,
  RtBlock,
  RtInline,
  RtMark,
} from "./rich-text-model.ts";
import { createRichTextState, toRichTextState } from "./rich-text-state.ts";
import type { RichTextCommand, RichTextState } from "./rich-text-state.ts";
import { richTextToHtml } from "./rich-text-html.ts";

export const schema = createDefaultRichTextSchema();

export type TestState = RichTextState<RichTextDefaultSchema>;

/** Text run. */
export function t(text: string, ...marks: (string | RtMark)[]): RtInline {
  return {
    type: "text",
    text,
    marks: marks.map((mark) =>
      typeof mark === "string"
        ? {
            type: mark,
            attrs: Object.entries(schema.marks).find(([name]) => name === mark)?.[1].defaults ?? {},
          }
        : mark,
    ),
  };
}

/** Paragraph. */
export function p(...content: (string | RtInline)[]): RtBlock {
  return {
    type: "paragraph",
    attrs: {},
    content: content.map((part) => (typeof part === "string" ? t(part) : part)),
  };
}

/** Heading. */
export function h(level: number, text: string): RtBlock {
  return { type: "heading", attrs: { level }, content: text ? [t(text)] : [] };
}

/** Blockquote. */
export function quote(...content: RtBlock[]): RtBlock {
  return { type: "blockquote", attrs: {}, content };
}

/** Bullet list of items (each item holds the given blocks). */
export function ul(...items: RtBlock[][]): RtBlock {
  return {
    type: "bulletList",
    attrs: {},
    content: items.map((blocks) => ({ type: "listItem", attrs: {}, content: blocks })),
  };
}

/** Document. */
export function doc(...content: RtBlock[]): RichTextDoc<RichTextDefaultSchema> {
  return assertRichTextDoc(schema, { type: "doc", content });
}

/** Position helper. */
export function at(path: number[], offset: number): RichTextPosition {
  return { path, offset };
}

/** State with a selection. */
export function state(
  document: RichTextDoc<RichTextDefaultSchema>,
  anchor: RichTextPosition,
  head: RichTextPosition = anchor,
): TestState {
  return createRichTextState(schema, { doc: document, selection: { anchor, head } });
}

/** Run a command and return the next state (or `null` when it does not apply). */
export function run(current: TestState, command: RichTextCommand): TestState | null {
  const captured: { state: TestState | null } = { state: null };
  const applied = command(current, (transaction) => {
    captured.state = toRichTextState(schema, transaction.state);
  });
  return applied ? captured.state : null;
}

/** Run a command that must apply. */
export function apply(current: TestState, command: RichTextCommand): TestState {
  const next = run(current, command);
  if (!next) throw new Error("command did not apply");
  return next;
}

/** Compact HTML of a state's document. */
export function html(current: TestState | RichTextDoc<RichTextDefaultSchema>): string {
  return richTextToHtml(schema, "doc" in current ? current.doc : current);
}

/** Plain text of a document. */
export function text(current: TestState): string {
  return textContent(current.doc);
}
