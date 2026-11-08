/** Compile-only assertions for the public Fresco devtools contract. */

import {
  createFrescoDevtools,
  formatFrescoDebugOverlay,
  inspectFrescoTree,
  type FrescoDevtoolsEvent,
  type FrescoDevtoolsSnapshot,
  type FrescoHostNode,
  type FrescoTreeInspection,
} from "./index.js";

export const devtools = createFrescoDevtools({ eventLimit: 50 });
export const devtoolsSnapshot: FrescoDevtoolsSnapshot = devtools.snapshot();
export const overlay: string = formatFrescoDebugOverlay(devtoolsSnapshot, {
  maxNodes: 10,
  maxEvents: 5,
});

declare const hostNode: FrescoHostNode;
export const inspectedTree: FrescoTreeInspection = inspectFrescoTree(hostNode, [
  { id: hostNode.id, x: 0, y: 0, width: 80, height: 24 },
]);

export function exhaustivelyNarrowDevtoolsEvent(event: FrescoDevtoolsEvent): string {
  switch (event.type) {
    case "create":
    case "remove":
      return event.nodeType;
    case "update":
      return event.updateKind;
    case "resize":
      return String(event.current.width);
    case "input":
      return event.input.type;
    case "render":
      return String(event.metrics.renderTime);
    default: {
      const exhaustive: never = event;
      return exhaustive;
    }
  }
}

// @ts-expect-error - event limits are numeric.
createFrescoDevtools({ eventLimit: "all" });

// @ts-expect-error - native layout ids are numeric.
inspectFrescoTree(hostNode, [{ id: "root", x: 0, y: 0, width: 80, height: 24 }]);
