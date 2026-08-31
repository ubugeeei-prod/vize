/** Compile-only assertions for the public Pagination composition contract. */

import type { Component, ComponentPublicInstance } from "vue";

import {
  Pagination,
  PaginationEllipsis,
  PaginationItem,
  PaginationList,
  PaginationNext,
  PaginationPage,
  PaginationPrevious,
  PaginationRoot,
  type PaginationControlExpose,
  type PaginationControlSlotState,
  type PaginationControlState,
  type PaginationEllipsisExpose,
  type PaginationEllipsisPosition,
  type PaginationEllipsisSlotState,
  type PaginationItemExpose,
  type PaginationItemSlotState,
  type PaginationListExpose,
  type PaginationListSlotState,
  type PaginationPageExpose,
  type PaginationPageSlotState,
  type PaginationPageState,
  type PaginationRangeItem,
  type PaginationRootExpose,
  type PaginationSlotState,
  type PaginationState,
} from "./pagination.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const root: PaginationRootExpose;
declare const list: PaginationListExpose;
declare const item: PaginationItemExpose;
declare const page: PaginationPageExpose;
declare const previous: PaginationControlExpose;
declare const next: PaginationControlExpose;
declare const ellipsis: PaginationEllipsisExpose;
declare const slot: PaginationSlotState;
declare const listSlot: PaginationListSlotState;
declare const itemSlot: PaginationItemSlotState;
declare const pageSlot: PaginationPageSlotState;
declare const controlSlot: PaginationControlSlotState;
declare const ellipsisSlot: PaginationEllipsisSlotState;
declare const rangeItem: PaginationRangeItem;

type _RootStateIsLiteral = Expect<Equal<PaginationState, "active" | "disabled">>;
type _PageStateIsLiteral = Expect<Equal<PaginationPageState, "current" | "disabled" | "idle">>;
type _ControlStateIsLiteral = Expect<Equal<PaginationControlState, "disabled" | "idle">>;
type _EllipsisPositionIsLiteral = Expect<Equal<PaginationEllipsisPosition, "start" | "end">>;
type _RootElementIsRenderable = Expect<
  Equal<typeof root.element, Element | ComponentPublicInstance | null>
>;
type _ListElementIsRenderable = Expect<
  Equal<typeof list.element, Element | ComponentPublicInstance | null>
>;
type _PageElementIsButton = Expect<Equal<typeof page.element, HTMLButtonElement | null>>;
type _ControlElementIsButton = Expect<Equal<typeof previous.element, HTMLButtonElement | null>>;
type _RootPageIsNumber = Expect<Equal<typeof root.page, number>>;
type _RootRangeIsReadonly = Expect<Equal<typeof root.range, readonly PaginationRangeItem[]>>;
type _ItemPageIsOptional = Expect<Equal<typeof item.page, number | undefined>>;
type _PageCurrentIsBoolean = Expect<Equal<typeof page.current, boolean>>;
type _PreviousTargetIsNullable = Expect<Equal<typeof previous.targetPage, number | null>>;
type _NextTargetIsNullable = Expect<Equal<typeof next.targetPage, number | null>>;
type _EllipsisDisabledIsLiteral = Expect<Equal<typeof ellipsis.disabled, true>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly page: number;
      readonly pageCount: number;
      readonly disabled: boolean;
      readonly canPrevious: boolean;
      readonly canNext: boolean;
      readonly previousPage: number | null;
      readonly nextPage: number | null;
      readonly range: readonly PaginationRangeItem[];
      readonly state: PaginationState;
    }
  >
>;
type _ListSlotStateExtendsRoot = Expect<Equal<typeof listSlot, PaginationListSlotState>>;
type _ItemSlotStateIsExact = Expect<
  Equal<
    typeof itemSlot,
    {
      readonly page: number | undefined;
      readonly current: boolean;
      readonly disabled: boolean;
      readonly state: PaginationPageState;
    }
  >
>;
type _PageSlotStateIsExact = Expect<
  Equal<
    typeof pageSlot,
    {
      readonly page: number;
      readonly current: boolean;
      readonly disabled: boolean;
      readonly state: PaginationPageState;
    }
  >
>;
type _ControlSlotStateIsExact = Expect<
  Equal<
    typeof controlSlot,
    {
      readonly targetPage: number | null;
      readonly disabled: boolean;
      readonly state: PaginationControlState;
    }
  >
>;
type _EllipsisSlotStateIsExact = Expect<
  Equal<
    typeof ellipsisSlot,
    {
      readonly position: PaginationEllipsisPosition;
      readonly disabled: true;
    }
  >
>;
type _RangeItemIsClosed = Expect<
  Equal<
    typeof rangeItem,
    | { readonly type: "page"; readonly key: `page-${number}`; readonly page: number }
    | {
        readonly type: "ellipsis";
        readonly key: "ellipsis-start" | "ellipsis-end";
        readonly position: "start" | "end";
      }
  >
>;

const rootProps: InstanceType<typeof PaginationRoot>["$props"] = {
  as: componentTarget,
  boundaryCount: 1,
  defaultValue: 2,
  disabled: false,
  id: "docs-pages",
  label: "Documentation pages",
  modelValue: 3,
  onChange: (value: number, previousValue: number, event: Event | null) => {
    void value;
    void previousValue;
    void event;
  },
  "onUpdate:modelValue": (value: number) => value,
  pageCount: 12,
  siblingCount: 2,
};
const aliasProps: InstanceType<typeof Pagination>["$props"] = rootProps;
const listProps: InstanceType<typeof PaginationList>["$props"] = { as: "ol" };
const itemProps: InstanceType<typeof PaginationItem>["$props"] = {
  as: "li",
  disabled: false,
  page: 2,
};
const pageProps: InstanceType<typeof PaginationPage>["$props"] = {
  ariaDescribedby: "page-help",
  ariaLabel: "Page two",
  ariaLabelledby: "page-label",
  disabled: false,
  page: 2,
  type: "button",
};
const previousProps: InstanceType<typeof PaginationPrevious>["$props"] = {
  ariaDescribedby: "previous-help",
  ariaLabel: "Previous",
  ariaLabelledby: "previous-label",
  disabled: false,
  type: "button",
};
const nextProps: InstanceType<typeof PaginationNext>["$props"] = previousProps;
const ellipsisProps: InstanceType<typeof PaginationEllipsis>["$props"] = {
  as: "span",
  label: "More pages",
  position: "start",
};

root.focus();
root.setPage(4);
root.goPrevious();
root.goNext();
root.reset();
list.focus();
page.focus();
page.select();
previous.focus();
previous.select();
next.focus();
next.select();

// @ts-expect-error root state is a closed styling contract.
const invalidRootState: PaginationState = "ready";

// @ts-expect-error page state is a closed styling contract.
const invalidPageState: PaginationPageState = "selected";

// @ts-expect-error ellipsis position is limited to compact-range gaps.
const invalidEllipsisPosition: PaginationEllipsisPosition = "middle";

// @ts-expect-error page count is required.
const missingPageCount: InstanceType<typeof PaginationRoot>["$props"] = {};

// @ts-expect-error page control requires a numeric page.
const badPageProps: InstanceType<typeof PaginationPage>["$props"] = { page: "2" };

const badPageType: InstanceType<typeof PaginationPage>["$props"] = {
  page: 2,
  // @ts-expect-error native button type is limited to platform submit modes.
  type: "menu",
};

void PaginationEllipsis;
void aliasProps;
void badPageProps;
void badPageType;
void ellipsisProps;
void invalidEllipsisPosition;
void invalidPageState;
void invalidRootState;
void itemProps;
void listProps;
void missingPageCount;
void nextProps;
void pageProps;
void previousProps;
void rootProps;
