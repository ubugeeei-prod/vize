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

/** Hit-test one normalized mouse packet and route it through the host tree. */
export function routeMouseInput(
  renderer: MouseInputRenderer,
  root: FrescoHostNode,
  event: MouseEvent,
  layouts: readonly LayoutResultNapi[],
): void {
  const target = renderer.hitTest(root, event.x, event.y, layouts);
  if (target) renderer.dispatchInput(target.id, event);
}

/** Route first, then publish the packet even when a routed handler throws. */
export function routeAndPublishMouseInput(
  renderer: MouseInputRenderer,
  root: FrescoHostNode | null,
  event: MouseEvent,
  layouts: readonly LayoutResultNapi[],
  sink: MouseEventSink,
): void {
  try {
    if (root) routeMouseInput(renderer, root, event, layouts);
  } finally {
    sink.value = event;
  }
}
