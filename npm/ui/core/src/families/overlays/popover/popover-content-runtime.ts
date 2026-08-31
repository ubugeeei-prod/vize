import { onMounted, onUnmounted, watch } from "vue";
import type { ShallowRef } from "vue";

import type { DismissableLayerController } from "../dismissable-layer/dismissable-layer.ts";
import type { FocusGuardsController } from "../../../focus-guards.ts";
import type { FocusScopeController } from "../../../focus-scope.ts";
import type { InertOutsideController } from "../../../inert-outside.ts";
import type {
  Placement,
  PlacementAlign,
  PlacementSide,
  PositionerStrategy,
  Rect,
} from "../positioner/positioner.ts";
import type { ScrollLockController } from "../../../scroll-lock.ts";
import type { PopoverContextValue } from "./popover-context.ts";
import type { PopoverContentSlotState, PopoverState } from "./popover-types.ts";

/** Lifecycle values managed after PopoverContent creates its document controllers. */
export interface PopoverContentLifecycleOptions {
  readonly context: PopoverContextValue;
  readonly dismissableLayer: DismissableLayerController;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly focusGuards: FocusGuardsController;
  readonly focusScope: FocusScopeController;
  readonly inertOutside: InertOutsideController;
  readonly ownerDocument: ShallowRef<Document | null>;
  readonly scrollLock: ScrollLockController;
}

export function usePopoverContentLifecycle(values: PopoverContentLifecycleOptions): void {
  const {
    context,
    dismissableLayer,
    element,
    focusGuards,
    focusScope,
    inertOutside,
    ownerDocument,
    scrollLock,
  } = values;
  let mounted = false;

  function activateControllers(): void {
    scrollLock.activate();
    inertOutside.activate();
    dismissableLayer.activate();
    focusGuards.activate();
    focusScope.activate();
  }

  function deactivateControllers(): void {
    dismissableLayer.deactivate();
    focusGuards.deactivate();
    inertOutside.deactivate();
    scrollLock.deactivate();
    focusScope.deactivate();
  }

  function syncControllers(): void {
    ownerDocument.value = element.value?.ownerDocument ?? null;
    if (!mounted || !context.open.value || !element.value) {
      deactivateControllers();
      return;
    }
    activateControllers();
  }

  watch(
    element,
    (next, previous) => {
      if (previous && context.contentElement.value === previous)
        context.contentElement.value = null;
      if (next) context.contentElement.value = next;
      syncControllers();
    },
    { flush: "post" },
  );
  watch(() => context.open.value, syncControllers, { flush: "post" });

  onMounted(() => {
    mounted = true;
    syncControllers();
  });

  onUnmounted(() => {
    mounted = false;
    deactivateControllers();
    dismissableLayer.dispose();
    focusGuards.dispose();
    focusScope.dispose();
    inertOutside.dispose();
    scrollLock.dispose();
    if (context.contentElement.value === element.value) context.contentElement.value = null;
  });
}

/** Positioner props assembled by PopoverContent from public props and shared context. */
export interface PopoverPositionerProps {
  readonly reference: HTMLButtonElement | null;
  readonly placement: Placement;
  readonly strategy: PositionerStrategy;
  readonly offset: number;
  readonly collisionPadding: number;
  readonly arrowPadding: number;
  readonly direction: "ltr" | "rtl";
  readonly flip: boolean;
  readonly shift: boolean;
  readonly size: boolean;
  readonly safeArea: boolean;
  readonly hide: boolean;
  readonly updateOnScroll: boolean;
  readonly updateOnResize: boolean;
  readonly viewport?: Rect;
}

/** Public PopoverContent props that map directly to Positioner. */
export type PopoverPositionerOptions = Omit<PopoverPositionerProps, "viewport">;

/** Context fields needed to create the PopoverContent default slot state. */
export interface PopoverSlotStateContext {
  readonly disabled: boolean;
  readonly modal: boolean;
  readonly open: boolean;
  readonly state: PopoverState;
}

export function withViewport(
  options: PopoverPositionerOptions,
  viewport: Rect | undefined,
): PopoverPositionerProps {
  return viewport === undefined ? options : { ...options, viewport };
}

export function resolvePopoverSlotPlacement(value: unknown, fallback: Placement): Placement {
  return typeof value === "string" ? (value as Placement) : fallback;
}

export function popoverSideFromPlacement(value: Placement): PlacementSide {
  return value.split("-", 1)[0] as PlacementSide;
}

export function popoverAlignFromPlacement(value: Placement): PlacementAlign {
  const [, align] = value.split("-");
  return (align ?? "center") as PlacementAlign;
}

export function createPopoverSlotState(
  placement: Placement,
  context: PopoverSlotStateContext,
): PopoverContentSlotState {
  return {
    align: popoverAlignFromPlacement(placement),
    disabled: context.disabled,
    modal: context.modal,
    open: context.open,
    placement,
    side: popoverSideFromPlacement(placement),
    state: context.state,
  };
}

export function existingPopoverElements(
  ...values: readonly (Element | null | undefined)[]
): Element[] {
  return values.filter((value): value is Element => value != null);
}
