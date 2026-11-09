/**
 * Compile-only tests for the public props/emits of the baseline components.
 *
 * Not executed by the test runner; `tsc --noEmit -p tsconfig.json` (wired
 * into `pnpm check` / `pnpm check:types`) fails the build when a positive
 * case stops typechecking or a `@ts-expect-error` negative starts passing.
 */

import { h } from "vue";

import Badge from "./FrescoBadge.vue";
import { Box, type BoxProps } from "./Box.js";
import FrameBuffer from "./FrescoFrameBuffer.vue";
import type { FrameBufferProps } from "./frame-buffer-types.js";
import type {
  BadgeProps,
  LinkProps,
  SpinnerProps,
  StaticProps,
} from "./display-primitive-types.js";
import Link from "./FrescoLink.vue";
import Markdown from "./FrescoMarkdown.vue";
import Modal from "./FrescoModal.vue";
import Alert from "./FrescoAlert.vue";
import Avatar from "./FrescoAvatar.vue";
import Breadcrumb from "./FrescoBreadcrumb.vue";
import Card from "./FrescoCard.vue";
import Divider from "./FrescoDivider.vue";
import Header from "./FrescoHeader.vue";
import HStack from "./FrescoHStack.vue";
import Newline from "./FrescoNewline.vue";
import ScrollBox from "./ScrollBox.vue";
import Spacer from "./FrescoSpacer.vue";
import Spinner from "./FrescoSpinner.vue";
import Stack from "./FrescoStack.vue";
import Static from "./FrescoStatic.vue";
import VStack from "./FrescoVStack.vue";
import type { DividerProps, StackProps } from "./layout-primitive-types.js";
import type {
  AlertProps,
  AvatarProps,
  BreadcrumbProps,
  CardProps,
  HeaderProps,
  TooltipProps,
} from "./presentational-primitive-types.js";
import type { ScrollBoxItemSlot, ScrollBoxProps } from "./scroll-types.js";
import { Text, type TextProps, type TextWrap } from "./Text.js";
import { TextArea, type TextAreaProps } from "./TextArea.js";
import { TextInput, type TextInputProps } from "./TextInput.js";
import { Diff, type DiffProps } from "./Diff.js";
import { LineNumber, type LineNumberProps } from "./LineNumber.js";
import Transform from "./FrescoTransform.vue";
import Tooltip from "./FrescoTooltip.vue";
import type { MarkdownBlockProvider, MarkdownProps } from "./markdown-types.js";
import type { ModalProps } from "./modal-types.js";
import type { TableProps, TableRow } from "./table-types.js";
import { createCodeBuffer, type CodeProps, type CodeSyntaxProvider } from "./Code.js";

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

export const frameBufferAcceptsExactRows: FrameBufferProps = {
  lines: ["first", "second"],
  style: { width: 20, overflow: "hidden" },
  fg: "cyan",
};

// @ts-expect-error - frame-buffer rows are text, not arbitrary payloads.
export const frameBufferRejectsNumericRows: FrameBufferProps = { lines: [1] };

// @ts-expect-error - "ellipsis" is not a TextWrap mode
export const textRejectsUnknownWrap: TextProps = { wrap: "ellipsis" };

// @ts-expect-error - content is a string, not a number
export const textRejectsNumberContent: TextProps = { content: 42 };

export const textInputAcceptsDocumentedProps: TextInputProps = {
  modelValue: "value",
  defaultValue: "fallback",
  placeholder: "type...",
  focus: true,
  focused: true,
  mask: false,
  maskChar: "#",
  width: 20,
  fg: "white",
  bg: "black",
  validate: (value: string) => (value ? true : "required"),
  "onUpdate:modelValue": (value: string) => value,
  onSubmit: (value: string) => value,
  onCancel: () => {},
  onSelectionChange: (start: number, end: number) => start + end,
  onCopy: (value: string) => value,
  onCut: (value: string) => value,
  onValidationChange: (valid: boolean, message?: string) => `${valid}:${message ?? ""}`,
};

// @ts-expect-error - modelValue is a string, not a number
export const textInputRejectsNumberModel: TextInputProps = { modelValue: 42 };

export const textInputRejectsBadSubmitHandler: TextInputProps = {
  // @ts-expect-error - onSubmit receives the submitted string, not a number
  onSubmit: (value: number) => value,
};

export const lineNumberAcceptsDocumentedProps: LineNumberProps = {
  value: 42,
  width: 4,
  separator: "│",
  padding: 1,
  fg: "gray",
  active: true,
  activeFg: "white",
  activeBg: "blue",
};

// @ts-expect-error - line numbers are numeric.
export const lineNumberRejectsStringValue: LineNumberProps = { value: "42" };

export const diffAcceptsDocumentedProps: DiffProps = {
  oldText: "before",
  newText: "after",
  view: "split",
  context: 3,
  showLineNumbers: true,
  viewportStart: 100,
  viewportSize: 20,
  overscan: 2,
  oldLabel: "before.ts",
  newLabel: "after.ts",
};

// @ts-expect-error - diff view has only unified and split modes.
export const diffRejectsUnknownView: DiffProps = { oldText: "a", newText: "b", view: "inline" };

export const staticAcceptsTypedItems: StaticProps<string> = { items: ["done"] };
export const linkAcceptsDocumentedProps: LinkProps = {
  text: "Docs",
  url: "https://example.test",
  showUrl: true,
};
export const markdownProvider: MarkdownBlockProvider = {
  render: (block) =>
    block.type === "thematic-break"
      ? [{ key: "rule", spans: [{ text: "***", dim: true }] }]
      : undefined,
};
export const markdownAcceptsDocumentedProps: MarkdownProps = {
  source: "# Hello",
  headingFg: "cyan",
  providers: [markdownProvider],
};
// @ts-expect-error - Markdown source is text, not an array of rows.
export const markdownRejectsArraySource: MarkdownProps = { source: ["hello"] };
export const badgeAcceptsDocumentedProps: BadgeProps = { label: "Ready", variant: "success" };
export const spinnerAcceptsDocumentedProps: SpinnerProps = {
  type: "line",
  interval: 100,
  label: "Loading",
};
export const dividerAcceptsDocumentedProps: DividerProps = {
  direction: "horizontal",
  char: "=",
  title: "Summary",
  fg: "gray",
  titleFg: "cyan",
};
export const stackAcceptsDocumentedProps: StackProps = {
  direction: "horizontal",
  gap: 2,
  align: "center",
  justify: "between",
  wrap: true,
};
export const alertAcceptsDocumentedProps: AlertProps = { message: "Saved", type: "success" };
export const avatarAcceptsDocumentedProps: AvatarProps = { name: "Ada Lovelace", status: "online" };
export const breadcrumbAcceptsDocumentedProps: BreadcrumbProps = {
  items: [{ key: "home", label: "Home" }],
};
export const cardAcceptsDocumentedProps: CardProps = { title: "Summary", border: "rounded" };
export const headerAcceptsDocumentedProps: HeaderProps = { title: "Dashboard", borderBottom: true };
export const tooltipAcceptsDocumentedProps: TooltipProps = { text: "Help", position: "right" };
export const textAreaAcceptsEditingProps: TextAreaProps = {
  defaultValue: "first\nsecond",
  rows: 2,
  focused: true,
  cursorLine: 1,
  cursorColumn: 3,
  validate: (value) => value.length > 0 || "required",
};
export const modalAcceptsFocusPolicyProps: ModalProps = {
  visible: true,
  focusId: "dialog",
  initialFocusId: "confirm",
  trapFocus: true,
  restoreFocus: true,
  closeOnEscape: true,
};
export const codeSyntaxProvider: CodeSyntaxProvider = {
  tokenize: (line, { language }) => [{ text: line, fg: language === "ts" ? "cyan" : undefined }],
};
export const codeAcceptsRichDisplayProps: CodeProps = {
  buffer: createCodeBuffer("const answer = 42", { maxLines: 1_000 }),
  language: "ts",
  syntaxProvider: codeSyntaxProvider,
  diagnostics: [{ line: 0, startColumn: 6, endColumn: 12, message: "example" }],
  selection: { anchor: { line: 0, column: 0 }, focus: { line: 0, column: 5 } },
  viewportSize: 40,
  overscan: 4,
};

type ScoreRow = TableRow & { readonly name: string; readonly score: number };
export const tableAcceptsInteractiveProps: TableProps<ScoreRow> = {
  columns: [
    { key: "name", header: "Name", width: 12, sortable: true },
    { key: "score", header: "Score", align: "right", minWidth: 3, maxWidth: 8 },
  ],
  data: [{ name: "Ada", score: 10 }],
  sortKey: "score",
  sortDirection: "descending",
  viewportSize: 20,
  overscan: 2,
  focused: true,
  columnWidths: { score: 6 },
};

export const tableRejectsUnknownSortDirection: TableProps = {
  columns: [],
  data: [],
  // @ts-expect-error - sort direction is a controlled ascending/descending state.
  sortDirection: "reverse",
};

export const scrollBoxAcceptsBoundedViewportProps: ScrollBoxProps = {
  itemCount: 100_000,
  viewportSize: 24,
  modelValue: 10,
  overscan: 3,
  focused: true,
  wheelStep: 4,
  showScrollbar: true,
  followOutput: false,
};

export const scrollBoxSlotContract: ScrollBoxItemSlot = {
  index: 10,
  visible: true,
  visibleStart: 10,
  visibleEnd: 34,
  overscanStart: 7,
  overscanEnd: 37,
};

export const scrollBoxRejectsStringViewport: ScrollBoxProps = {
  itemCount: 10,
  // @ts-expect-error - viewport sizes are numeric row counts.
  viewportSize: "5",
};

// --- Component prop contracts through h() ----------------------------------

export function componentsRenderWithTypedProps() {
  return [
    h(Box, { flexDirection: "column", borderStyle: "round", ariaRole: "list" }),
    h(Text, { content: "hi", wrap: "truncate-middle", bold: true }),
    h(TextInput, { modelValue: "v", focused: true, maskChar: "#" }),
    h(TextArea, { modelValue: "first\nsecond", focused: true, rows: 2 }),
    h(Modal, { visible: true, focusId: "dialog", initialFocusId: "confirm" }),
    h(LineNumber, { value: 42, width: 3 }),
    h(Diff, { oldText: "before", newText: "after", view: "unified" }),
    h(ScrollBox, { itemCount: 100, viewportSize: 10 }),
    h(Spacer),
    h(Newline, { count: 2 }),
    h(Transform, { transform: (line: string, index: number) => `${index}:${line}` }),
    h(Static, { items: ["done"] }),
    h(Link, { text: "Docs", showUrl: false }),
    h(Markdown, { source: "# Docs", linkFg: "cyan" }),
    h(Badge, { label: "Ready", variant: "success" }),
    h(FrameBuffer, { lines: ["first", "second"], fg: "cyan" }),
    h(Spinner, { type: "line", label: "Loading" }),
    h(Divider, { title: "Summary", direction: "horizontal" }),
    h(Stack, { direction: "horizontal", gap: 2, align: "center" }),
    h(HStack, { gap: 1, justify: "between" }),
    h(VStack, { gap: 0, align: "stretch" }),
    h(Alert, { message: "Saved", type: "success" }),
    h(Avatar, { name: "Ada Lovelace", status: "online" }),
    h(Breadcrumb, { items: [{ key: "home", label: "Home" }] }),
    h(Card, { title: "Summary" }),
    h(Header, { title: "Dashboard" }),
    h(Tooltip, { text: "Help", visible: true }),
  ];
}

export function textUtilityComponentsRejectMistypedProps() {
  return [
    // @ts-expect-error - count is numeric.
    h(Newline, { count: "2" }),
  ];
}

// @ts-expect-error - transform is required.
export const transformRejectsMissingCallback: InstanceType<typeof Transform>["$props"] = {};

// @ts-expect-error - text is required.
export const linkRejectsMissingText: InstanceType<typeof Link>["$props"] = {};

export function displayPrimitivesRejectMistypedProps() {
  return [
    // @ts-expect-error - unknown is not a Badge variant.
    h(Badge, { label: "Bad", variant: "unknown" }),
    // @ts-expect-error - wave is not a built-in Spinner frame set.
    h(Spinner, { type: "wave" }),
  ];
}

export function layoutPrimitivesRejectMistypedProps() {
  return [
    // @ts-expect-error - diagonal is not a Divider direction.
    h(Divider, { direction: "diagonal" }),
    // @ts-expect-error - left is not a Stack alignment.
    h(Stack, { align: "left" }),
    // @ts-expect-error - gaps are numeric.
    h(HStack, { gap: "1" }),
  ];
}

export function presentationalPrimitivesRejectMistypedProps() {
  return [
    // @ts-expect-error - critical is not an Alert type.
    h(Alert, { message: "Bad", type: "critical" }),
    // @ts-expect-error - diagonal is not a Tooltip position.
    h(Tooltip, { text: "Bad", position: "diagonal" }),
    // @ts-expect-error - Breadcrumb items require stable keys.
    h(Breadcrumb, { items: [{ label: "Missing key" }] }),
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
