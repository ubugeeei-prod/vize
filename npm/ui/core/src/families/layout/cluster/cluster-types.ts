import type { ComponentPublicInstance } from "vue";

/** Native CSS `align-items` values supported by {@link Cluster}. */
export type ClusterAlign = "stretch" | "start" | "center" | "end" | "baseline";

/** Native CSS `justify-content` values supported by {@link Cluster}. */
export type ClusterJustify =
  | "start"
  | "center"
  | "end"
  | "space-between"
  | "space-around"
  | "space-evenly";

/** Native CSS `gap` value accepted by {@link Cluster}. Numbers resolve to px lengths. */
export type ClusterGap = string | number;

/** CSS-ready `gap` value published after numeric gaps are normalized. */
export type ClusterResolvedGap = string;

/** CSS `flex-direction` produced by the inline cluster flow. */
export type ClusterFlexDirection = "row" | "row-reverse";

/** CSS `flex-wrap` value produced by the Cluster wrap flag. */
export type ClusterFlexWrap = "wrap" | "nowrap";

/** Rendered data state for {@link Cluster}. */
export type ClusterState = "clustered";

/** Rendered value exposed by {@link Cluster}. */
export type ClusterElement = Element | ComponentPublicInstance;

/** State exposed to the default Cluster slot. */
export interface ClusterSlotState {
  /** Whether items can wrap onto additional lines. */
  readonly wrap: boolean;

  /** Whether inline item flow is reversed without changing DOM order. */
  readonly reversed: boolean;

  /** CSS `flex-direction` applied to the rendered host. */
  readonly direction: ClusterFlexDirection;

  /** CSS `flex-wrap` applied to the rendered host. */
  readonly wrapMode: ClusterFlexWrap;

  /** Resolved CSS `gap` value between direct children. */
  readonly gap: ClusterResolvedGap;

  /** Native CSS `align-items` value for the cross axis. */
  readonly align: ClusterAlign;

  /** Native CSS `justify-content` value for the inline axis. */
  readonly justify: ClusterJustify;

  /** Stable state token for styling and tests. */
  readonly state: ClusterState;
}

/** Inline style hooks applied to the rendered Cluster host. */
export interface ClusterStyle {
  /** Consumer-overridable gap hook read by the host `gap` declaration. */
  readonly "--vize-ui-cluster-gap": ClusterResolvedGap;

  /** Consumer-overridable alignment hook read by the host `align-items` declaration. */
  readonly "--vize-ui-cluster-align": ClusterAlign;

  /** Consumer-overridable justification hook read by the host `justify-content` declaration. */
  readonly "--vize-ui-cluster-justify": ClusterJustify;

  /** Flex layout mode for wrapping inline clusters. */
  readonly display: "flex";

  /** CSS inline flow direction. */
  readonly flexDirection: ClusterFlexDirection;

  /** CSS line wrapping behavior. */
  readonly flexWrap: ClusterFlexWrap;

  /** Native child spacing declaration. */
  readonly gap: string;

  /** Native cross-axis alignment declaration. */
  readonly alignItems: string;

  /** Native inline-axis distribution declaration. */
  readonly justifyContent: string;
}

/** Resolved layout state published by {@link Cluster}. */
export interface ClusterResolvedLayout extends ClusterSlotState {
  /** Native flexbox style object applied to the host. */
  readonly style: ClusterStyle;
}

/** Public instance state exposed by the Cluster primitive. */
export interface ClusterExpose extends ClusterSlotState {
  /** Rendered host element or component instance. */
  readonly element: ClusterElement | null;
}
