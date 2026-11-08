/**
 * Fresco Components
 */

// Layout
export { Box, type BoxProps } from "./Box.js";
export { default as Spacer } from "./FrescoSpacer.vue";
export { Divider, type DividerProps } from "./Divider.js";
export { Stack, HStack, VStack, type StackProps } from "./Stack.js";
export { Grid, type GridProps } from "./Grid.js";
export { Card, type CardProps } from "./Card.js";
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
export { default as FrameBuffer } from "./FrescoFrameBuffer.vue";
export type { FrameBufferProps } from "./frame-buffer-types.js";
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
export { Alert, type AlertProps, type AlertType } from "./Alert.js";
export { default as Badge } from "./FrescoBadge.vue";
export type { BadgeProps, BadgeVariant } from "./display-primitive-types.js";
export { Timer, type TimerProps, type TimerMode } from "./Timer.js";
export { Tooltip, type TooltipProps, type TooltipPosition } from "./Tooltip.js";

// Data Display
export { List, type ListProps, type ListItem } from "./List.js";
export { Table, type TableProps, type TableColumn } from "./Table.js";
export { Tree, type TreeProps, type TreeNode } from "./Tree.js";

// Navigation
export { Menu, type MenuProps, type MenuItem } from "./Menu.js";
export { Tabs, type TabsProps, type Tab } from "./Tabs.js";
export { Breadcrumb, type BreadcrumbProps, type BreadcrumbItem } from "./Breadcrumb.js";
export { Stepper, type StepperProps, type Step, type StepStatus } from "./Stepper.js";

// Overlay
export { Modal, type ModalProps } from "./Modal.js";

// Status
export { StatusBar, type StatusBarProps, type StatusBarItem } from "./StatusBar.js";
export { Header, type HeaderProps } from "./Header.js";
export { KeyHint, type KeyHintProps, type KeyBinding } from "./KeyHint.js";
export { Avatar, type AvatarProps } from "./Avatar.js";
