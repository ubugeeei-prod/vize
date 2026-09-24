import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DrawerSide, DrawerSnapPoint } from "./drawer-types.ts";

/** Drawer-only state layered on top of the shared Dialog context. */
export interface DrawerContextValue {
  readonly side: ComputedRef<DrawerSide>;
  readonly snapPoints: ComputedRef<readonly DrawerSnapPoint[]>;
  readonly activeSnapPoint: ComputedRef<DrawerSnapPoint | null>;
  readonly dismissible: ComputedRef<boolean>;
  readonly dragging: Readonly<ShallowRef<boolean>>;
  readonly dragOffset: Readonly<ShallowRef<number>>;
  readonly snapOffset: ComputedRef<number>;
  readonly size: ShallowRef<number>;
  readonly dialogElement: ShallowRef<HTMLDialogElement | null>;
  readonly measure: () => void;
  readonly snapTo: (snapPoint: DrawerSnapPoint, event?: Event | null) => boolean;
  readonly stepSnapPoint: (direction: 1 | -1, wrap: boolean, event?: Event | null) => boolean;
  readonly startDrag: (event: PointerEvent) => boolean;
  readonly moveDrag: (event: PointerEvent) => void;
  readonly endDrag: (event: PointerEvent, cancelled: boolean) => boolean;
}

export const drawerContext = createContext<DrawerContextValue>("Drawer");
