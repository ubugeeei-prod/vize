import type { FrescoBorderStyle, FrescoStyle } from "../protocol.js";

/** Props for the public raw frame-buffer extension boundary. */
export interface FrameBufferProps {
  /** Exact terminal rows painted without wrapping or reinterpretation. */
  lines: readonly string[];
  /** Layout applied before the native renderer paints the supplied rows. */
  style?: FrescoStyle;
  /** Optional foreground color shared by every supplied row. */
  fg?: string;
  /** Optional background color shared by every supplied row. */
  bg?: string;
  /** Draw supplied rows with bold terminal cells. @default false */
  bold?: boolean;
  /** Draw supplied rows with dim terminal cells. @default false */
  dim?: boolean;
  /** Draw supplied rows with italic terminal cells. @default false */
  italic?: boolean;
  /** Draw supplied rows with underlined terminal cells. @default false */
  underline?: boolean;
  /** Draw supplied rows with inverted terminal colors. @default false */
  inverse?: boolean;
  /** Optional border around the computed frame-buffer layout. */
  border?: FrescoBorderStyle;
}
