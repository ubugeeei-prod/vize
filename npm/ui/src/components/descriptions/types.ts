/** Public props for the description list. */
export interface DescriptionsProps {
  /**
   * Accessible name of the list.
   */
  readonly label: string;
}

/** Public props for one term and detail pair. */
export interface DescriptionsItemProps {
  /**
   * Term announced for the detail.
   */
  readonly label: string;
}

/** State passed to a description item's default slot. */
export interface DescriptionsItemSlotProps {
  readonly label: string;
}
