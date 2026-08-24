/**
 * Test utilities for Fresco component and app behavior.
 *
 * The harness mounts through the real Vue custom renderer and records frames
 * without loading the native terminal binding, so it is safe in plain Node,
 * Vitest, and `node:test` suites.
 */

import { nextTick, type Component, type VNodeChild } from "@vue/runtime-core";
import {
  lastCompositionEvent,
  lastFocusEvent,
  lastMouseEvent,
  lastPasteEvent,
  lastResizeEvent,
  type CompositionEvent,
  type FocusEvent,
  type KeyEvent,
  type MouseEvent,
  type PasteEvent,
  type ResizeEvent,
} from "../app.js";
import type { FrescoNode } from "../renderer.js";
import {
  dispatchKey,
  mountFresco,
  toTreeSnapshot,
  type FrescoNodeSnapshot,
  type MountedFresco,
  type MountFrescoOptions,
} from "./mount.js";

export {
  dispatchKey,
  findNodes,
  firstChild,
  mountComponent,
  mountFresco,
  toTreeSnapshot,
  typeChars,
  type FrescoNodeSnapshot,
  type MountedFresco,
  type MountFrescoOptions,
} from "./mount.js";
export {
  getByDescription,
  getByRole,
  getByTestId,
  getByText,
  queryAllByDescription,
  queryAllByRole,
  queryAllByTestId,
  queryAllByText,
  type FrescoRoleQueryOptions,
  type FrescoTextMatcher,
} from "./queries.js";

export type RenderTuiOptions = MountFrescoOptions;
export type RenderTuiRoot = Component | (() => VNodeChild);
export type KeyInput = Partial<Omit<KeyEvent, "type">>;
export type MouseInput = Omit<MouseEvent, "type">;

/** A single recorded harness frame. */
export interface FrescoFrameSnapshot {
  /** Plain terminal output projected from the current mounted tree. */
  output: string;
  /** Serializable renderer tree for structural assertions. */
  tree: FrescoNodeSnapshot;
}

/** Input driver that injects events through Fresco's public event refs. */
export interface FrescoInputDriver {
  /** Dispatch a key event and record the resulting frame. */
  key(event: KeyInput): Promise<FrescoFrameSnapshot>;
  /** Dispatch each printable character and record one frame after the text. */
  text(text: string): Promise<FrescoFrameSnapshot>;
  /** Dispatch a bracketed paste payload and record the resulting frame. */
  paste(text: string): Promise<FrescoFrameSnapshot>;
  /** Update the provided app size, dispatch a resize event, and record a frame. */
  resize(width: number, height: number): Promise<FrescoFrameSnapshot>;
  /** Dispatch a mouse event and record the resulting frame. */
  mouse(event: MouseInput): Promise<FrescoFrameSnapshot>;
  /** Dispatch terminal focus state and record the resulting frame. */
  focus(focused: boolean): Promise<FrescoFrameSnapshot>;
  /** Dispatch a composition start event and record the resulting frame. */
  compositionStart(text?: string, cursor?: number): Promise<FrescoFrameSnapshot>;
  /** Dispatch a composition update event and record the resulting frame. */
  compositionUpdate(text: string, cursor: number): Promise<FrescoFrameSnapshot>;
  /** Dispatch a composition end event and record the resulting frame. */
  compositionEnd(text: string, cursor?: number): Promise<FrescoFrameSnapshot>;
}

/** Mounted TUI harness with frame history and input helpers. */
export interface RenderTuiResult extends MountedFresco {
  /** Recorded plain frames, including the initial mount frame. */
  readonly frames: readonly string[];
  /** Recorded frame snapshots, including the initial mount frame. */
  readonly frameSnapshots: readonly FrescoFrameSnapshot[];
  /** Event injection helpers. */
  readonly input: FrescoInputDriver;
  /** Record and return a fresh snapshot of the current mounted tree. */
  captureFrame(): FrescoFrameSnapshot;
  /** Return the latest recorded plain frame. */
  lastFrame(): string;
  /** Return the latest recorded frame snapshot. */
  frameSnapshot(): FrescoFrameSnapshot;
}

function nodeText(node: FrescoNode): string {
  if (node.text !== undefined) return node.text;
  const propText = node.props.text ?? node.props.content;
  if (typeof propText === "string" || typeof propText === "number") return String(propText);
  if (node.type !== "input") return "";

  const value = node.props.value;
  const placeholder = node.props.placeholder;
  if (typeof value === "string" || typeof value === "number") return String(value);
  if (typeof placeholder === "string" || typeof placeholder === "number")
    return String(placeholder);
  return "";
}

function isStaticNode(node: FrescoNode): boolean {
  return node.props.internal_static === true || node.props.internalStatic === true;
}

function joinOutput(parts: readonly string[], separator = "\n"): string {
  return parts.filter(Boolean).join(separator);
}

function normalizeOutput(output: string): string {
  return output.endsWith("\n") ? output.slice(0, -1) : output;
}

function appendOutput(previous: string, next: string): string {
  const normalized = normalizeOutput(next);
  if (!normalized) return previous;
  return previous ? `${previous}\n${normalized}` : normalized;
}

function treeToFrameOutput(node: FrescoNode, options: { skipStatic?: boolean } = {}): string {
  if (options.skipStatic && isStaticNode(node)) return "";

  const ownText = nodeText(node);
  const childOutput = node.children.map((child) => treeToFrameOutput(child, options));

  if (node.type === "text" || node.type === "input") {
    return `${ownText}${childOutput.join("")}`;
  }

  const style = (node.props.style ?? {}) as Record<string, unknown>;
  const flexDirection = style.flexDirection ?? style.flex_direction;
  const separator = flexDirection === "column" ? "\n" : "";
  return joinOutput(childOutput, separator);
}

function captureStaticOutput(node: FrescoNode, renderedStaticItems: Map<number, number>): string {
  const output: string[] = [];

  const visit = (current: FrescoNode) => {
    if (isStaticNode(current)) {
      const renderedItems = renderedStaticItems.get(current.id) ?? 0;
      const nextItems = current.children.slice(renderedItems);
      const nextOutput = joinOutput(
        nextItems.map((child) => treeToFrameOutput(child)),
        "\n",
      );
      if (nextOutput) output.push(nextOutput);
      renderedStaticItems.set(current.id, current.children.length);
      return;
    }

    for (const child of current.children) visit(child);
  };

  visit(node);
  return joinOutput(output, "\n");
}

function renderFrame(root: FrescoNode, staticOutput: string): string {
  return joinOutput([staticOutput, treeToFrameOutput(root, { skipStatic: true })], "\n");
}

function latestSnapshot(snapshots: readonly FrescoFrameSnapshot[]): FrescoFrameSnapshot {
  const snapshot = snapshots[snapshots.length - 1];
  if (!snapshot) throw new Error("expected renderTui to record an initial frame");
  return snapshot;
}

async function dispatchPaste(text: string): Promise<void> {
  const event: PasteEvent = { type: "paste", text };
  lastPasteEvent.value = event;
  await nextTick();
}

async function dispatchMouse(event: MouseInput): Promise<void> {
  lastMouseEvent.value = { type: "mouse", ...event };
  await nextTick();
}

async function dispatchFocus(focused: boolean): Promise<void> {
  const event: FocusEvent = { type: "focus", focused };
  lastFocusEvent.value = event;
  await nextTick();
}

async function dispatchComposition(
  type: CompositionEvent["type"],
  text: string,
  cursor: number,
): Promise<void> {
  lastCompositionEvent.value = { type, text, cursor };
  await nextTick();
}

async function dispatchResize(
  mounted: MountedFresco,
  width: number,
  height: number,
): Promise<void> {
  const event: ResizeEvent = { type: "resize", width, height };
  lastResizeEvent.value = event;
  mounted.appContext.width.value = width;
  mounted.appContext.height.value = height;
  await nextTick();
}

/** Mount a Fresco app for tests with frame snapshots and input injection. */
export function renderTui(root: RenderTuiRoot, options: RenderTuiOptions = {}): RenderTuiResult {
  const mounted = mountFresco(root, options);
  const frames: string[] = [];
  const frameSnapshots: FrescoFrameSnapshot[] = [];
  const renderedStaticItems = new Map<number, number>();
  let staticOutput = "";

  const captureFrame = () => {
    staticOutput = appendOutput(
      staticOutput,
      captureStaticOutput(mounted.root, renderedStaticItems),
    );
    const snapshot: FrescoFrameSnapshot = {
      output: renderFrame(mounted.root, staticOutput),
      tree: toTreeSnapshot(mounted.root),
    };
    frames.push(snapshot.output);
    frameSnapshots.push(snapshot);
    return snapshot;
  };

  const recordAfter = async (dispatch: Promise<void>) => {
    await dispatch;
    return captureFrame();
  };

  const input: FrescoInputDriver = {
    key: (event) => recordAfter(dispatchKey(event)),
    async text(text) {
      for (const char of text) {
        await dispatchKey({ char });
      }
      return captureFrame();
    },
    paste: (text) => recordAfter(dispatchPaste(text)),
    resize: (width, height) => recordAfter(dispatchResize(mounted, width, height)),
    mouse: (event) => recordAfter(dispatchMouse(event)),
    focus: (focused) => recordAfter(dispatchFocus(focused)),
    compositionStart: (text = "", cursor = 0) =>
      recordAfter(dispatchComposition("compositionstart", text, cursor)),
    compositionUpdate: (text, cursor) =>
      recordAfter(dispatchComposition("compositionupdate", text, cursor)),
    compositionEnd: (text, cursor = text.length) =>
      recordAfter(dispatchComposition("compositionend", text, cursor)),
  };

  captureFrame();

  return {
    ...mounted,
    frames,
    frameSnapshots,
    input,
    captureFrame,
    lastFrame: () => latestSnapshot(frameSnapshots).output,
    frameSnapshot: () => latestSnapshot(frameSnapshots),
  };
}
