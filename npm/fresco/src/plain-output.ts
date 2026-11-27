/**
 * Plain-text output of a Fresco node tree.
 *
 * Non-interactive renders and `renderToString` print the tree as text instead
 * of drawing it, and `<Static>` items are captured once and accumulated ahead
 * of the dynamic output.
 */

import type { FrescoNode } from "./renderer.js";
import { isStaticItemNode, isStaticNode } from "./staticNodes.js";

function nodeText(node: FrescoNode): string {
  if (node.text !== undefined) return node.text;
  const propText = node.props.text ?? node.props.content;
  if (typeof propText === "string" || typeof propText === "number") return String(propText);
  if (node.type === "input") {
    const value = node.props.value;
    const placeholder = node.props.placeholder;
    if (typeof value === "string" || typeof value === "number") return String(value);
    if (typeof placeholder === "string" || typeof placeholder === "number")
      return String(placeholder);
  }
  return "";
}

function joinOutput(parts: string[], separator = "\n"): string {
  return parts.filter(Boolean).join(separator);
}

function treeToString(node: FrescoNode, options: { skipStatic?: boolean } = {}): string {
  if (options.skipStatic && isStaticNode(node)) return "";

  const ownText = nodeText(node);
  const childOutput = node.children.map((child) => treeToString(child, options));

  if (node.type === "text" || node.type === "input") {
    return `${ownText}${childOutput.join("")}`;
  }

  const style = (node.props.style ?? {}) as Record<string, unknown>;
  const flexDirection = style.flexDirection ?? style.flex_direction;
  const separator = flexDirection === "column" ? "\n" : "";
  return joinOutput(childOutput, separator);
}

export function normalizeOutput(output: string): string {
  return output.endsWith("\n") ? output.slice(0, -1) : output;
}

export function appendOutput(previous: string, next: string): string {
  const normalized = normalizeOutput(next);
  if (!normalized) return previous;
  return previous ? `${previous}\n${normalized}` : normalized;
}

export function captureStaticOutput(
  node: FrescoNode,
  renderedStaticItems: Map<number, number>,
): string {
  const output: string[] = [];

  function visit(current: FrescoNode) {
    if (isStaticNode(current)) {
      const renderedItems = renderedStaticItems.get(current.id) ?? 0;
      const staticItems = current.children.filter(isStaticItemNode);
      const nextItems = staticItems.slice(renderedItems);
      const nextOutput = joinOutput(
        nextItems.map((child) => treeToString(child)),
        "\n",
      );
      if (nextOutput) output.push(nextOutput);
      renderedStaticItems.set(current.id, staticItems.length);
      return;
    }

    for (const child of current.children) {
      visit(child);
    }
  }

  visit(node);
  return joinOutput(output, "\n");
}

// oxlint-disable-next-line no-control-regex -- control characters are exactly what is removed.
const PLAIN_OUTPUT_CONTROL_CHARACTERS = /[\u0000-\u0008\u000b-\u001f\u007f-\u009f]/gu;

/**
 * Plain output is written to the stream as is, so displayed text must not be able
 * to drive the terminal: every control character except tab and newline is dropped.
 */
export function renderPlainOutput(root: FrescoNode, staticOutput: string): string {
  return joinOutput([staticOutput, treeToString(root, { skipStatic: true })], "\n").replaceAll(
    PLAIN_OUTPUT_CONTROL_CHARACTERS,
    "",
  );
}
