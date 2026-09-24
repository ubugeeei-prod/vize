/** Compile-only assertions for the public InfiniteScroll contract. */

import {
  InfiniteScroll,
  InfiniteScrollItem,
  InfiniteScrollLoadMore,
  InfiniteScrollRoot,
  InfiniteScrollSentinel,
  InfiniteScrollStatus,
  type InfiniteScrollItemExpose,
  type InfiniteScrollItemSlotState,
  type InfiniteScrollLoader,
  type InfiniteScrollLoadMoreExpose,
  type InfiniteScrollObserverRoot,
  type InfiniteScrollRootExpose,
  type InfiniteScrollSentinelExpose,
  type InfiniteScrollSlotState,
  type InfiniteScrollState,
  type InfiniteScrollStatusExpose,
  type InfiniteScrollTrigger,
} from "./infinite-scroll.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: InfiniteScrollRootExpose;
declare const sentinel: InfiniteScrollSentinelExpose;
declare const loadMore: InfiniteScrollLoadMoreExpose;
declare const status: InfiniteScrollStatusExpose;
declare const item: InfiniteScrollItemExpose;

type _StateIsLiteral = Expect<
  Equal<InfiniteScrollState, "complete" | "disabled" | "error" | "idle" | "loading">
>;
type _TriggerIsLiteral = Expect<Equal<InfiniteScrollTrigger, "api" | "button" | "sentinel">>;
type _ObserverRootIsLiteral = Expect<Equal<InfiniteScrollObserverRoot, "self" | "viewport">>;
type _LoaderReceivesTrigger = Expect<
  Equal<Parameters<InfiniteScrollLoader>, [trigger: InfiniteScrollTrigger]>
>;
type _SlotStateIsExact = Expect<
  Equal<
    InfiniteScrollSlotState,
    {
      readonly state: InfiniteScrollState;
      readonly busy: boolean;
      readonly hasMore: boolean;
      readonly error: unknown;
    }
  >
>;
type _ItemSlotIsExact = Expect<
  Equal<InfiniteScrollItemSlotState, { readonly position: number; readonly setSize: number }>
>;
type _RootElementIsDiv = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _RootLoadMoreReports = Expect<Equal<typeof root.loadMore, () => boolean>>;
type _SentinelIntersecting = Expect<Equal<typeof sentinel.intersecting, boolean>>;
type _LoadMoreElementIsButton = Expect<Equal<typeof loadMore.element, HTMLButtonElement | null>>;
type _StatusElementIsDiv = Expect<Equal<typeof status.element, HTMLDivElement | null>>;
type _ItemElementIsElement = Expect<Equal<typeof item.element, HTMLElement | null>>;
type _AliasIsRoot = Expect<Equal<typeof InfiniteScroll, typeof InfiniteScrollRoot>>;

void InfiniteScrollItem;
void InfiniteScrollLoadMore;
void InfiniteScrollSentinel;
void InfiniteScrollStatus;

// @ts-expect-error states are a closed union.
const _unknownState: InfiniteScrollState = "paused";
// @ts-expect-error triggers are a closed union.
const _unknownTrigger: InfiniteScrollTrigger = "scroll";
// @ts-expect-error exposed state is read-only.
root.state = "idle";
