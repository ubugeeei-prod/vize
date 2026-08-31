/** Compile-only assertions for the public Table family contract. */

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./table.ts";
import type {
  TableBodyEmits,
  TableBodyExpose,
  TableBodySlots,
  TableBodySlotState,
  TableCaptionEmits,
  TableCaptionExpose,
  TableCaptionProps,
  TableCaptionSide,
  TableCaptionSlots,
  TableCaptionSlotState,
  TableCaptionStyle,
  TableCellAlign,
  TableCellEmits,
  TableCellExpose,
  TableCellProps,
  TableCellSlots,
  TableCellSlotState,
  TableCellStyle,
  TableCssCustomProperty,
  TableDataAttribute,
  TableDataName,
  TableDensity,
  TableEmits,
  TableExpose,
  TableHeadEmits,
  TableHeadExpose,
  TableHeaderEmits,
  TableHeaderExpose,
  TableHeaderProps,
  TableHeaderScope,
  TableHeaderSlots,
  TableHeaderSlotState,
  TableHeadSlots,
  TableHeadSlotState,
  TableLayout,
  TablePart,
  TableProps,
  TableRowEmits,
  TableRowExpose,
  TableRowProps,
  TableRowSlots,
  TableRowSlotState,
  TableRowState,
  TableSlots,
  TableSlotState,
  TableStyle,
} from "./table.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const table: TableExpose;
declare const caption: TableCaptionExpose;
declare const head: TableHeadExpose;
declare const body: TableBodyExpose;
declare const row: TableRowExpose;
declare const header: TableHeaderExpose;
declare const cell: TableCellExpose;

type _LayoutIsLiteral = Expect<Equal<TableLayout, "auto" | "fixed">>;
type _DensityIsLiteral = Expect<Equal<TableDensity, "compact" | "normal" | "spacious">>;
type _CaptionSideIsLiteral = Expect<Equal<TableCaptionSide, "top" | "bottom">>;
type _RowStateIsLiteral = Expect<Equal<TableRowState, "default" | "selected">>;
type _CellAlignIsLiteral = Expect<Equal<TableCellAlign, "start" | "center" | "end">>;
type _HeaderScopeIsNativeLiteral = Expect<
  Equal<TableHeaderScope, "col" | "colgroup" | "row" | "rowgroup">
>;
type _PartNamesAreClosed = Expect<
  Equal<TablePart, "body" | "caption" | "cell" | "head" | "header" | "root" | "row">
>;
type _DataNamesAreClosed = Expect<
  Equal<
    TableDataName,
    | "table"
    | "table-body"
    | "table-caption"
    | "table-cell"
    | "table-head"
    | "table-header"
    | "table-row"
  >
>;
type _DataAttributesAreClosed = Expect<
  Equal<
    TableDataAttribute,
    | "data-align"
    | "data-caption-side"
    | "data-colspan"
    | "data-density"
    | "data-layout"
    | "data-rowspan"
    | "data-scope"
    | "data-section"
    | "data-selected"
    | "data-state"
    | "data-vize-ui"
  >
>;
type _CssCustomPropertiesAreClosed = Expect<
  Equal<
    TableCssCustomProperty,
    "--vize-ui-table-caption-side" | "--vize-ui-table-cell-align" | "--vize-ui-table-layout"
  >
>;
type _TablePropsKeysAreClosed = Expect<Equal<keyof TableProps, "density" | "layout">>;
type _CaptionPropsKeysAreClosed = Expect<Equal<keyof TableCaptionProps, "side">>;
type _RowPropsKeysAreClosed = Expect<Equal<keyof TableRowProps, "state">>;
type _HeaderPropsKeysAreClosed = Expect<
  Equal<keyof TableHeaderProps, "abbr" | "align" | "colspan" | "rowspan" | "scope">
>;
type _CellPropsKeysAreClosed = Expect<
  Equal<keyof TableCellProps, "align" | "colspan" | "headers" | "rowspan">
>;
type _TableEmitsAreClosed = Expect<Equal<keyof TableEmits, never>>;
type _CaptionEmitsAreClosed = Expect<Equal<keyof TableCaptionEmits, never>>;
type _HeadEmitsAreClosed = Expect<Equal<keyof TableHeadEmits, never>>;
type _BodyEmitsAreClosed = Expect<Equal<keyof TableBodyEmits, never>>;
type _RowEmitsAreClosed = Expect<Equal<keyof TableRowEmits, never>>;
type _HeaderEmitsAreClosed = Expect<Equal<keyof TableHeaderEmits, never>>;
type _CellEmitsAreClosed = Expect<Equal<keyof TableCellEmits, never>>;
type _TableSlotReceivesState = Expect<Equal<Parameters<TableSlots["default"]>[0], TableSlotState>>;
type _CaptionSlotReceivesState = Expect<
  Equal<Parameters<TableCaptionSlots["default"]>[0], TableCaptionSlotState>
>;
type _HeadSlotReceivesState = Expect<
  Equal<Parameters<TableHeadSlots["default"]>[0], TableHeadSlotState>
>;
type _BodySlotReceivesState = Expect<
  Equal<Parameters<TableBodySlots["default"]>[0], TableBodySlotState>
>;
type _RowSlotReceivesState = Expect<
  Equal<Parameters<TableRowSlots["default"]>[0], TableRowSlotState>
>;
type _HeaderSlotReceivesState = Expect<
  Equal<Parameters<TableHeaderSlots["default"]>[0], TableHeaderSlotState>
>;
type _CellSlotReceivesState = Expect<
  Equal<Parameters<TableCellSlots["default"]>[0], TableCellSlotState>
>;
type _TableStyleIsStrict = Expect<
  Equal<
    Pick<TableStyle, "--vize-ui-table-layout" | "tableLayout">,
    {
      readonly "--vize-ui-table-layout": TableLayout;
      readonly tableLayout: TableLayout;
    }
  >
>;
type _CaptionStyleIsStrict = Expect<
  Equal<
    Pick<TableCaptionStyle, "--vize-ui-table-caption-side" | "captionSide">,
    {
      readonly "--vize-ui-table-caption-side": TableCaptionSide;
      readonly captionSide: TableCaptionSide;
    }
  >
>;
type _CellStyleIsStrict = Expect<
  Equal<
    Pick<TableCellStyle, "--vize-ui-table-cell-align" | "textAlign">,
    {
      readonly "--vize-ui-table-cell-align": TableCellAlign;
      readonly textAlign: TableCellAlign;
    }
  >
>;

const tableProps = {
  density: "compact",
  layout: "fixed",
} satisfies TableProps;
const captionProps = { side: "bottom" } satisfies TableCaptionProps;
const rowProps = { state: "selected" } satisfies TableRowProps;
const headerProps = {
  abbr: "Revenue",
  align: "end",
  colspan: 2,
  rowspan: 1,
  scope: "row",
} satisfies TableHeaderProps;
const cellProps = {
  align: "center",
  colspan: 3,
  headers: "product revenue",
  rowspan: 2,
} satisfies TableCellProps;
const tableComponentProps: InstanceType<typeof Table>["$props"] = tableProps;
const captionComponentProps: InstanceType<typeof TableCaption>["$props"] = captionProps;
const rowComponentProps: InstanceType<typeof TableRow>["$props"] = rowProps;
const headerComponentProps: InstanceType<typeof TableHeader>["$props"] = headerProps;
const cellComponentProps: InstanceType<typeof TableCell>["$props"] = cellProps;

const tableElement: HTMLTableElement | null = table.element;
const captionElement: HTMLTableCaptionElement | null = caption.element;
const headElement: HTMLTableSectionElement | null = head.element;
const bodyElement: HTMLTableSectionElement | null = body.element;
const rowElement: HTMLTableRowElement | null = row.element;
const headerElement: HTMLTableCellElement | null = header.element;
const cellElement: HTMLTableCellElement | null = cell.element;
const tableLayout: TableLayout = table.layout;
const tableDensity: TableDensity = table.density;
const captionSide: TableCaptionSide = caption.side;
const headSection: "head" = head.section;
const bodySection: "body" = body.section;
const rowState: TableRowState = row.state;
const rowSelected: boolean = row.selected;
const headerScope: TableHeaderScope = header.scope;
const cellAlign: TableCellAlign = cell.align;

// @ts-expect-error Table layout only supports native CSS table-layout modes.
const invalidLayout: TableProps = { layout: "grid" };

// @ts-expect-error Table density is a strict consumer hook token.
const invalidDensity: TableProps = { density: "loose" };

// @ts-expect-error Caption side only supports native top and bottom placement.
const invalidCaptionSide: TableCaptionProps = { side: "inline-start" };

// @ts-expect-error Row state is not a selection model.
const invalidRowState: TableRowProps = { state: "expanded" };

// @ts-expect-error Header scope must stay within the native table scope tokens.
const invalidHeaderScope: TableHeaderProps = { scope: "column" };

// @ts-expect-error Cell alignment uses logical text alignment tokens.
const invalidCellAlign: TableCellProps = { align: "right" };

void Table;
void TableBody;
void TableCaption;
void TableCell;
void TableHead;
void TableHeader;
void TableRow;
void bodyElement;
void bodySection;
void captionComponentProps;
void captionElement;
void captionProps;
void captionSide;
void cellAlign;
void cellComponentProps;
void cellElement;
void cellProps;
void headElement;
void headSection;
void headerComponentProps;
void headerElement;
void headerProps;
void headerScope;
void invalidCaptionSide;
void invalidCellAlign;
void invalidDensity;
void invalidHeaderScope;
void invalidLayout;
void invalidRowState;
void rowComponentProps;
void rowElement;
void rowProps;
void rowSelected;
void rowState;
void tableComponentProps;
void tableDensity;
void tableElement;
void tableLayout;
void tableProps;
