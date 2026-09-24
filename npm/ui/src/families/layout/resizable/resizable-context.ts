import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { ResizableConstraints } from "./resizable-geometry.ts";
import type {
  ResizableDirection,
  ResizablePhysicalEdge,
  ResizableSize,
  ResizableSource,
  ResizableState,
} from "./resizable-types.ts";

/** Shared state and actions for Resizable handles. */
export interface ResizableContextValue {
  readonly id: ComputedRef<string>;
  readonly size: ComputedRef<ResizableSize>;
  readonly state: ComputedRef<ResizableState>;
  readonly constraints: ComputedRef<ResizableConstraints>;
  readonly disabled: ComputedRef<boolean>;
  readonly dir: ComputedRef<ResizableDirection>;
  readonly step: ComputedRef<number>;
  readonly largeStep: ComputedRef<number>;
  readonly begin: (
    edge: ResizablePhysicalEdge,
    source: ResizableSource,
    event: Event | null,
  ) => void;
  readonly update: (
    target: ResizableSize,
    event: Event | null,
    driver?: "height" | "width",
  ) => void;
  readonly end: (event: Event | null) => void;
  readonly initialSize: () => ResizableSize | null;
}

export const resizableContext = createContext<ResizableContextValue>("Resizable");
