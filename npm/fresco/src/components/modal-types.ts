/** Public behavior and appearance contract for the Fresco Modal. */
export interface ModalProps {
  /** Whether the modal is visible and owns its focus scope. */
  visible?: boolean;
  /** Modal title and default accessible label. */
  title?: string;
  /** Modal width. */
  width?: number | string;
  /** Modal height. */
  height?: number | string;
  /** Border style. */
  border?: "single" | "double" | "rounded" | "heavy";
  /** Title foreground color. */
  titleFg?: string;
  /** Border foreground color. */
  borderFg?: string;
  /** Background color. */
  bg?: string;
  /** Stable focus boundary ID. */
  focusId?: string;
  /** Descendant focus ID selected when the modal opens. */
  initialFocusId?: string;
  /** Constrain sequential and programmatic focus to modal descendants. */
  trapFocus?: boolean;
  /** Restore the target focused before the modal opened. */
  restoreFocus?: boolean;
  /** Emit close when Escape is pressed. */
  closeOnEscape?: boolean;
  /** Accessible label when no visible title is suitable. */
  accessibilityLabel?: string;
}
