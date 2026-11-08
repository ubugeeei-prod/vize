/** Public props for the Newline component. */
export interface NewlineProps {
  /** Number of newline characters to insert. Defaults to 1. */
  count?: number;
}

/** Public props for the Transform component. */
export interface TransformProps {
  /** Screen-reader-specific label, accepted for Ink API parity. */
  accessibilityLabel?: string;
  /** Transform each rendered line. */
  transform: (children: string, index: number) => string;
}
