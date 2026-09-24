import { blockAt, caret, isCollapsed } from "./rich-text-model.ts";
import type { RichTextSchema } from "./rich-text-schema.ts";
import { createTransaction } from "./rich-text-state.ts";
import type { RichTextCommand, RichTextTransaction, RtState } from "./rich-text-state.ts";
import {
  createUntypedRichTextCommands,
  insertText,
  textBeforeCaret,
} from "./rich-text-commands.ts";
import { replaceRange } from "./rich-text-transform.ts";

/**
 * Transforms typed text. `match` runs against the textblock text before the
 * caret plus the inserted text and must end with `$`.
 */
export interface RichTextInputRule {
  readonly match: RegExp;
  /** Build the replacement for the matched text `[start, end)` (offsets before the new text). */
  readonly apply: (
    state: RtState,
    match: RegExpExecArray,
    start: number,
    end: number,
  ) => RichTextTransaction | null;
}

/** Remove the matched prefix, then run `command` on the result, as one undo step. */
export function blockInputRule(
  match: RegExp,
  command: (match: RegExpExecArray) => RichTextCommand,
): RichTextInputRule {
  return {
    match,
    apply: (state, result, start, end) => {
      const head = state.selection.head;
      const removed = replaceRange(
        state.schema,
        state.doc,
        { ...head, offset: start },
        { ...head, offset: end },
        [],
      );
      const intermediate = createTransaction(
        state,
        { doc: removed.doc, selection: caret(removed.position) },
        "input",
        {
          addToHistory: false,
        },
      ).state;
      const captured: { state: RtState | null } = { state: null };
      const applied = command(result)(intermediate, (transaction) => {
        captured.state = transaction.state;
      });
      if (!applied || captured.state === null) return null;
      return createTransaction(
        state,
        { doc: captured.state.doc, selection: captured.state.selection },
        "structure",
      );
    },
  };
}

/** Replace the matched text with its first capture group carrying a mark. */
export function markInputRule(match: RegExp, mark: string): RichTextInputRule {
  return {
    match,
    apply: (state, result, start, end) => {
      const text = result[1];
      if (!text || !Object.hasOwn(state.schema.marks, mark)) return null;
      const head = state.selection.head;
      const leading = result[0].length - result[0].trimStart().length;
      const from = { ...head, offset: start + leading };
      const replaced = replaceRange(state.schema, state.doc, from, { ...head, offset: end }, [
        {
          type: "text",
          text,
          marks: [{ type: mark, attrs: state.schema.marks[mark]?.defaults ?? {} }],
        },
      ]);
      return createTransaction(
        state,
        { doc: replaced.doc, selection: caret(replaced.position), storedMarks: [] },
        "format",
      );
    },
  };
}

/** Markdown-style input rules for every built-in node and mark the schema declares. */
export function defaultRichTextInputRules<Schema extends RichTextSchema>(
  schema: Schema,
): RichTextInputRule[] {
  const commands = createUntypedRichTextCommands(schema);
  const has = (name: string): boolean =>
    Object.hasOwn(schema.nodes, name) || Object.hasOwn(schema.marks, name);
  const rules: RichTextInputRule[] = [];
  if (has("heading")) {
    rules.push(
      blockInputRule(/^(#{1,6}) $/u, (match) =>
        commands.setBlockType("heading", { level: match[1]?.length ?? 1 }),
      ),
    );
  }
  if (has("blockquote")) rules.push(blockInputRule(/^> $/u, () => commands.wrapIn("blockquote")));
  if (has("bulletList"))
    rules.push(blockInputRule(/^[-*+] $/u, () => commands.wrapIn("bulletList")));
  if (has("orderedList")) {
    rules.push(
      blockInputRule(/^(\d{1,9})\. $/u, (match) =>
        commands.wrapIn("orderedList", { start: Number(match[1] ?? 1) }),
      ),
    );
  }
  if (has("codeBlock"))
    rules.push(blockInputRule(/^```$/u, () => commands.setBlockType("codeBlock")));
  if (has("bold")) rules.push(markInputRule(/\*\*([^*]+)\*\*$/u, "bold"));
  if (has("italic")) rules.push(markInputRule(/(?:^|\s)_([^_]+)_$/u, "italic"));
  if (has("code")) rules.push(markInputRule(/`([^`]+)`$/u, "code"));
  if (has("strike")) rules.push(markInputRule(/~~([^~]+)~~$/u, "strike"));
  return rules;
}

/** Insert text, letting the first matching input rule transform it instead. */
export function insertTextWithRules(
  text: string,
  rules: readonly RichTextInputRule[],
): RichTextCommand {
  return (state, dispatch) => {
    const head = state.selection.head;
    const block = blockAt(state.doc, head.path);
    const code = block !== undefined && state.schema.nodes[block.type]?.code === true;
    if (isCollapsed(state.selection) && !code) {
      const before = textBeforeCaret(state);
      const probe = before + text;
      for (const rule of rules) {
        const match = new RegExp(rule.match.source, rule.match.flags.replace("g", "")).exec(probe);
        if (!match) continue;
        const start = probe.length - match[0].length;
        if (start > before.length) continue;
        const transaction = rule.apply(state, match, start, head.offset);
        if (!transaction) continue;
        dispatch?.(transaction);
        return true;
      }
    }
    return insertText(text)(state, dispatch);
  };
}
