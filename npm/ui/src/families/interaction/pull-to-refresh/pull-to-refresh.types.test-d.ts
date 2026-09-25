/** Compile-only assertions for the public PullToRefresh contract. */

import {
  PullToRefresh,
  PullToRefreshTrigger,
  type PullToRefreshExpose,
  type PullToRefreshSource,
  type PullToRefreshState,
} from "./pull-to-refresh.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const api: PullToRefreshExpose;

type _StateIsClosed = Expect<
  Equal<PullToRefreshState, "armed" | "idle" | "pulling" | "refreshing">
>;
type _SourceIsClosed = Expect<Equal<PullToRefreshSource, "api" | "gesture" | "trigger">>;
type _RefreshIsAsync = Expect<Equal<ReturnType<typeof api.refresh>, Promise<void>>>;

const props: InstanceType<typeof PullToRefresh>["$props"] = {
  threshold: 72,
  refreshAction: async (source: PullToRefreshSource) => {
    void source;
  },
  onSettle: (source: PullToRefreshSource, error: unknown) => {
    void source;
    void error;
  },
};
const triggerProps: InstanceType<typeof PullToRefreshTrigger>["$props"] = {};

// @ts-expect-error thresholds are numbers.
const wrongThreshold: InstanceType<typeof PullToRefresh>["$props"] = { threshold: "64" };

void props;
void triggerProps;
void wrongThreshold;
