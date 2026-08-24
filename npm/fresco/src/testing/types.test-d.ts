/** Compile-only assertions for the Fresco testing harness entrypoint. */

import { h } from "@vue/runtime-core";

import { TextInput } from "../components/index.js";
import {
  getByDescription,
  getByRole,
  getByTestId,
  getByText,
  queryAllByDescription,
  queryAllByRole,
  queryAllByTestId,
  queryAllByText,
  renderTui,
  type FrescoFrameSnapshot,
  type FrescoInputDriver,
  type FrescoRenderNode,
  type FrescoRoleQueryOptions,
  type FrescoTextMatcher,
  type RenderTuiResult,
} from "./index.js";

const rendered: RenderTuiResult = renderTui(() => h(TextInput, { modelValue: "value" }));
const roleQuery: FrescoRoleQueryOptions = {
  description: /field help/u,
  name: /value/u,
  state: { disabled: false },
};
const textMatcher: FrescoTextMatcher = "value";
const input: FrescoInputDriver = rendered.input;
const frame: string = rendered.lastFrame();
const frames: readonly string[] = rendered.frames;
const snapshot: FrescoFrameSnapshot = rendered.frameSnapshot();
const snapshots: readonly FrescoFrameSnapshot[] = rendered.frameSnapshots;
const protocolNodes: readonly FrescoRenderNode[] = snapshot.protocolNodes;
const roleNode = getByRole(rendered.root, "textbox", roleQuery);
const descriptionNode = getByDescription(rendered.root, /field help/u);
const testNode = getByTestId(rendered.root, "field");
const textNode = getByText(rendered.root, textMatcher);
const roleNodes = queryAllByRole(rendered.root, "textbox", { name: "value" });
const descriptionNodes = queryAllByDescription(rendered.root, "field help");
const testNodes = queryAllByTestId(rendered.root, "field");
const textNodes = queryAllByText(rendered.root, /value/u);

void frame;
void frames;
void snapshot.tree.children;
void protocolNodes[0]?.nodeType;
void snapshots;
void roleNode.props;
void descriptionNode.props;
void testNode.id;
void textNode.type;
void roleNodes;
void descriptionNodes;
void testNodes;
void textNodes;

export const keyFrame: Promise<FrescoFrameSnapshot> = input.key({
  key: "enter",
  ctrl: true,
  eventType: "press",
});
export const textFrame: Promise<FrescoFrameSnapshot> = input.text("abc");
export const pasteFrame: Promise<FrescoFrameSnapshot> = input.paste("pasted");
export const resizeFrame: Promise<FrescoFrameSnapshot> = input.resize(120, 40);
export const mouseFrame: Promise<FrescoFrameSnapshot> = input.mouse({ x: 2, y: 3, button: "left" });
export const focusFrame: Promise<FrescoFrameSnapshot> = input.focus(true);
export const compositionStartFrame: Promise<FrescoFrameSnapshot> = input.compositionStart();
export const compositionUpdateFrame: Promise<FrescoFrameSnapshot> = input.compositionUpdate(
  "かな",
  1,
);
export const compositionEndFrame: Promise<FrescoFrameSnapshot> = input.compositionEnd("かな");

// @ts-expect-error - key event phases are closed.
void input.key({ eventType: "hold" });

// @ts-expect-error - resize dimensions must be numbers.
void input.resize("120", 40);

// @ts-expect-error - mouse injection requires coordinates.
void input.mouse({ button: "left" });

// @ts-expect-error - frame outputs are strings.
const _invalidOutput: number = snapshot.output;

// @ts-expect-error - protocol snapshots use closed render node kinds.
const _invalidProtocolNode: FrescoRenderNode = { id: 1, nodeType: "grid" };

// @ts-expect-error - role names are closed to the Fresco accessibility contract.
void getByRole(rendered.root, "dialog");

// @ts-expect-error - text matchers are exact strings or regular expressions.
void getByText(rendered.root, 123);

// @ts-expect-error - descriptions are exact strings or regular expressions.
void getByDescription(rendered.root, 123);

// @ts-expect-error - test identifiers are strings.
void getByTestId(rendered.root, /field/u);
