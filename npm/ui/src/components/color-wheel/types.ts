/** Public props for the hue wheel. */
export interface ColorWheelProps {
  /**
   * Hue in degrees. `0` and `360` are the same color; `End` sets `360`.
   *
   * @default 0
   */
  readonly modelValue?: number;

  /**
   * Accessible name of the slider.
   *
   * @default "Hue"
   */
  readonly label?: string;

  /**
   * Ignore pointer and keyboard changes.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/** Pointed hue published to the default slot. */
export interface ColorWheelSlotProps {
  readonly hue: number;
  readonly color: string;
}
