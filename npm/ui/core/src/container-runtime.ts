import type {
  ContainerLength,
  ContainerResolvedLayout,
  ContainerResolvedLength,
  ContainerSize,
  ContainerStyle,
} from "./container-types.ts";

export const CONTAINER_DEFAULT_SIZE = "md" satisfies ContainerSize;
export const CONTAINER_DEFAULT_PADDING_INLINE = 0 satisfies ContainerLength;

export const CONTAINER_MAX_INLINE_SIZES = {
  xs: "36rem",
  sm: "48rem",
  md: "64rem",
  lg: "80rem",
  xl: "96rem",
  full: "none",
} as const satisfies Record<ContainerSize, ContainerResolvedLength>;

interface ContainerLayoutOptions {
  readonly centered?: boolean | undefined;
  readonly maxInlineSize?: ContainerLength | undefined;
  readonly paddingInline?: ContainerLength | undefined;
  readonly size?: ContainerSize | undefined;
}

function normalizeContainerLength(length: ContainerLength): ContainerResolvedLength {
  if (typeof length === "number") return length === 0 ? "0" : `${length}px`;
  return length;
}

/** Resolve public Container props into a native CSS logical sizing contract. */
export function resolveContainerLayout(options: ContainerLayoutOptions): ContainerResolvedLayout {
  const size = options.size ?? CONTAINER_DEFAULT_SIZE;
  const maxInlineSize = normalizeContainerLength(
    options.maxInlineSize ?? CONTAINER_MAX_INLINE_SIZES[size],
  );
  const paddingInline = normalizeContainerLength(
    options.paddingInline ?? CONTAINER_DEFAULT_PADDING_INLINE,
  );
  const centered = options.centered ?? true;
  const baseStyle = {
    "--vize-ui-container-max-inline-size": maxInlineSize,
    "--vize-ui-container-padding-inline": paddingInline,
    maxInlineSize: "var(--vize-ui-container-max-inline-size)",
    paddingInline: "var(--vize-ui-container-padding-inline)",
  } satisfies ContainerStyle;
  const style: ContainerStyle = centered ? { ...baseStyle, marginInline: "auto" } : baseStyle;

  return {
    centered,
    maxInlineSize,
    paddingInline,
    size,
    style,
  };
}
