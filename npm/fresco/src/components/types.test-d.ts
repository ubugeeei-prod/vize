/**
 * Compile-only tests for the public props/emits of the baseline components.
 *
 * Not executed by the test runner; `tsc --noEmit -p tsconfig.json` (wired
 * into `pnpm check` / `pnpm check:types`) fails the build when a positive
 * case stops typechecking or a `@ts-expect-error` negative starts passing.
 */

import { h } from "@vue/runtime-core";

import { Box, type BoxProps } from "./Box.js";
import { Text, type TextProps, type TextWrap } from "./Text.js";
import { TextInput, type TextInputProps } from "./TextInput.js";

// --- Public prop interfaces ------------------------------------------------

export const boxAcceptsDocumentedProps: BoxProps = {
  display: "flex",
  position: "relative",
  flexDirection: "column",
  justifyContent: "space-between",
  alignItems: "center",
  flexGrow: 1,
  width: "50%",
  height: 10,
  padding: 1,
  paddingX: 2,
  marginY: 1,
  gap: 1,
  overflow: "hidden",
  border: "round",
  borderStyle: "double",
  borderColor: "cyan",
  fg: "white",
  bg: "black",
  "aria-label": "panel",
  "aria-hidden": false,
  "aria-role": "list",
  "aria-state": { expanded: true },
};

// @ts-expect-error - "grid" is not a BoxProps flexDirection
export const boxRejectsUnknownFlexDirection: BoxProps = { flexDirection: "grid" };

// @ts-expect-error - "dotted" is not a supported border style name
export const boxRejectsUnknownBorderStyle: BoxProps = { borderStyle: "dotted" };

// @ts-expect-error - padding shorthands are numbers, not strings
export const boxRejectsStringPadding: BoxProps = { padding: "2" };

export const textAcceptsDocumentedProps: TextProps = {
  content: "hello",
  wrap: "truncate-middle",
  fg: "red",
  color: "red",
  bg: "blue",
  backgroundColor: "blue",
  bold: true,
  dim: false,
  dimColor: false,
  italic: true,
  underline: true,
  strikethrough: false,
  inverse: false,
  "aria-label": "greeting",
  "aria-hidden": false,
};

export const wrapAcceptsBooleans: TextWrap = true;

// @ts-expect-error - "ellipsis" is not a TextWrap mode
export const textRejectsUnknownWrap: TextProps = { wrap: "ellipsis" };

// @ts-expect-error - content is a string, not a number
export const textRejectsNumberContent: TextProps = { content: 42 };

export const textInputAcceptsDocumentedProps: TextInputProps = {
  modelValue: "value",
  placeholder: "type...",
  focus: true,
  focused: true,
  mask: false,
  maskChar: "#",
  width: 20,
  fg: "white",
  bg: "black",
  "onUpdate:modelValue": (value: string) => value,
  onSubmit: (value: string) => value,
  onCancel: () => {},
};

// @ts-expect-error - modelValue is a string, not a number
export const textInputRejectsNumberModel: TextInputProps = { modelValue: 42 };

export const textInputRejectsBadSubmitHandler: TextInputProps = {
  // @ts-expect-error - onSubmit receives the submitted string, not a number
  onSubmit: (value: number) => value,
};

// --- Component prop contracts through h() ----------------------------------

export function componentsRenderWithTypedProps() {
  return [
    h(Box, { flexDirection: "column", borderStyle: "round", ariaRole: "list" }),
    h(Text, { content: "hi", wrap: "truncate-middle", bold: true }),
    h(TextInput, { modelValue: "v", focused: true, maskChar: "#" }),
  ];
}

export function boxRejectsMistypedRenderProps() {
  // @ts-expect-error - flexGrow is a number, not a string
  return h(Box, { flexGrow: "1" });
}

export function textRejectsMistypedRenderProps() {
  // @ts-expect-error - bold is a boolean, not a string
  return h(Text, { bold: "yes" });
}

// --- Emits -----------------------------------------------------------------

type TextInputEmit = InstanceType<typeof TextInput>["$emit"];

export function textInputEmitsDocumentedEvents(emit: TextInputEmit) {
  emit("update:modelValue", "next");
  emit("submit", "value");
  emit("cancel");
  emit("compositionstart");
  emit("compositionupdate", "text", 1);
  emit("compositionend", "text");
  // @ts-expect-error - "change" is not a declared TextInput event
  emit("change");
}
