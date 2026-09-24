import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  TagsInputAriaInvalid,
  TagsInputDirection,
  TagsInputItemSlotState,
  TagsInputRemoveSource,
} from "./tags-input-types.ts";

/**
 * Shared TagsInput state. Tag values are erased to `unknown` at the context
 * boundary because root and item generics cannot be linked through Vue's
 * provide/inject; every value crossing it originates from the root's own list.
 */
export interface TagsInputContextValue {
  readonly inputId: ComputedRef<string>;
  readonly tags: ComputedRef<readonly unknown[]>;
  readonly inputValue: ShallowRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly readonly: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly editable: ComputedRef<boolean>;
  readonly direction: ComputedRef<TagsInputDirection>;
  readonly ariaInvalid: ComputedRef<Exclude<TagsInputAriaInvalid, false> | undefined>;
  readonly ariaLabel: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly editingIndex: ShallowRef<number | null>;
  readonly activeIndex: ShallowRef<number | null>;
  readonly tagText: (tag: unknown) => string;
  readonly itemId: (index: number) => string;
  readonly focusItem: (index: number) => boolean;
  readonly focusInput: () => void;
  readonly removeAt: (index: number, source: TagsInputRemoveSource) => boolean;
  readonly commitInput: (source: "blur" | "delimiter" | "enter") => boolean;
  readonly handleInputText: (text: string) => void;
  readonly handlePaste: (pasted: string, selectionStart: number, selectionEnd: number) => boolean;
  readonly isDelimiterKey: (key: string) => boolean;
  readonly addOnBlur: ComputedRef<boolean>;
  readonly startEdit: (index: number) => boolean;
  readonly commitEdit: (index: number, text: string, keepEditingOnInvalid: boolean) => boolean;
  readonly cancelEdit: (index: number) => void;
}

export const tagsInputContext = createContext<TagsInputContextValue>("TagsInput");

/** Per-item state shared with TagsInputItemText and TagsInputItemDelete. */
export interface TagsInputItemContextValue {
  readonly slotState: ComputedRef<TagsInputItemSlotState<unknown>>;
  readonly remove: (source: TagsInputRemoveSource) => boolean;
}

export const tagsInputItemContext = createContext<TagsInputItemContextValue>("TagsInputItem");
