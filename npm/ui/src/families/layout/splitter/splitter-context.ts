import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  SplitterDirection,
  SplitterOrientation,
  SplitterPanelConstraints,
  SplitterResizeReason,
} from "./splitter-types.ts";

/** Reactive panel data registered with the owning group. */
export interface SplitterPanelRegistrationInput {
  readonly id: string;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly defaultSize: () => number | undefined;
  readonly constraints: () => SplitterPanelConstraints;
  readonly order: () => number | undefined;
}

/** Reactive handle data registered with the owning group. */
export interface SplitterHandleRegistrationInput {
  readonly id: string;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly order: () => number | undefined;
}

/** Shared state and actions for the Splitter compound parts. */
export interface SplitterContextValue {
  readonly id: ComputedRef<string>;
  readonly orientation: ComputedRef<SplitterOrientation>;
  readonly dir: ComputedRef<SplitterDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly keyboardStep: ComputedRef<number>;
  readonly draggingHandle: ComputedRef<string | null>;
  readonly registerPanel: (input: SplitterPanelRegistrationInput) => CollectionRegistration<string>;
  readonly registerHandle: (
    input: SplitterHandleRegistrationInput,
  ) => CollectionRegistration<string>;
  readonly getPanelIndex: (id: string) => number;
  readonly getPanelSize: (id: string) => number;
  readonly getPanelConstraints: (index: number) => SplitterPanelConstraints | undefined;
  readonly getPanelId: (index: number) => string | undefined;
  readonly getHandleIndex: (id: string) => number;
  readonly getSize: (index: number) => number;
  readonly resizeHandle: (index: number, delta: number, reason: SplitterResizeReason) => boolean;
  readonly setPanelSize: (index: number, size: number, reason: SplitterResizeReason) => boolean;
  readonly collapsePanel: (index: number, reason: SplitterResizeReason) => boolean;
  readonly expandPanel: (index: number, reason: SplitterResizeReason) => boolean;
  readonly startDrag: (handleId: string, event: PointerEvent) => void;
}

export const splitterContext = createContext<SplitterContextValue>("Splitter");
