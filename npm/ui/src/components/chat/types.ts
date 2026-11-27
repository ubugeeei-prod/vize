/** Public props for the message log. */
export interface ChatProps {
  /**
   * Accessible name of the log.
   *
   * @default "Conversation"
   */
  readonly label?: string;
}

/** Public props for one message. */
export interface ChatMessageProps {
  /**
   * Name announced for this message.
   */
  readonly author: string;
}

/** Public props for the composer. */
export interface ChatComposerProps {
  /**
   * Accessible name of the text field.
   *
   * @default "Message"
   */
  readonly label?: string;

  /**
   * Label of the submit button.
   *
   * @default "Send"
   */
  readonly submitLabel?: string;

  /**
   * Disable the field and the button.
   *
   * @default false
   */
  readonly disabled?: boolean;
}
