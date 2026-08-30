import type { ComponentPublicInstance } from "vue";

/** Native CSS `align-items` values supported by {@link Grid}. */
export type GridAlign = "stretch" | "start" | "center" | "end" | "baseline";

/** Native CSS `justify-items` values supported by {@link Grid}. */
export type GridJustify = "stretch" | "start" | "center" | "end";

/** Native CSS `grid-auto-flow` values supported by {@link Grid}. */
export type GridAutoFlow = "row" | "column" | "dense" | "row dense" | "column dense";

/** CSS grid template columns accepted by {@link Grid}. Numbers resolve to equal fr tracks. */
export type GridColumns = string | number;

/** Native CSS gap value accepted by {@link Grid}. Numbers resolve to px lengths. */
export type GridGap = string | number;

/** CSS-ready grid template columns published after numeric columns are normalized. */
export type GridResolvedColumns = string;

/** CSS-ready gap value published after numeric gaps are normalized. */
export type GridResolvedGap = string;

/** Rendered value exposed by {@link Grid}. */
export type GridElement = Element | ComponentPublicInstance;

/** Inline style hooks applied to the rendered Grid host. */
export interface GridStyle {
  /** Consumer-overridable column hook read by the host `grid-template-columns` declaration. */
  readonly "--vize-ui-grid-columns": GridResolvedColumns;

  /** Consumer-overridable gap hook read by the host `gap` declaration. */
  readonly "--vize-ui-grid-gap": GridResolvedGap;

  /** Consumer-overridable row gap hook read by the host `row-gap` declaration. */
  readonly "--vize-ui-grid-row-gap": GridResolvedGap;

  /** Consumer-overridable column gap hook read by the host `column-gap` declaration. */
  readonly "--vize-ui-grid-column-gap": GridResolvedGap;

  /** Consumer-overridable alignment hook read by the host `align-items` declaration. */
  readonly "--vize-ui-grid-align": GridAlign;

  /** Consumer-overridable justification hook read by the host `justify-items` declaration. */
  readonly "--vize-ui-grid-justify": GridJustify;

  /** Consumer-overridable auto-flow hook read by the host `grid-auto-flow` declaration. */
  readonly "--vize-ui-grid-auto-flow": GridAutoFlow;

  /** Native grid layout mode. */
  readonly display: "grid";

  /** Native track list declaration. */
  readonly gridTemplateColumns: string;

  /** Native auto-placement declaration. */
  readonly gridAutoFlow: string;

  /** Native child spacing declaration. */
  readonly gap: string;

  /** Native row spacing declaration. */
  readonly rowGap: string;

  /** Native column spacing declaration. */
  readonly columnGap: string;

  /** Native block-axis item alignment declaration. */
  readonly alignItems: string;

  /** Native inline-axis item alignment declaration. */
  readonly justifyItems: string;
}

/** State exposed to the default Grid slot. */
export interface GridSlotState {
  /** Resolved CSS grid template columns value. */
  readonly columns: GridResolvedColumns;

  /** Resolved CSS gap value between direct children. */
  readonly gap: GridResolvedGap;

  /** Resolved CSS row gap value between direct children. */
  readonly rowGap: GridResolvedGap;

  /** Resolved CSS column gap value between direct children. */
  readonly columnGap: GridResolvedGap;

  /** Native CSS `align-items` value for grid items. */
  readonly align: GridAlign;

  /** Native CSS `justify-items` value for grid items. */
  readonly justify: GridJustify;

  /** Native CSS `grid-auto-flow` value for auto placement. */
  readonly autoFlow: GridAutoFlow;

  /** Native CSS grid style object applied to the host. */
  readonly style: GridStyle;
}

/** Resolved layout state published by {@link Grid}. */
export type GridResolvedLayout = GridSlotState;

/** Public instance state exposed by the Grid primitive. */
export interface GridExpose extends GridSlotState {
  /** Rendered host element or component instance. */
  readonly element: GridElement | null;
}
