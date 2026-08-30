import type {
  SpacerAxis,
  SpacerDisplay,
  SpacerResolvedLayout,
  SpacerSize,
} from "./spacer-types.ts";

export const SPACER_DEFAULT_AXIS = "block" satisfies SpacerAxis;
export const SPACER_DEFAULT_SIZE = "1rem" satisfies SpacerSize;
export const SPACER_CROSS_AXIS_SIZE = "auto" satisfies SpacerSize;

interface SpacerLayoutOptions {
  readonly axis?: SpacerAxis;
  readonly blockSize?: SpacerSize | undefined;
  readonly display?: SpacerDisplay | undefined;
  readonly inlineSize?: SpacerSize | undefined;
  readonly size?: SpacerSize | undefined;
}

function defaultSpacerDisplay(axis: SpacerAxis): SpacerDisplay {
  return axis === "block" ? "block" : "inline-block";
}

function resolveInlineSize(axis: SpacerAxis, size: SpacerSize): SpacerSize {
  return axis === "inline" || axis === "both" ? size : SPACER_CROSS_AXIS_SIZE;
}

function resolveBlockSize(axis: SpacerAxis, size: SpacerSize): SpacerSize {
  return axis === "block" || axis === "both" ? size : SPACER_CROSS_AXIS_SIZE;
}

/** Resolve public spacer props into a native CSS logical-size contract. */
export function resolveSpacerLayout(options: SpacerLayoutOptions): SpacerResolvedLayout {
  const axis = options.axis ?? SPACER_DEFAULT_AXIS;
  const size = options.size ?? SPACER_DEFAULT_SIZE;
  const inlineSize = options.inlineSize ?? resolveInlineSize(axis, size);
  const blockSize = options.blockSize ?? resolveBlockSize(axis, size);
  const display = options.display ?? defaultSpacerDisplay(axis);

  return {
    axis,
    blockSize,
    display,
    inlineSize,
    state: "sized",
    style: {
      "--vize-ui-spacer-block-size": blockSize,
      "--vize-ui-spacer-inline-size": inlineSize,
      blockSize: "var(--vize-ui-spacer-block-size)",
      display,
      inlineSize: "var(--vize-ui-spacer-inline-size)",
    },
  };
}
