/**
 * Fresco Components
 */

// Layout
export { Box, type BoxProps } from "./Box.js";
export { default as Spacer } from "./FrescoSpacer.vue";
export { default as Divider } from "./FrescoDivider.vue";
export { default as Stack } from "./FrescoStack.vue";
export { default as HStack } from "./FrescoHStack.vue";
export { default as VStack } from "./FrescoVStack.vue";
export type { DividerProps, StackProps } from "./layout-primitive-types.js";
export { Grid, type GridProps } from "./Grid.js";
export { default as Card } from "./FrescoCard.vue";
export type { CardProps } from "./presentational-primitive-types.js";
export { default as ScrollBox } from "./ScrollBox.vue";
export { default as ScrollBar } from "./ScrollBar.vue";
export {
  type ScrollBarOrientation,
  type ScrollBarProps,
  type ScrollBoxExposed,
  type ScrollBoxItemSlot,
  type ScrollBoxProps,
} from "./scroll-types.js";

// Text
export { Text, type TextProps } from "./Text.js";
export { default as Newline } from "./FrescoNewline.vue";
export type { NewlineProps } from "./text-utility-types.js";
export { default as Static } from "./FrescoStatic.vue";
export type { StaticProps } from "./display-primitive-types.js";
export { default as Transform } from "./FrescoTransform.vue";
export type { TransformProps } from "./text-utility-types.js";
export { Code, type CodeProps } from "./Code.js";
export { default as Markdown } from "./FrescoMarkdown.vue";
export { defaultMarkdownRenderer, parseMarkdown, renderMarkdown } from "./markdown.js";
export type {
  MarkdownBlock,
  MarkdownBlockProvider,
  MarkdownCodeBlock,
  MarkdownHeadingBlock,
  MarkdownListBlock,
  MarkdownListItem,
  MarkdownParagraphBlock,
  MarkdownProps,
  MarkdownQuoteBlock,
  MarkdownRenderer,
  MarkdownRenderLine,
  MarkdownRenderOptions,
  MarkdownSpan,
  MarkdownThematicBreakBlock,
} from "./markdown-types.js";
export { default as FrameBuffer } from "./FrescoFrameBuffer.vue";
export type { FrameBufferProps } from "./frame-buffer-types.js";
export { LineNumber, type LineNumberProps } from "./LineNumber.js";
export { default as Link } from "./FrescoLink.vue";
export type { LinkProps } from "./display-primitive-types.js";

// Input
export { TextInput, type TextInputProps } from "./TextInput.js";
export { TextArea, type TextAreaProps } from "./TextArea.js";
export { Select, type SelectProps, type SelectOption } from "./Select.js";
export { Checkbox, type CheckboxProps } from "./Checkbox.js";
export { RadioGroup, type RadioGroupProps, type RadioOption } from "./RadioGroup.js";
export { Confirm, type ConfirmProps } from "./Confirm.js";
export { Form, type FormProps, type FormField } from "./Form.js";

// Feedback
export { default as Spinner } from "./FrescoSpinner.vue";
export type { SpinnerProps } from "./display-primitive-types.js";
export { ProgressBar, type ProgressBarProps } from "./ProgressBar.js";
export { default as Alert } from "./FrescoAlert.vue";
export type { AlertProps, AlertType } from "./presentational-primitive-types.js";
export { default as Badge } from "./FrescoBadge.vue";
export type { BadgeProps, BadgeVariant } from "./display-primitive-types.js";
export { Timer, type TimerProps, type TimerMode } from "./Timer.js";
export { default as Tooltip } from "./FrescoTooltip.vue";
export type { TooltipPosition, TooltipProps } from "./presentational-primitive-types.js";

// Data Display
export { List, type ListProps, type ListItem } from "./List.js";
export { Table, type TableProps, type TableColumn } from "./Table.js";
export { Tree, type TreeProps, type TreeNode } from "./Tree.js";

// Navigation
export { Menu, type MenuProps, type MenuItem } from "./Menu.js";
export { Tabs, type TabsProps, type Tab } from "./Tabs.js";
export { default as Breadcrumb } from "./FrescoBreadcrumb.vue";
export type { BreadcrumbItem, BreadcrumbProps } from "./presentational-primitive-types.js";
export { Stepper, type StepperProps, type Step, type StepStatus } from "./Stepper.js";

// Overlay
export { Modal, type ModalProps } from "./Modal.js";

// Status
export { StatusBar, type StatusBarProps, type StatusBarItem } from "./StatusBar.js";
export { default as Header } from "./FrescoHeader.vue";
export type { HeaderProps } from "./presentational-primitive-types.js";
export { KeyHint, type KeyHintProps, type KeyBinding } from "./KeyHint.js";
export { default as Avatar } from "./FrescoAvatar.vue";
export type { AvatarProps } from "./presentational-primitive-types.js";
