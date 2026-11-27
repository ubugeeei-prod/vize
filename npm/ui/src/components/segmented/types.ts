/** Layout hook. Arrows move on both axes either way. */
export type SegmentedOrientation = "horizontal" | "vertical";

/** Public props for the segmented radiogroup. */
export interface SegmentedProps {
  /**
   * Selected value. An empty string selects nothing.
   *
   * @default ""
   */
  readonly modelValue?: string;

  /**
   * Accessible name of the group.
   */
  readonly label: string;

  /**
   * Disable every item.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Visual axis. Keyboard movement does not depend on it.
   *
   * @default "horizontal"
   */
  readonly orientation?: SegmentedOrientation;
}

/** Public props for one exclusive option. */
export interface SegmentedItemProps {
  /**
   * Value selected when this item is activated.
   */
  readonly value: string;

  /**
   * Skip this item for selection and roving focus.
   *
   * @default false
   */
  readonly disabled?: boolean;
}
