/**
 * Fresco Vue Custom Renderer
 */

import {
  createRenderer as createVueRenderer,
  type RendererOptions,
  type RendererNode,
  type RendererElement,
} from "@vue/runtime-core";
import type { FrescoRenderNode, FrescoRenderNodeKind } from "./protocol.js";
import { frescoNodeToRenderNode } from "./renderPayload.js";

export type { FrescoRenderNodeKind } from "./protocol.js";

/** @internal Mutable Vue host node; not part of the public render protocol. */
export interface FrescoNode extends RendererNode {
  id: number;
  type: FrescoRenderNodeKind;
  props: Record<string, unknown>;
  children: FrescoNode[];
  parent: FrescoNode | null;
  text?: string;
}

/** @internal Mutable Vue host element; not part of the public render protocol. */
export interface FrescoElement extends FrescoNode, RendererElement {}

let nextId = 0;

function createNode(type: FrescoRenderNodeKind): FrescoNode {
  return {
    id: nextId++,
    type,
    props: {},
    children: [],
    parent: null,
  };
}

/**
 * Renderer options for Fresco
 */
const rendererOptions: RendererOptions<FrescoNode, FrescoElement> = {
  patchProp(el, key, _prevValue, nextValue) {
    if (nextValue == null) {
      delete el.props[key];
    } else {
      el.props[key] = nextValue;
    }
  },

  insert(child, parent, anchor) {
    // Vue reuses host nodes for keyed moves and re-inserts them without a
    // preceding remove, relying on DOM insertBefore move semantics. Detach the
    // child from its current position first so a reorder repositions the
    // existing node instead of duplicating it.
    if (child.parent) {
      const existing = child.parent.children.indexOf(child);
      if (existing !== -1) {
        child.parent.children.splice(existing, 1);
      }
    }
    child.parent = parent;
    if (anchor) {
      const index = parent.children.indexOf(anchor);
      if (index !== -1) {
        parent.children.splice(index, 0, child);
        return;
      }
    }
    parent.children.push(child);
  },

  remove(child) {
    if (child.parent) {
      const index = child.parent.children.indexOf(child);
      if (index !== -1) {
        child.parent.children.splice(index, 1);
      }
      child.parent = null;
    }
  },

  createElement(type) {
    const nodeType = mapElementType(type);
    return createNode(nodeType) as FrescoElement;
  },

  createText(text) {
    const node = createNode("text");
    node.text = text;
    return node;
  },

  createComment() {
    // Comments are ignored in TUI
    return createNode("text");
  },

  setText(node, text) {
    node.text = text;
  },

  setElementText(el, text) {
    el.text = text;
    el.children = [];
  },

  parentNode(node) {
    return node.parent;
  },

  nextSibling(node) {
    if (!node.parent) return null;
    const index = node.parent.children.indexOf(node);
    return node.parent.children[index + 1] || null;
  },
};

/**
 * Map Vue element types to Fresco node types
 */
function mapElementType(type: string): FrescoRenderNodeKind {
  switch (type.toLowerCase()) {
    case "box":
    case "div":
    case "view":
      return "box";
    case "text":
    case "span":
      return "text";
    case "input":
    case "textinput":
      return "input";
    default:
      return "box";
  }
}

/**
 * Create the Fresco renderer
 */
export function createRenderer() {
  return createVueRenderer(rendererOptions);
}

/** Compatibility name for the render payload produced by this package. */
export type NativeRenderNode = FrescoRenderNode;

/**
 * Convert Fresco tree to render nodes for native
 */
export function treeToRenderNodes(root: FrescoNode): NativeRenderNode[] {
  const nodes: NativeRenderNode[] = [];

  function visit(node: FrescoNode) {
    const renderNode = frescoNodeToRenderNode(node);

    nodes.push(renderNode);

    // Visit children
    for (const child of node.children) {
      visit(child);
    }
  }

  visit(root);
  return nodes;
}
