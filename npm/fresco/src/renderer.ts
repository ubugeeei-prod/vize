/**
 * Fresco Vue Custom Renderer
 */

import {
  createRenderer as createVueRenderer,
  type RendererOptions,
  type RendererNode,
  type RendererElement,
} from "vue";
import type { FrescoRenderNode, FrescoRenderNodeKind } from "./protocol.js";
import { frescoNodeToRenderNode } from "./renderPayload.js";

export type { FrescoRenderNodeKind } from "./protocol.js";

/** Read-only view of a node owned by one Fresco renderer instance. */
export interface FrescoHostNode {
  /** Stable, process-unique identifier retained across keyed moves and updates. */
  readonly id: number;
  /** Native render-node kind represented by this host node. */
  readonly type: FrescoRenderNodeKind;
  /** Current Vue host properties. */
  readonly props: Readonly<Record<string, unknown>>;
  /** Ordered child host nodes. */
  readonly children: readonly FrescoHostNode[];
  /** Parent host node, or null while detached. */
  readonly parent: FrescoHostNode | null;
  /** Current text-node content when present. */
  readonly text?: string;
}

/** Declared host dimensions observed before native layout. */
export interface FrescoDeclaredSize {
  /** Declared width, when the host props provide one. */
  readonly width?: string | number;
  /** Declared height, when the host props provide one. */
  readonly height?: string | number;
}

/** Host-tree update category passed to lifecycle observers. */
export type FrescoNodeUpdateKind = "children" | "props" | "text";

/** Optional lifecycle observers for nodes owned by a Fresco renderer. */
export interface FrescoNodeLifecycleHooks {
  /** Observe allocation of a stable host node. @default No-op. */
  onCreate?(node: FrescoHostNode): void;
  /** Observe a committed host-tree update. @default No-op. */
  onUpdate?(node: FrescoHostNode, kind: FrescoNodeUpdateKind): void;
  /** Observe a change to declared width or height. @default No-op. */
  onResize?(node: FrescoHostNode, previous: FrescoDeclaredSize, current: FrescoDeclaredSize): void;
  /** Observe recursive disposal after descendants have been removed. @default No-op. */
  onRemove?(node: FrescoHostNode): void;
}

/** Configuration for {@link createRenderer}. */
export interface FrescoRendererOptions {
  /** Observe a host-tree mutation that requires native frame work. @default No-op. */
  onMutation?: () => void;
  /** Observe stable-node lifecycle transitions. @default No lifecycle observers. */
  lifecycle?: FrescoNodeLifecycleHooks;
}

/** @internal Mutable Vue host node; not part of the public render protocol. */
export interface FrescoNode extends RendererNode, FrescoHostNode {
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

function dimension(value: unknown): string | number | undefined {
  return typeof value === "string" || typeof value === "number" ? value : undefined;
}

function declaredSize(node: FrescoNode): FrescoDeclaredSize {
  const style =
    typeof node.props.style === "object" && node.props.style !== null
      ? (node.props.style as Record<string, unknown>)
      : {};
  return {
    width: dimension(style.width ?? node.props.width),
    height: dimension(style.height ?? node.props.height),
  };
}

function sameSize(left: FrescoDeclaredSize, right: FrescoDeclaredSize): boolean {
  return Object.is(left.width, right.width) && Object.is(left.height, right.height);
}

interface HostTree {
  readonly nodes: Map<number, FrescoNode>;
  readonly lifecycle: FrescoNodeLifecycleHooks;
  readonly onMutation: () => void;
}

function createNode(tree: HostTree, type: FrescoRenderNodeKind, text?: string): FrescoNode {
  const node: FrescoNode = {
    id: nextId++,
    type,
    props: {},
    children: [],
    parent: null,
    ...(text === undefined ? {} : { text }),
  };
  tree.nodes.set(node.id, node);
  tree.lifecycle.onCreate?.(node);
  return node;
}

function notifyUpdate(tree: HostTree, node: FrescoNode, kind: FrescoNodeUpdateKind): void {
  tree.lifecycle.onUpdate?.(node, kind);
}

function disposeNode(tree: HostTree, node: FrescoNode): boolean {
  if (!tree.nodes.has(node.id)) return false;
  for (const child of node.children) disposeNode(tree, child);
  node.children = [];
  node.parent = null;
  tree.nodes.delete(node.id);
  tree.lifecycle.onRemove?.(node);
  return true;
}

function detachNode(tree: HostTree, node: FrescoNode): boolean {
  const parent = node.parent;
  if (!parent) return false;
  const index = parent.children.indexOf(node);
  node.parent = null;
  if (index === -1) return false;
  parent.children.splice(index, 1);
  notifyUpdate(tree, parent, "children");
  return true;
}

/**
 * Renderer options for Fresco
 */
function rendererOptions(tree: HostTree): RendererOptions<FrescoNode, FrescoElement> {
  return {
    patchProp(el, key, _prevValue, nextValue) {
      const previousSize = declaredSize(el);
      if (nextValue == null) {
        if (!(key in el.props)) return;
        delete el.props[key];
      } else {
        if (Object.is(el.props[key], nextValue)) return;
        el.props[key] = nextValue;
      }
      notifyUpdate(tree, el, "props");
      const currentSize = declaredSize(el);
      if (!sameSize(previousSize, currentSize)) {
        tree.lifecycle.onResize?.(el, previousSize, currentSize);
      }
      tree.onMutation();
    },

    insert(child, parent, anchor) {
      // Vue reuses host nodes for keyed moves and re-inserts them without a
      // preceding remove, relying on DOM insertBefore move semantics. Detach the
      // child from its current position first so a reorder repositions the
      // existing node instead of duplicating it.
      const previousParent = child.parent;
      if (previousParent) {
        const existing = previousParent.children.indexOf(child);
        if (existing !== -1) previousParent.children.splice(existing, 1);
        child.parent = null;
        if (previousParent !== parent) notifyUpdate(tree, previousParent, "children");
      }
      child.parent = parent;
      if (anchor) {
        const index = parent.children.indexOf(anchor);
        if (index !== -1) {
          parent.children.splice(index, 0, child);
          notifyUpdate(tree, parent, "children");
          tree.onMutation();
          return;
        }
      }
      parent.children.push(child);
      notifyUpdate(tree, parent, "children");
      tree.onMutation();
    },

    remove(child) {
      const detached = detachNode(tree, child);
      const disposed = disposeNode(tree, child);
      if (detached || disposed) tree.onMutation();
    },

    createElement(type) {
      const nodeType = mapElementType(type);
      return createNode(tree, nodeType) as FrescoElement;
    },

    createText(text) {
      return createNode(tree, "text", text);
    },

    createComment() {
      // Comments are ignored in TUI
      return createNode(tree, "text");
    },

    setText(node, text) {
      if (node.text === text) return;
      node.text = text;
      notifyUpdate(tree, node, "text");
      tree.onMutation();
    },

    setElementText(el, text) {
      if (el.text === text && el.children.length === 0) return;
      for (const child of el.children) disposeNode(tree, child);
      el.text = text;
      el.children = [];
      notifyUpdate(tree, el, "text");
      tree.onMutation();
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
}

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
 * Create a Fresco renderer with stable host-node lookup and lifecycle control.
 */
export function createRenderer(options: FrescoRendererOptions | (() => void) = {}) {
  const normalized = typeof options === "function" ? { onMutation: options } : options;
  const tree: HostTree = {
    nodes: new Map(),
    lifecycle: normalized.lifecycle ?? {},
    onMutation: normalized.onMutation ?? (() => {}),
  };
  const renderer = createVueRenderer(rendererOptions(tree));

  return Object.assign(renderer, {
    /** Return one live host node by its stable identifier. */
    getNode(id: number): FrescoHostNode | undefined {
      return tree.nodes.get(id);
    },
    /** Find one node under the supplied subtree without crossing its boundary. */
    findDescendantById(root: FrescoHostNode, id: number): FrescoHostNode | undefined {
      if (root.id === id) return root;
      const registered = tree.nodes.get(id);
      if (!registered) return undefined;
      for (let parent = registered.parent; parent; parent = parent.parent) {
        if (parent === root) return registered;
      }
      return undefined;
    },
    /** Detach and dispose a registered node and every descendant exactly once. */
    destroyRecursively(id: number): boolean {
      const node = tree.nodes.get(id);
      if (!node) return false;
      detachNode(tree, node);
      disposeNode(tree, node);
      tree.onMutation();
      return true;
    },
  });
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
