import type {
  ClusterAlign,
  ClusterFlexDirection,
  ClusterFlexWrap,
  ClusterGap,
  ClusterJustify,
  ClusterResolvedGap,
  ClusterResolvedLayout,
} from "./cluster-types.ts";

export const CLUSTER_DEFAULT_GAP = 0 satisfies ClusterGap;
export const CLUSTER_DEFAULT_ALIGN = "stretch" satisfies ClusterAlign;
export const CLUSTER_DEFAULT_JUSTIFY = "start" satisfies ClusterJustify;

interface ClusterLayoutOptions {
  readonly align?: ClusterAlign | undefined;
  readonly gap?: ClusterGap | undefined;
  readonly justify?: ClusterJustify | undefined;
  readonly reversed?: boolean | undefined;
  readonly wrap?: boolean | undefined;
}

function normalizeClusterGap(gap: ClusterGap): ClusterResolvedGap {
  if (typeof gap === "number") return gap === 0 ? "0" : `${gap}px`;
  return gap;
}

function resolveFlexDirection(reversed: boolean): ClusterFlexDirection {
  return reversed ? "row-reverse" : "row";
}

function resolveFlexWrap(wrap: boolean): ClusterFlexWrap {
  return wrap ? "wrap" : "nowrap";
}

/** Resolve public Cluster props into a native CSS flexbox contract. */
export function resolveClusterLayout(options: ClusterLayoutOptions): ClusterResolvedLayout {
  const align = options.align ?? CLUSTER_DEFAULT_ALIGN;
  const gap = normalizeClusterGap(options.gap ?? CLUSTER_DEFAULT_GAP);
  const justify = options.justify ?? CLUSTER_DEFAULT_JUSTIFY;
  const reversed = options.reversed ?? false;
  const wrap = options.wrap ?? true;
  const direction = resolveFlexDirection(reversed);
  const wrapMode = resolveFlexWrap(wrap);

  return {
    align,
    direction,
    gap,
    justify,
    reversed,
    state: "clustered",
    wrap,
    wrapMode,
    style: {
      "--vize-ui-cluster-align": align,
      "--vize-ui-cluster-gap": gap,
      "--vize-ui-cluster-justify": justify,
      alignItems: "var(--vize-ui-cluster-align)",
      display: "flex",
      flexDirection: direction,
      flexWrap: wrapMode,
      gap: "var(--vize-ui-cluster-gap)",
      justifyContent: "var(--vize-ui-cluster-justify)",
    },
  };
}
