/** Public props for the input group. */
export interface InputGroupProps {
  /**
   * Accessible name of the group.
   */
  readonly label: string;

  /**
   * Disable every field and button in the group.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/** Public props for the text field. */
export interface InputGroupInputProps {
  /**
   * Current value.
   *
   * @default ""
   */
  readonly modelValue?: string;

  /**
   * Native input type.
   *
   * @default "text"
   */
  readonly type?: "text" | "email" | "url" | "search" | "tel" | "password";

  /**
   * Disable this field even when the group is enabled.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Native form name.
   *
   * @default undefined
   */
  readonly name?: string;
}

/** Public props for the textarea. */
export interface InputGroupTextareaProps {
  /**
   * Current value.
   *
   * @default ""
   */
  readonly modelValue?: string;

  /**
   * Visible row count.
   *
   * @default 2
   */
  readonly rows?: number;

  /**
   * Disable this field even when the group is enabled.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Native form name.
   *
   * @default undefined
   */
  readonly name?: string;
}

/** Public props for the group button. */
export interface InputGroupButtonProps {
  /**
   * Disable this button even when the group is enabled.
   *
   * @default false
   */
  readonly disabled?: boolean;
}
