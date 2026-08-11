import {
  createLayoutNode,
  flushTerminalMeasured,
  getLayout,
  pollEvent,
  setImeMode,
  type FrameOutputTelemetryNapi,
  type InputEventNapi,
  type LayoutResultNapi,
  type RenderNodeNapi,
} from "@vizejs/fresco-native";

export const nullableStyleNode: number = createLayoutNode(null);
// eslint-disable-next-line typescript-eslint/no-redundant-type-constituents -- resolved in the staged package fixture
export const maybeLayout: LayoutResultNapi | null = getLayout(nullableStyleNode);
// eslint-disable-next-line typescript-eslint/no-redundant-type-constituents -- resolved in the staged package fixture
export const maybeEvent: InputEventNapi | null = pollEvent(0);
export const imeModeResult: void = setImeMode("hiragana");
export const frameOutput: FrameOutputTelemetryNapi = flushTerminalMeasured();
export const changedCells: bigint = frameOutput.changedCells;
export const bytesWritten: bigint = frameOutput.bytesWritten;

export const nativeKeyPhaseIsRustDefined: InputEventNapi = {
  eventType: "key",
  key: "enter",
  keyEventType: "future-native-phase",
};

export const textNode: RenderNodeNapi = {
  id: 1,
  nodeType: "text",
  text: "generated contract",
};

// @ts-expect-error Rust exposes a closed render-node kind union.
export const unknownNode: RenderNodeNapi = { id: 2, nodeType: "grid" };

// @ts-expect-error setImeMode is a side-effect-only native call.
export const booleanImeModeResult: boolean = setImeMode("hiragana");
