import type { LayoutResultNapi } from "@vizejs/fresco-native";

import type { MouseEvent } from "./protocol.js";
import type { FrescoHostNode } from "./renderer.js";

interface MouseInputRenderer {
  hitTest(
    root: FrescoHostNode,
    x: number,
    y: number,
    layouts: readonly LayoutResultNapi[],
  ): FrescoHostNode | undefined;
  dispatchInput(targetId: number, event: MouseEvent): unknown;
}

interface MouseEventSink {
  value: MouseEvent | null;
}

/** Per-application pointer target retained between normalized mouse packets. */
export interface MouseRoutingState {
  hoveredTargetId: number | undefined;
}

/** Create isolated pointer-routing state for one mounted Fresco application. */
export function createMouseRoutingState(): MouseRoutingState {
  return { hoveredTargetId: undefined };
}

const mountedRootStates = new WeakMap<FrescoHostNode, MouseRoutingState>();

function stateForMountedRoot(root: FrescoHostNode): MouseRoutingState {
  const existing = mountedRootStates.get(root);
  if (existing) return existing;

  const state = createMouseRoutingState();
  mountedRootStates.set(root, state);
  return state;
}

function pointerTransition(event: MouseEvent, kind: "enter" | "leave"): MouseEvent {
  return { ...event, kind };
}

function routePointerTransitions(
  renderer: MouseInputRenderer,
  target: FrescoHostNode | undefined,
  event: MouseEvent,
  state: MouseRoutingState,
  publish?: (event: MouseEvent) => void,
): void {
  if (event.kind !== "moved" && event.kind !== "drag") return;
  const nextTargetId = target?.id;
  if (state.hoveredTargetId === nextTargetId) return;

  const previousTargetId = state.hoveredTargetId;
  state.hoveredTargetId = nextTargetId;
  if (previousTargetId !== undefined) {
    const leave = pointerTransition(event, "leave");
    try {
      renderer.dispatchInput(previousTargetId, leave);
    } finally {
      publish?.(leave);
    }
  }
  if (nextTargetId !== undefined) {
    const enter = pointerTransition(event, "enter");
    try {
      renderer.dispatchInput(nextTargetId, enter);
    } finally {
      publish?.(enter);
    }
  }
}

/** Hit-test one normalized mouse packet and route it through the host tree. */
export function routeMouseInput(
  renderer: MouseInputRenderer,
  root: FrescoHostNode,
  event: MouseEvent,
  layouts: readonly LayoutResultNapi[],
  state?: MouseRoutingState,
  publishTransition?: (event: MouseEvent) => void,
): void {
  const target = renderer.hitTest(root, event.x, event.y, layouts);
  if (state) routePointerTransitions(renderer, target, event, state, publishTransition);
  if (target) renderer.dispatchInput(target.id, event);
}

/** Route transitions and the native packet, publishing each even when a handler throws. */
export function routeAndPublishMouseInput(
  renderer: MouseInputRenderer,
  root: FrescoHostNode | null,
  event: MouseEvent,
  layouts: readonly LayoutResultNapi[],
  sink: MouseEventSink,
  state?: MouseRoutingState,
): void {
  try {
    if (root) {
      routeMouseInput(
        renderer,
        root,
        event,
        layouts,
        state ?? stateForMountedRoot(root),
        (transition) => {
          sink.value = transition;
        },
      );
    }
  } finally {
    sink.value = event;
  }
}
