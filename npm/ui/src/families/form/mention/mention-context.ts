import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import type { VirtualElement } from "../../overlays/positioner/positioner.ts";
import type { MentionFieldKind } from "./mention-caret.ts";
import type { MentionLoadStatus, MentionState } from "./mention-types.ts";

/** Registration input of one `MentionItem`. */
export interface MentionItemRegistration<T> {
  readonly id: ComputedRef<string>;
  readonly value: T;
  readonly element: MaybeRefOrGetter<Element | null | undefined>;
  readonly textValue: MaybeRefOrGetter<string | null | undefined>;
  readonly disabled: MaybeRefOrGetter<boolean | undefined>;
}

/**
 * Shared state for Mention compound parts.
 *
 * The root is generic over its item type; parts are generic independently,
 * so value-accepting members are methods, which keeps a root typed over `T`
 * assignable to this `unknown` context without casts. Every value a part
 * hands back comes from the root's items or an item authored for that root.
 */
export interface MentionContextValue<T> {
  readonly fieldId: ComputedRef<string>;
  readonly listboxId: ComputedRef<string>;
  readonly text: ComputedRef<string>;
  readonly query: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<MentionState>;
  readonly disabled: ComputedRef<boolean>;
  readonly empty: ComputedRef<boolean>;
  readonly status: ComputedRef<MentionLoadStatus>;
  readonly activeKey: ComputedRef<string | null>;
  readonly activeDescendant: ComputedRef<string | undefined>;
  readonly fieldElement: ShallowRef<HTMLElement | null>;
  readonly contentElement: ShallowRef<HTMLElement | null>;
  readonly caretReference: VirtualElement;
  attachField(element: HTMLElement | null, kind: MentionFieldKind): void;
  onFieldInput(event: Event): void;
  onFieldKeydown(event: KeyboardEvent): void;
  onFieldCaret(event: Event): void;
  onFieldBlur(event: FocusEvent): void;
  setOpen(open: boolean, event: Event | null): boolean;
  registerItem(input: MentionItemRegistration<T>): CollectionRegistration<string>;
  highlight(key: string): boolean;
  choose(value: T, event: Event | null): boolean;
}

export const mentionContext = createContext<MentionContextValue<unknown>>("Mention");
