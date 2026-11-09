import type { AriaRole } from "../accessibility.js";

/** Placement of the Popover panel relative to its anchor slot. */
export type PopoverPlacement = "top" | "right" | "bottom" | "left";

/** Public focus, dismissal, placement, and appearance contract for Popover. */
export interface PopoverProps {
  /** Whether the panel is mounted and owns its focus boundary. */
  open?: boolean;
  /** Placement of the panel relative to the anchor. @default "bottom" */
  placement?: PopoverPlacement;
  /** Gap in terminal cells between the anchor and panel. @default 0 */
  offset?: number;
  /** Panel width. */
  width?: number | string;
  /** Panel height. */
  height?: number | string;
  /** Panel border style. @default "rounded" */
  border?: "single" | "double" | "rounded" | "heavy" | "none";
  /** Panel foreground color. */
  fg?: string;
  /** Panel background color. */
  bg?: string;
  /** Stable focus-boundary ID. */
  focusId?: string;
  /** Descendant focus ID selected after the panel opens. */
  initialFocusId?: string;
  /** Constrain sequential and programmatic focus to the popover subtree. @default true */
  trapFocus?: boolean;
  /** Restore the target focused before the panel opened. @default true */
  restoreFocus?: boolean;
  /** Emit `close` when Escape is pressed. @default true */
  closeOnEscape?: boolean;
  /** Semantic role exposed by the panel. @default "dialog" */
  accessibilityRole?: AriaRole;
  /** Accessible panel label. @default "Popover" */
  accessibilityLabel?: string;
}
