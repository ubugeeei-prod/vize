/**
 * Ink-compatible screen reader helpers.
 */

import type { InjectionKey, Ref } from "@vue/runtime-core";
import type { FlexStyleNapi } from "@vizejs/fresco-native";
import type { FrescoNode, NativeRenderNode } from "./renderer.js";

export const SCREEN_READER_KEY: InjectionKey<Readonly<Ref<boolean>>> =
  Symbol("fresco-screen-reader");

export type AriaRole =
  | "button"
  | "checkbox"
  | "combobox"
  | "list"
  | "listbox"
  | "listitem"
  | "menu"
  | "menuitem"
  | "option"
  | "progressbar"
  | "radio"
  | "radiogroup"
  | "tab"
  | "tablist"
  | "table"
  | "textbox"
  | "timer"
  | "toolbar";

export type AriaState = Partial<
  Record<
    | "busy"
    | "checked"
    | "disabled"
    | "expanded"
    | "multiline"
    | "multiselectable"
    | "readonly"
    | "required"
    | "selected",
    boolean
  >
>;

export function isScreenReaderEnabledByDefault(): boolean {
  return process.env.INK_SCREEN_READER === "true" || process.env.FRESCO_SCREEN_READER === "true";
}

function stringValue(value: unknown): string | undefined {
  if (typeof value === "string" || typeof value === "number") return String(value);
  return undefined;
}

function nodeText(node: FrescoNode): string {
  const ariaLabel = stringValue(node.props["aria-label"]);
  if (ariaLabel !== undefined) return ariaLabel;
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

function flexDirection(node: FrescoNode): string {
  const style = (node.props.style ?? {}) as Record<string, unknown>;
  return stringValue(style.flexDirection ?? style.flex_direction) ?? "column";
}

function ariaRole(node: FrescoNode): AriaRole | undefined {
  return stringValue(node.props["aria-role"]) as AriaRole | undefined;
}

function ariaState(node: FrescoNode): AriaState | undefined {
  const state = node.props["aria-state"];
  if (!state || typeof state !== "object") return undefined;
  return state as AriaState;
}

function isAriaHidden(node: FrescoNode): boolean {
  return node.props["aria-hidden"] === true;
}

export function treeToScreenReaderString(
  node: FrescoNode,
  options: { parentRole?: AriaRole } = {},
): string {
  if (isAriaHidden(node)) return "";

  let output = "";
  const text = nodeText(node);

  if (node.type === "text" || node.type === "input") {
    output = `${text}${node.children.map((child) => treeToScreenReaderString(child, options)).join("")}`;
  } else if (text) {
    output = text;
  } else {
    const direction = flexDirection(node);
    const separator = direction === "row" || direction === "row-reverse" ? " " : "\n";
    const children =
      direction === "row-reverse" || direction === "column-reverse"
        ? [...node.children].reverse()
        : node.children;

    const role = ariaRole(node);
    output = children
      .map((child) => treeToScreenReaderString(child, { parentRole: role }))
      .filter(Boolean)
      .join(separator);
  }

  const state = ariaState(node);
  if (state) {
    const stateDescription = Object.keys(state)
      .filter((key) => state[key as keyof AriaState])
      .join(", ");

    if (stateDescription) {
      output = `(${stateDescription}) ${output}`;
    }
  }

  const role = ariaRole(node);
  if (role && role !== options.parentRole) {
    output = `${role}: ${output}`;
  }

  return output;
}

export function treeToScreenReaderRenderNodes(root: FrescoNode): NativeRenderNode[] {
  const output = treeToScreenReaderString(root);
  const rootStyle = (root.props.style ?? {}) as FlexStyleNapi;
  const nodes: NativeRenderNode[] = [
    {
      id: root.id,
      nodeType: "root",
      style: {
        ...rootStyle,
        flexDirection: "column",
      },
    },
  ];

  if (!output) return nodes;

  const textId = root.id - 1;
  nodes[0].children = [textId];
  nodes.push({
    id: textId,
    nodeType: "text",
    text: output,
    wrap: true,
    wrapMode: "wrap",
  });

  return nodes;
}
