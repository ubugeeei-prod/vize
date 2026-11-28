import type { AriaRole, AriaState } from "../accessibility.js";
import type { FrescoNode } from "../renderer.js";
import { findNodes } from "./mount.js";

/** Exact text or a regular expression used by terminal semantic queries. */
export type FrescoTextMatcher = string | RegExp;

/** Optional accessible-name, description, and state filters for role queries. */
export interface FrescoRoleQueryOptions {
  /** Accessible-name filter; omission accepts every name. @default undefined */
  readonly name?: FrescoTextMatcher;
  /** Accessible-description filter; omission accepts every description. @default undefined */
  readonly description?: FrescoTextMatcher;
  /** Requested state subset compared with strict equality; omission accepts every state. @default undefined */
  readonly state?: AriaState;
}

function stringValue(value: unknown): string | undefined {
  if (typeof value === "string" || typeof value === "number") return String(value);
  return undefined;
}

function ownText(node: FrescoNode): string | undefined {
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

  return undefined;
}

function accessibleName(node: FrescoNode): string {
  const ariaLabel = stringValue(node.props["aria-label"]);
  if (ariaLabel !== undefined) return ariaLabel;

  const text = ownText(node);
  if (text !== undefined) return text;

  return node.children
    .map((child) => accessibleName(child))
    .filter(Boolean)
    .join(" ");
}

function accessibleDescription(node: FrescoNode): string | undefined {
  return stringValue(
    node.props["aria-description"] ?? node.props.ariaDescription ?? node.props.description,
  );
}

function matchesText(text: string, matcher: FrescoTextMatcher): boolean {
  if (typeof matcher === "string") return text === matcher;
  matcher.lastIndex = 0;
  return matcher.test(text);
}

function roleOf(node: FrescoNode): AriaRole | undefined {
  return stringValue(node.props["aria-role"]) as AriaRole | undefined;
}

function testIdOf(node: FrescoNode): string | undefined {
  return stringValue(node.props["test-id"] ?? node.props["data-testid"]);
}

function stateOf(node: FrescoNode): AriaState {
  const state = node.props["aria-state"];
  return state && typeof state === "object" ? (state as AriaState) : {};
}

function matchesState(node: FrescoNode, expected: AriaState | undefined): boolean {
  if (expected === undefined) return true;
  const actual = stateOf(node);
  return Object.entries(expected).every(([key, value]) => actual[key as keyof AriaState] === value);
}

function describeRole(role: AriaRole, options: FrescoRoleQueryOptions): string {
  const parts = [`role ${JSON.stringify(role)}`];
  if (options.name !== undefined) parts.push(`name ${String(options.name)}`);
  if (options.description !== undefined) parts.push(`description ${String(options.description)}`);
  if (options.state !== undefined) parts.push(`state ${JSON.stringify(options.state)}`);
  return parts.join(", ");
}

function uniqueNode(nodes: readonly FrescoNode[], description: string): FrescoNode {
  if (nodes.length === 0) throw new Error(`Unable to find Fresco node with ${description}`);
  if (nodes.length > 1) throw new Error(`Found ${nodes.length} Fresco nodes with ${description}`);
  return nodes[0]!;
}

/** Return every node with the requested semantic role and optional accessibility filters. */
export function queryAllByRole(
  root: FrescoNode,
  role: AriaRole,
  options: FrescoRoleQueryOptions = {},
): FrescoNode[] {
  return findNodes(root, (node) => {
    if (roleOf(node) !== role) return false;
    if (options.name !== undefined && !matchesText(accessibleName(node), options.name))
      return false;
    if (
      options.description !== undefined &&
      !matchesText(accessibleDescription(node) ?? "", options.description)
    )
      return false;
    return matchesState(node, options.state);
  });
}

/** Return the only role match; throw when no node or multiple nodes match. */
export function getByRole(
  root: FrescoNode,
  role: AriaRole,
  options: FrescoRoleQueryOptions = {},
): FrescoNode {
  return uniqueNode(queryAllByRole(root, role, options), describeRole(role, options));
}

/** Return every node whose own text matches exactly or satisfies the regular expression. */
export function queryAllByText(root: FrescoNode, text: FrescoTextMatcher): FrescoNode[] {
  return findNodes(root, (node) => {
    const value = ownText(node);
    return value !== undefined && matchesText(value, text);
  });
}

/** Return the only text match; throw when no node or multiple nodes match. */
export function getByText(root: FrescoNode, text: FrescoTextMatcher): FrescoNode {
  return uniqueNode(queryAllByText(root, text), `text ${String(text)}`);
}

/** Return every node whose accessible description matches the text or regular expression. */
export function queryAllByDescription(
  root: FrescoNode,
  description: FrescoTextMatcher,
): FrescoNode[] {
  return findNodes(root, (node) => {
    const value = accessibleDescription(node);
    return value !== undefined && matchesText(value, description);
  });
}

/** Return the only description match; throw when no node or multiple nodes match. */
export function getByDescription(root: FrescoNode, description: FrescoTextMatcher): FrescoNode {
  return uniqueNode(queryAllByDescription(root, description), `description ${String(description)}`);
}

/** Return every node with the exact test-id or data-testid value. */
export function queryAllByTestId(root: FrescoNode, testId: string): FrescoNode[] {
  return findNodes(root, (node) => testIdOf(node) === testId);
}

/** Return the only test-id match; throw when no node or multiple nodes match. */
export function getByTestId(root: FrescoNode, testId: string): FrescoNode {
  return uniqueNode(queryAllByTestId(root, testId), `test id ${JSON.stringify(testId)}`);
}
