/** Cancelable selection event emitted by ActionSheetItem. */
export interface ActionSheetSelectEvent {
  /** Value of the selected action. */
  readonly value: string;

  /** Native click or keyboard event that selected the action. */
  readonly nativeEvent: Event;

  /** Keep the sheet open after this selection. */
  readonly preventDefault: () => void;

  /** Whether `preventDefault` was called. */
  readonly defaultPrevented: boolean;
}
