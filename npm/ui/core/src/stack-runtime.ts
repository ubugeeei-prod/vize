import type {
  StackAlign,
  StackAxis,
  StackFlexDirection,
  StackGap,
  StackJustify,
  StackResolvedLayout,
} from "./stack-types.ts";

export const STACK_DEFAULT_AXIS = "block" satisfies StackAxis;
export const STACK_DEFAULT_GAP = "1rem" satisfies StackGap;
export const STACK_DEFAULT_ALIGN = "stretch" satisfies StackAlign;
export const STACK_DEFAULT_JUSTIFY = "start" satisfies StackJustify;

interface StackLayoutOptions {
  readonly align?: StackAlign | undefined;
  readonly axis?: StackAxis | undefined;
  readonly gap?: StackGap | undefined;
  readonly justify?: StackJustify | undefined;
  readonly reversed?: boolean | undefined;
}

function resolveFlexDirection(axis: StackAxis, reversed: boolean): StackFlexDirection {
  if (axis === "inline") return reversed ? "row-reverse" : "row";
  return reversed ? "column-reverse" : "column";
}

/** Resolve public Stack props into a native CSS flexbox contract. */
export function resolveStackLayout(options: StackLayoutOptions): StackResolvedLayout {
  const align = options.align ?? STACK_DEFAULT_ALIGN;
  const axis = options.axis ?? STACK_DEFAULT_AXIS;
  const gap = options.gap ?? STACK_DEFAULT_GAP;
  const justify = options.justify ?? STACK_DEFAULT_JUSTIFY;
  const reversed = options.reversed ?? false;
  const direction = resolveFlexDirection(axis, reversed);

  return {
    align,
    axis,
    direction,
    gap,
    justify,
    reversed,
    state: "stacked",
    style: {
      "--vize-ui-stack-align": align,
      "--vize-ui-stack-gap": gap,
      "--vize-ui-stack-justify": justify,
      alignItems: "var(--vize-ui-stack-align)",
      display: "flex",
      flexDirection: direction,
      gap: "var(--vize-ui-stack-gap)",
      justifyContent: "var(--vize-ui-stack-justify)",
    },
  };
}
