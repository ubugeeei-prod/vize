import type { CSSProperties } from "vue";

/** Direction, glyph, optional title, and color of a terminal divider. */
export interface DividerProps {
  /** Divider direction */
  direction?: "horizontal" | "vertical";
  /** Divider character */
  char?: string;
  /** Title in the middle of the divider */
  title?: string;
  /** Foreground color */
  fg?: string;
  /** Title foreground color */
  titleFg?: string;
}

/** Direction, spacing, alignment, and justification of a flex stack. */
export interface StackProps {
  /** Stack direction */
  direction?: "horizontal" | "vertical";
  /** Gap between children */
  gap?: number;
  /** Align items */
  align?: "start" | "center" | "end" | "stretch";
  /** Justify content */
  justify?: "start" | "center" | "end" | "between" | "around" | "evenly";
  /** Wrap children */
  wrap?: boolean;
}

/** Shared spacing and alignment inputs for fixed-direction stack components. */
export type ConvenienceStackProps = Pick<StackProps, "gap" | "align" | "justify">;

/** Translate public stack alignment names to renderer flex alignment values. */
export const stackAlignValues: Record<
  NonNullable<StackProps["align"]>,
  NonNullable<CSSProperties["alignItems"]>
> = {
  start: "flex-start",
  center: "center",
  end: "flex-end",
  stretch: "stretch",
};

/** Translate public stack justification names to renderer flex distribution values. */
export const stackJustifyValues: Record<
  NonNullable<StackProps["justify"]>,
  NonNullable<CSSProperties["justifyContent"]>
> = {
  start: "flex-start",
  center: "center",
  end: "flex-end",
  between: "space-between",
  around: "space-around",
  evenly: "space-evenly",
};
