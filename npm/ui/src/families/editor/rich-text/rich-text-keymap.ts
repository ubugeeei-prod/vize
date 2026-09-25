import { createUntypedRichTextCommands } from "./rich-text-commands.ts";
import type { RichTextSchema } from "./rich-text-schema.ts";
import type { RichTextCommand } from "./rich-text-state.ts";

/** Key bindings: `Mod` is Ctrl or Cmd; modifiers are ordered `Mod-Alt-Shift-`. */
export type RichTextKeymap = Readonly<Record<string, RichTextCommand>>;

/** Normalized binding name of a keyboard event, e.g. `Mod-Shift-z`. */
export function richTextKeyName(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push("Mod");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  // Prefer the physical digit for Alt/Shift+digit chords whose `key` is a symbol.
  const digit = /^Digit(\d)$/u.exec(event.code)?.[1];
  const key =
    digit !== undefined && parts.length > 0
      ? digit
      : event.key.length === 1
        ? event.key.toLowerCase()
        : event.key;
  parts.push(key);
  return parts.join("-");
}

/** Standard shortcuts for every built-in node and mark the schema declares. */
export function defaultRichTextKeymap<Schema extends RichTextSchema>(
  schema: Schema,
): RichTextKeymap {
  const commands = createUntypedRichTextCommands(schema);
  const has = (name: string): boolean =>
    Object.hasOwn(schema.nodes, name) || Object.hasOwn(schema.marks, name);
  const keymap: Record<string, RichTextCommand> = {
    "Mod-z": commands.undo,
    "Mod-Shift-z": commands.redo,
    "Mod-y": commands.redo,
    "Mod-a": commands.selectAll,
    Enter: commands.splitBlock,
    Backspace: commands.deleteBackward("character"),
    "Alt-Backspace": commands.deleteBackward("word"),
    "Mod-Backspace": commands.deleteBackward("word"),
    Delete: commands.deleteForward("character"),
    "Alt-Delete": commands.deleteForward("word"),
  };
  const marks: [string, string][] = [
    ["bold", "Mod-b"],
    ["italic", "Mod-i"],
    ["underline", "Mod-u"],
    ["strike", "Mod-Shift-x"],
    ["code", "Mod-e"],
  ];
  for (const [mark, key] of marks) if (has(mark)) keymap[key] = commands.toggleMark(mark);
  if (has("hardBreak")) keymap["Shift-Enter"] = commands.insertInline("hardBreak");
  keymap["Mod-Alt-0"] = commands.setBlockType(schema.defaultBlock);
  if (has("heading")) {
    for (let level = 1; level <= 6; level++) {
      keymap[`Mod-Alt-${level}`] = commands.toggleBlockType("heading", { level });
    }
  }
  if (has("codeBlock")) keymap["Mod-Alt-c"] = commands.toggleBlockType("codeBlock");
  if (has("orderedList")) keymap["Mod-Shift-7"] = commands.toggleWrap("orderedList");
  if (has("bulletList")) keymap["Mod-Shift-8"] = commands.toggleWrap("bulletList");
  if (has("blockquote")) keymap["Mod-Shift-9"] = commands.toggleWrap("blockquote");
  return keymap;
}
