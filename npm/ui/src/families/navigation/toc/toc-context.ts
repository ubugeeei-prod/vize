import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";

/** Shared state for TOC compound parts. */
export interface TocContextValue {
  readonly activeId: ComputedRef<string | null>;
  readonly registerLink: (input: {
    readonly key: string;
    readonly targetId: string;
    readonly element: Readonly<ShallowRef<HTMLAnchorElement | null>>;
  }) => CollectionRegistration<string>;
  readonly navigate: (targetId: string, event: MouseEvent) => void;
}

export const tocContext = createContext<TocContextValue>("Toc");

/** Link state shared from a TocItem to its nested TocLink. */
export interface TocItemContextValue {
  readonly setTargetId: (targetId: string | null) => void;
}

export const tocItemContext = createContext<TocItemContextValue>("TocItem");
