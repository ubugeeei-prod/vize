/**
 * Test-only in-memory Vue renderer, so lifecycle hooks such as `onMounted`
 * run under `node --test` without a DOM. Not part of the published package.
 */
import { createRenderer } from "vue";
import type { App, Component } from "vue";

/** Minimal element/text/comment node of the in-memory tree. */
export interface MemoryNode {
  readonly type: "element" | "text" | "comment";
  readonly tag: string;
  text: string;
  parent: MemoryNode | null;
  readonly children: MemoryNode[];
  readonly props: Record<string, unknown>;
}

function createNode(type: MemoryNode["type"], tag: string, text = ""): MemoryNode {
  return { type, tag, text, parent: null, children: [], props: {} };
}

function detach(node: MemoryNode): void {
  const siblings = node.parent?.children;
  if (siblings !== undefined) siblings.splice(siblings.indexOf(node), 1);
  node.parent = null;
}

const renderer = createRenderer<MemoryNode, MemoryNode>({
  createElement: (tag) => createNode("element", tag),
  createText: (text) => createNode("text", "#text", text),
  createComment: (text) => createNode("comment", "#comment", text),
  setText: (node, text) => {
    node.text = text;
  },
  setElementText: (node, text) => {
    for (const child of node.children.splice(0)) child.parent = null;
    const textNode = createNode("text", "#text", text);
    textNode.parent = node;
    node.children.push(textNode);
  },
  insert: (child, parent, anchor) => {
    detach(child);
    child.parent = parent;
    const index = anchor == null ? -1 : parent.children.indexOf(anchor);
    if (index === -1) parent.children.push(child);
    else parent.children.splice(index, 0, child);
  },
  remove: (child) => {
    detach(child);
  },
  parentNode: (node) => node.parent,
  nextSibling: (node) => {
    const siblings = node.parent?.children ?? [];
    return siblings[siblings.indexOf(node) + 1] ?? null;
  },
  patchProp: (node, key, _previous, next) => {
    node.props[key] = next;
  },
});

/** Serialize the text content of a memory tree. */
export function textContent(node: MemoryNode): string {
  return node.type === "text" ? node.text : node.children.map(textContent).join("");
}

/** Mount a component into a fresh in-memory root. */
export function mountInMemory(component: Component): {
  readonly app: App<MemoryNode>;
  readonly root: MemoryNode;
  readonly unmount: () => void;
} {
  const root = createNode("element", "root");
  const app = renderer.createApp(component);
  app.mount(root);
  return { app, root, unmount: () => app.unmount() };
}
