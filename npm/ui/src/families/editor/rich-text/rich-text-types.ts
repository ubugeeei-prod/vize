import type { RichTextSchema } from "./rich-text-schema.ts";
import type { RichTextCommand, RichTextState } from "./rich-text-state.ts";

/** Props of the RichTextRoot default slot. */
export interface RichTextSlotProps<Schema extends RichTextSchema> {
  /** Current editor state. */
  readonly state: RichTextState<Schema>;

  /** Whether editing is enabled. */
  readonly editable: boolean;

  /** Whether the content has focus. */
  readonly focused: boolean;

  /** Run a command; returns whether it applied. */
  readonly run: (command: RichTextCommand) => boolean;
}

/** Imperative surface of RichTextRoot. */
export interface RichTextRootExpose<Schema extends RichTextSchema> {
  /** Current editor state. */
  readonly state: RichTextState<Schema>;

  /** Run a command; returns whether it applied. */
  readonly run: (command: RichTextCommand) => boolean;

  /** Focus the editable content. */
  readonly focus: () => void;

  /** Serialize the document to sanitized HTML. */
  readonly toHtml: () => string;
}

/** State exposed to toolbar button slots. */
export interface RichTextToolbarButtonSlotProps {
  /** Whether the button's `active` predicate holds. */
  readonly active: boolean;

  /** Whether the command cannot run now. */
  readonly disabled: boolean;
}

/** Imperative surface of RichTextContent. */
export interface RichTextContentExpose {
  /** Rendered editable element. */
  readonly element: HTMLDivElement | null;

  /** Focus the editable element. */
  readonly focus: () => void;
}
