import type { FrescoNode } from "./renderer.js";

/** Whether a renderer node owns append-only static output. */
export function isStaticNode(node: FrescoNode): boolean {
  return node.props.internal_static === true || node.props.internalStatic === true;
}

/** Whether a renderer node is one append-only item inside a static owner. */
export function isStaticItemNode(node: FrescoNode): boolean {
  return node.props.internal_static_item === true || node.props.internalStaticItem === true;
}
