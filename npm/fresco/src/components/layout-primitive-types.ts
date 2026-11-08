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

export type ConvenienceStackProps = Pick<StackProps, "gap" | "align" | "justify">;

export const stackAlignValues: Record<
  NonNullable<StackProps["align"]>,
  NonNullable<CSSProperties["alignItems"]>
> = {
  start: "flex-start",
  center: "center",
  end: "flex-end",
  stretch: "stretch",
};

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
import type { CSSProperties } from "vue";
