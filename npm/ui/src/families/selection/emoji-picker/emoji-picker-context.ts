import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { GridMove } from "../listbox-grid/listbox-grid-model.ts";
import type { EmojiPickerSection, EmojiSkinTone } from "./emoji-picker-model.ts";

/**
 * Shared EmojiPicker state. Value-accepting members are methods so a root
 * typed over `T` stays assignable to the `unknown` context.
 */
export interface EmojiPickerContextValue<T> {
  readonly baseId: ComputedRef<string>;
  readonly gridId: ComputedRef<string>;
  readonly columns: ComputedRef<number>;
  readonly sections: ComputedRef<readonly EmojiPickerSection<T>[]>;
  readonly activeIndex: ComputedRef<number>;
  readonly activeItem: ComputedRef<T | undefined>;
  readonly skinTone: ComputedRef<EmojiSkinTone>;
  readonly search: ComputedRef<string>;
  readonly empty: ComputedRef<boolean>;
  readonly totalRows: ComputedRef<number>;
  glyphOf(item: T): string;
  nameOf(item: T): string;
  cellId(section: number, index: number): string;
  labelId(section: number): string;
  virtualIndex(section: number, index: number): number;
  setActive(virtualIndex: number): void;
  move(move: GridMove): boolean;
  selectActive(event: Event): boolean;
  select(item: T, event: Event | null): void;
  setSkinTone(tone: EmojiSkinTone): void;
  setSearch(text: string): void;
  registerGrid(focus: () => void): () => void;
  focusGrid(): void;
}

export const emojiPickerContext = createContext<EmojiPickerContextValue<unknown>>("EmojiPicker");

/** Section position shared by one category with its items. */
export interface EmojiPickerCategoryContextValue {
  readonly section: ComputedRef<number>;
}

export const emojiPickerCategoryContext =
  createContext<EmojiPickerCategoryContextValue>("EmojiPickerCategory");
