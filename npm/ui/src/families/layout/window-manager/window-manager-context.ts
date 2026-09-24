import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  WindowBounds,
  WindowConstraints,
  WindowEdge,
  WindowEntry,
  WindowMode,
  WindowRect,
} from "./window-manager-model.ts";
import type { WindowSummary } from "./window-manager-types.ts";

/** Registration input of one window. */
export interface WindowRegistration {
  readonly id: string;
  readonly title: () => string;
  readonly defaultRect: () => WindowRect;
  readonly defaultMode: () => WindowMode;
}

/** Shared state between `WindowManager`, its windows, and docks. */
export interface WindowManagerContextValue {
  readonly register: (registration: WindowRegistration) => () => void;
  readonly entry: (id: string) => WindowEntry;
  readonly zIndex: (id: string) => number;
  readonly activeId: ComputedRef<string | null>;
  readonly bounds: ComputedRef<WindowBounds | null>;
  readonly windows: ComputedRef<readonly WindowSummary[]>;
  readonly keyboardStep: ComputedRef<number>;
  readonly focus: (id: string) => void;
  readonly move: (id: string, deltaX: number, deltaY: number, origin?: WindowRect) => void;
  readonly resize: (
    id: string,
    edge: WindowEdge,
    deltaX: number,
    deltaY: number,
    constraints: WindowConstraints,
    origin?: WindowRect,
  ) => void;
  readonly setMode: (id: string, mode: WindowMode) => void;
}

export const windowManagerContext = createContext<WindowManagerContextValue>("WindowManager");
