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

function zIndex(node: FrescoHostNode): number {
  const style = node.props.style;
  if (typeof style !== "object" || style === null) return 0;
  const record = style as Record<string, unknown>;
  const value = record.zIndex !== undefined ? record.zIndex : record.z_index;
  return typeof value === "number" &&
    Number.isInteger(value) &&
    value >= -2_147_483_648 &&
    value <= 2_147_483_647
    ? value
    : 0;
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

    if (node.children.some((child) => zIndex(child) !== 0)) {
      const children = node.children
        .map((child, index) => ({ child, index, zIndex: zIndex(child) }))
        .sort((left, right) => right.zIndex - left.zIndex || right.index - left.index);
      for (const { child } of children) {
        const target = visit(child, childClip);
        if (target) return target;
      }
    } else {
      for (let index = node.children.length - 1; index >= 0; index--) {
        const child = node.children[index];
        if (!child) continue;
        const target = visit(child, childClip);
        if (target) return target;
      }
    }
    if (!layout || !nodes.has(node.id)) return undefined;
    return containsPoint(layout, ancestorClip, x, y) ? node : undefined;
  };

  return visit(root, unbounded);
}
