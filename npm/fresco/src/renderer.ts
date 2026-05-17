/**
 * Fresco Vue Custom Renderer
 */

import {
  createRenderer as createVueRenderer,
  type RendererOptions,
  type RendererNode,
  type RendererElement,
} from "@vue/runtime-core";
import type { FlexStyleNapi, RenderNodeNapi, StyleNapi } from "@vizejs/fresco-native";

/**
 * Fresco node types
 */
export interface FrescoNode extends RendererNode {
  id: number;
  type: "box" | "text" | "input" | "root";
  props: Record<string, unknown>;
  children: FrescoNode[];
  parent: FrescoNode | null;
  text?: string;
}

/**
 * Fresco element (extends node)
 */
export interface FrescoElement extends FrescoNode, RendererElement {}

let nextId = 0;

function createNode(type: FrescoNode["type"]): FrescoNode {
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
function mapElementType(type: string): FrescoNode["type"] {
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

export interface NativeRenderNode extends RenderNodeNapi {}

function stringValue(value: unknown): string | undefined {
  if (typeof value === "string" || typeof value === "number") return String(value);
  return undefined;
}

function styleValue(style: Record<string, unknown>, ...keys: string[]): unknown {
  for (const key of keys) {
    if (style[key] !== undefined) return style[key];
  }
  return undefined;
}

function copyStringStyle(
  output: Record<string, unknown>,
  style: Record<string, unknown>,
  nativeKey: string,
  ...sourceKeys: string[]
) {
  const value = styleValue(style, ...sourceKeys);
  const normalized = stringValue(value);
  if (normalized !== undefined) output[nativeKey] = normalized;
}

function copyNumberStyle(
  output: Record<string, unknown>,
  style: Record<string, unknown>,
  nativeKey: string,
  ...sourceKeys: string[]
) {
  const value = styleValue(style, ...sourceKeys);
  if (typeof value === "number") {
    output[nativeKey] = value;
  }
}

function copyRawStyle(
  output: Record<string, unknown>,
  style: Record<string, unknown>,
  nativeKey: string,
  ...sourceKeys: string[]
) {
  const value = styleValue(style, ...sourceKeys);
  if (value !== undefined) output[nativeKey] = value;
}

function isWrappingEnabled(value: unknown): boolean {
  if (value === undefined) return false;
  if (value === false) return false;
  if (typeof value === "string" && value.startsWith("truncate")) return false;
  return true;
}

function textWrapMode(value: unknown): string | undefined {
  if (value === undefined) return undefined;
  if (value === false) return "none";
  if (value === true) return "wrap";
  if (value === "end") return "truncate-end";
  if (value === "middle") return "truncate-middle";
  if (typeof value === "string") return value;
  return undefined;
}

/**
 * Convert Fresco tree to render nodes for native
 */
export function treeToRenderNodes(root: FrescoNode): NativeRenderNode[] {
  const nodes: NativeRenderNode[] = [];

  function visit(node: FrescoNode) {
    const renderNode: NativeRenderNode = {
      id: node.id,
      nodeType: node.type,
    };

    // Extract props
    const text = node.text ?? stringValue(node.props.text) ?? stringValue(node.props.content);
    if (text !== undefined) {
      renderNode.text = text;
    }
    if (node.props.wrap !== undefined) {
      renderNode.wrap = isWrappingEnabled(node.props.wrap);
      renderNode.wrapMode = textWrapMode(node.props.wrap);
    }
    if (node.props.value !== undefined) {
      renderNode.value = stringValue(node.props.value) ?? "";
    }
    if (node.props.placeholder !== undefined) {
      renderNode.placeholder = stringValue(node.props.placeholder) ?? "";
    }
    if (node.props.focused !== undefined || node.props.focus !== undefined) {
      renderNode.focused = Boolean(node.props.focused ?? node.props.focus);
    }
    if (node.props.cursor !== undefined) {
      renderNode.cursor = Number(node.props.cursor);
    }
    if (node.props.mask !== undefined) {
      renderNode.mask = Boolean(node.props.mask);
    }
    if (node.props.maskChar !== undefined || node.props["mask-char"] !== undefined) {
      renderNode.maskChar = stringValue(node.props.maskChar ?? node.props["mask-char"]);
    }
    if (node.props.border !== undefined) {
      renderNode.border = stringValue(node.props.border) ?? "";
    }

    // Extract style - only include defined values
    if (node.props.style) {
      const s = node.props.style as Record<string, unknown>;
      const style: Record<string, unknown> = {};

      copyRawStyle(style, s, "display", "display");
      copyRawStyle(style, s, "position", "position");
      copyStringStyle(style, s, "top", "top");
      copyStringStyle(style, s, "right", "right");
      copyStringStyle(style, s, "bottom", "bottom");
      copyStringStyle(style, s, "left", "left");
      copyRawStyle(style, s, "flexDirection", "flexDirection", "flex_direction");
      copyRawStyle(style, s, "flexWrap", "flexWrap", "flex_wrap");
      copyRawStyle(style, s, "justifyContent", "justifyContent", "justify_content");
      copyRawStyle(style, s, "alignItems", "alignItems", "align_items");
      copyRawStyle(style, s, "alignSelf", "alignSelf", "align_self");
      copyRawStyle(style, s, "alignContent", "alignContent", "align_content");
      copyNumberStyle(style, s, "flexGrow", "flexGrow", "flex_grow");
      copyNumberStyle(style, s, "flexShrink", "flexShrink", "flex_shrink");
      copyStringStyle(style, s, "flexBasis", "flexBasis", "flex_basis");
      copyStringStyle(style, s, "width", "width");
      copyStringStyle(style, s, "height", "height");
      copyStringStyle(style, s, "minWidth", "minWidth", "min_width");
      copyStringStyle(style, s, "minHeight", "minHeight", "min_height");
      copyStringStyle(style, s, "maxWidth", "maxWidth", "max_width");
      copyStringStyle(style, s, "maxHeight", "maxHeight", "max_height");
      copyNumberStyle(style, s, "aspectRatio", "aspectRatio", "aspect_ratio");
      copyNumberStyle(style, s, "padding", "padding");
      copyNumberStyle(style, s, "paddingTop", "paddingTop", "padding_top");
      copyNumberStyle(style, s, "paddingRight", "paddingRight", "padding_right");
      copyNumberStyle(style, s, "paddingBottom", "paddingBottom", "padding_bottom");
      copyNumberStyle(style, s, "paddingLeft", "paddingLeft", "padding_left");
      copyNumberStyle(style, s, "margin", "margin");
      copyNumberStyle(style, s, "marginTop", "marginTop", "margin_top");
      copyNumberStyle(style, s, "marginRight", "marginRight", "margin_right");
      copyNumberStyle(style, s, "marginBottom", "marginBottom", "margin_bottom");
      copyNumberStyle(style, s, "marginLeft", "marginLeft", "margin_left");
      copyNumberStyle(style, s, "gap", "gap");
      copyNumberStyle(style, s, "columnGap", "columnGap", "column_gap");
      copyNumberStyle(style, s, "rowGap", "rowGap", "row_gap");
      copyRawStyle(style, s, "overflow", "overflow");
      copyRawStyle(style, s, "overflowX", "overflowX", "overflow_x");
      copyRawStyle(style, s, "overflowY", "overflowY", "overflow_y");

      if (Object.keys(style).length > 0) {
        renderNode.style = style as FlexStyleNapi;
      }
    }

    // Extract appearance (fg, bg, bold, etc.)
    const appearance: Record<string, unknown> = {};
    const fg = node.props.fg ?? node.props.color;
    const bg = node.props.bg ?? node.props.backgroundColor;
    if (fg) appearance.fg = fg;
    if (bg) appearance.bg = bg;
    if (node.props.bold) appearance.bold = node.props.bold;
    if (node.props.dim || node.props.dimColor)
      appearance.dim = Boolean(node.props.dim || node.props.dimColor);
    if (node.props.italic) appearance.italic = node.props.italic;
    if (node.props.underline) appearance.underline = node.props.underline;
    if (node.props.strikethrough) appearance.strikethrough = node.props.strikethrough;
    if (node.props.inverse) appearance.inverse = node.props.inverse;
    if (Object.keys(appearance).length > 0) {
      renderNode.appearance = appearance as StyleNapi;
    }

    // Children
    if (node.children.length > 0) {
      renderNode.children = node.children.map((c) => c.id);
    }

    nodes.push(renderNode);

    // Visit children
    for (const child of node.children) {
      visit(child);
    }
  }

  visit(root);
  return nodes;
}
