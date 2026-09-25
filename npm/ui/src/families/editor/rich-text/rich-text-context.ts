import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { RichTextUntypedCommands } from "./rich-text-commands.ts";
import type { RichTextSelection } from "./rich-text-model.ts";
import type { RichTextCommand, RtState } from "./rich-text-state.ts";

/** Editor services shared by the rich-text parts (schema-agnostic view). */
export interface RichTextContextValue {
  readonly id: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly state: Readonly<{ readonly value: RtState }>;
  readonly editable: ComputedRef<boolean>;
  readonly focused: ShallowRef<boolean>;
  readonly contentElement: ShallowRef<HTMLElement | null>;
  readonly commands: RichTextUntypedCommands;
  /** Insert typed text through the editor's input rules; returns whether it applied. */
  readonly typeText: (text: string) => boolean;
  /** Run the editor keymap binding for a key name (see `richTextKeyName`). */
  readonly runKey: (name: string) => boolean;
  /** Run a command against the current state; returns whether it applied. */
  readonly run: (command: RichTextCommand) => boolean;
  /** Whether a command would apply now. */
  readonly can: (command: RichTextCommand) => boolean;
  /** Replace the selection without changing the document. */
  readonly select: (selection: RichTextSelection) => void;
  /** Focus the editable content. */
  readonly focus: () => void;
  /** Toolbars register so Alt+F10 can move focus to the first one. */
  readonly toolbars: ShallowRef<readonly (() => boolean)[]>;
}

export const richTextContext = createContext<RichTextContextValue>("RichText");

/** Roving focus shared by a toolbar and its buttons. */
export interface RichTextToolbarContextValue {
  readonly registry: CollectionRegistry<string, null>;
}

export const richTextToolbarContext = createContext<RichTextToolbarContextValue>("RichTextToolbar");
