/** Compile-only assertions for the public grid contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  GridAlign,
  GridAutoFlow,
  GridColumns,
  GridElement,
  GridExpose,
  GridGap,
  GridJustify,
  GridResolvedColumns,
  GridResolvedGap,
  GridResolvedLayout,
  GridSlotState,
  GridStyle,
} from "./grid.ts";
import { Grid } from "./grid.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: GridExpose;

type _AlignIsLogical = Expect<
  Equal<GridAlign, "stretch" | "start" | "center" | "end" | "baseline">
>;
type _JustifyIsLogical = Expect<Equal<GridJustify, "stretch" | "start" | "center" | "end">>;
type _AutoFlowIsNativeGrid = Expect<
  Equal<GridAutoFlow, "row" | "column" | "dense" | "row dense" | "column dense">
>;
type _ElementIsRenderable = Expect<Equal<GridElement, Element | ComponentPublicInstance>>;
type _ColumnsAcceptCssStringOrNumber = Expect<Equal<GridColumns, string | number>>;
type _GapAcceptsCssStringOrNumber = Expect<Equal<GridGap, string | number>>;
type _ResolvedColumnsAreCssString = Expect<Equal<GridResolvedColumns, string>>;
type _ResolvedGapIsCssString = Expect<Equal<GridResolvedGap, string>>;
type _StyleIsNativeGridOnly = Expect<
  Equal<
    GridStyle,
    {
      readonly "--vize-ui-grid-columns": GridResolvedColumns;
      readonly "--vize-ui-grid-gap": GridResolvedGap;
      readonly "--vize-ui-grid-row-gap": GridResolvedGap;
      readonly "--vize-ui-grid-column-gap": GridResolvedGap;
      readonly "--vize-ui-grid-align": GridAlign;
      readonly "--vize-ui-grid-justify": GridJustify;
      readonly "--vize-ui-grid-auto-flow": GridAutoFlow;
      readonly display: "grid";
      readonly gridTemplateColumns: string;
      readonly gridAutoFlow: string;
      readonly gap: string;
      readonly rowGap: string;
      readonly columnGap: string;
      readonly alignItems: string;
      readonly justifyItems: string;
    }
  >
>;
type _SlotStateIsStable = Expect<
  Equal<
    GridSlotState,
    {
      readonly columns: GridResolvedColumns;
      readonly gap: GridResolvedGap;
      readonly rowGap: GridResolvedGap;
      readonly columnGap: GridResolvedGap;
      readonly align: GridAlign;
      readonly justify: GridJustify;
      readonly autoFlow: GridAutoFlow;
      readonly style: GridStyle;
    }
  >
>;
type _ExposeStateMatchesSlot = Expect<
  Equal<
    Omit<GridExpose, "element">,
    {
      readonly columns: GridResolvedColumns;
      readonly gap: GridResolvedGap;
      readonly rowGap: GridResolvedGap;
      readonly columnGap: GridResolvedGap;
      readonly align: GridAlign;
      readonly justify: GridJustify;
      readonly autoFlow: GridAutoFlow;
      readonly style: GridStyle;
    }
  >
>;

const exposedElement: GridElement | null = exposed.element;
const resolved: GridResolvedLayout = {
  align: "center",
  autoFlow: "row dense",
  columnGap: "2rem",
  columns: "repeat(3, minmax(0, 1fr))",
  gap: "1rem",
  justify: "end",
  rowGap: "0.5rem",
  style: {
    "--vize-ui-grid-align": "center",
    "--vize-ui-grid-auto-flow": "row dense",
    "--vize-ui-grid-column-gap": "2rem",
    "--vize-ui-grid-columns": "repeat(3, minmax(0, 1fr))",
    "--vize-ui-grid-gap": "1rem",
    "--vize-ui-grid-justify": "end",
    "--vize-ui-grid-row-gap": "0.5rem",
    alignItems: "var(--vize-ui-grid-align)",
    columnGap: "var(--vize-ui-grid-column-gap)",
    display: "grid",
    gap: "var(--vize-ui-grid-gap)",
    gridAutoFlow: "var(--vize-ui-grid-auto-flow)",
    gridTemplateColumns: "var(--vize-ui-grid-columns)",
    justifyItems: "var(--vize-ui-grid-justify)",
    rowGap: "var(--vize-ui-grid-row-gap)",
  },
};
const customHost: InstanceType<typeof Grid>["$props"] = {
  align: "baseline",
  as: componentTarget,
  autoFlow: "column dense",
  columnGap: 24,
  columns: 4,
  gap: "1rem",
  justify: "center",
  rowGap: 8,
};

// @ts-expect-error alignment uses logical CSS values, not physical left/right.
const badAlign: InstanceType<typeof Grid>["$props"] = { align: "left" };

// @ts-expect-error justification is constrained to native justify-items values.
const badJustify: InstanceType<typeof Grid>["$props"] = { justify: "space-between" };

// @ts-expect-error auto-flow uses native CSS grid-auto-flow keywords.
const badAutoFlow: InstanceType<typeof Grid>["$props"] = { autoFlow: "rows" };

// @ts-expect-error gap values must be CSS strings or numeric pixel lengths.
const badGap: InstanceType<typeof Grid>["$props"] = { gap: true };

// @ts-expect-error columns must be a CSS track list string or numeric column count.
const badColumns: InstanceType<typeof Grid>["$props"] = { columns: false };

void Grid;
void badAlign;
void badAutoFlow;
void badColumns;
void badGap;
void badJustify;
void customHost;
void exposed;
void exposedElement;
void resolved;
