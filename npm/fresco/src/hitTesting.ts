import type { LayoutResultNapi } from "@vizejs/fresco-native";

import { normalizeFrescoStyle } from "./renderPayload.js";
import type { FrescoHostNode } from "./renderer.js";

interface HitRegion {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

function clipsOverflow(value: unknown): boolean {
  return value === "hidden" || value === "scroll";
}

function clipsDescendants(node: FrescoHostNode): boolean {
  const style = normalizeFrescoStyle(node.props.style);
  return clipsOverflow(style?.overflow ?? style?.overflowX ?? style?.overflowY);
}

function containsPoint(layout: LayoutResultNapi, clip: HitRegion, x: number, y: number): boolean {
  return (
    x >= layout.x &&
    x < layout.x + layout.width &&
    y >= layout.y &&
    y < layout.y + layout.height &&
    x >= clip.left &&
    x < clip.right &&
    y >= clip.top &&
    y < clip.bottom
  );
}

/** Return the deepest topmost live host node under one native-layout coordinate. */
export function hitTest(
  nodes: ReadonlyMap<number, FrescoHostNode>,
  root: FrescoHostNode,
  x: number,
  y: number,
  layouts: readonly LayoutResultNapi[],
): FrescoHostNode | undefined {
  if (!Number.isFinite(x) || !Number.isFinite(y)) return undefined;
  const byId = new Map(layouts.map((layout) => [layout.id, layout]));
  const unbounded: HitRegion = {
    left: Number.NEGATIVE_INFINITY,
    top: Number.NEGATIVE_INFINITY,
    right: Number.POSITIVE_INFINITY,
    bottom: Number.POSITIVE_INFINITY,
  };

  const visit = (node: FrescoHostNode, ancestorClip: HitRegion): FrescoHostNode | undefined => {
    const layout = byId.get(node.id);
    const clips = clipsDescendants(node);
    const childClip = layout
      ? {
          left: clips ? Math.max(ancestorClip.left, layout.x) : ancestorClip.left,
          top: clips ? Math.max(ancestorClip.top, layout.y) : ancestorClip.top,
          right: clips ? Math.min(ancestorClip.right, layout.x + layout.width) : ancestorClip.right,
          bottom: clips
            ? Math.min(ancestorClip.bottom, layout.y + layout.height)
            : ancestorClip.bottom,
        }
      : ancestorClip;

    for (let index = node.children.length - 1; index >= 0; index -= 1) {
      const child = node.children[index];
      if (!child) continue;
      const target = visit(child, childClip);
      if (target) return target;
    }
    if (!layout || !nodes.has(node.id)) return undefined;
    return containsPoint(layout, ancestorClip, x, y) ? node : undefined;
  };

  return visit(root, unbounded);
}
