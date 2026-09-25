/** Compile-only assertions for the typed route query and params helpers. */

import { shallowRef } from "vue";
import type { ComputedRef, WritableComputedRef } from "vue";

import { oneOf, queryParsers, useRouteParams, useRouteQuery } from "./use-route-query.ts";
import type { RouteQueryRecord } from "./use-route-query.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

/** Mirrors `RouteMatch` from `@vizejs/router` for `/users/:id/:tab?`. */
interface UserMatch {
  readonly name: "user";
  readonly pathname: string;
  readonly params: { readonly id: string; readonly tab?: string };
  readonly query: RouteQueryRecord;
  readonly hash: string;
}

declare const match: UserMatch;
const route = shallowRef(match);
declare const navigate: (query: RouteQueryRecord) => void;

const raw = useRouteQuery(route, "q");
type _DefaultIsOptionalString = Expect<Equal<typeof raw, ComputedRef<string | undefined>>>;
const page = useRouteQuery(route, "page", { parse: queryParsers.integer, default: 1 });
type _DefaultRemovesUndefined = Expect<Equal<typeof page, ComputedRef<number>>>;
const writablePage = useRouteQuery(route, "page", {
  parse: queryParsers.integer,
  default: 1,
  navigate,
});
type _NavigateMakesItWritable = Expect<Equal<typeof writablePage, WritableComputedRef<number>>>;
const sort = useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "new" });
type _OneOfNarrows = Expect<Equal<typeof sort, ComputedRef<"new" | "top">>>;
const tags = useRouteQuery(route, "tags", { parse: queryParsers.array, navigate });
type _OptionalWritable = Expect<
  Equal<typeof tags, WritableComputedRef<string[] | undefined, string[] | undefined>>
>;
// @ts-expect-error the default must match the parser.
useRouteQuery(route, "page", { parse: queryParsers.integer, default: "1" });
// @ts-expect-error oneOf defaults must be allowed values.
useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "old" });
// @ts-expect-error read-only without navigate.
page.value = 2;

const params = useRouteParams(route);
type _AllParams = Expect<
  Equal<typeof params, ComputedRef<{ readonly id: string; readonly tab?: string }>>
>;
const id = useRouteParams(route, "id");
type _OneParam = Expect<Equal<typeof id, ComputedRef<string>>>;
const tab = useRouteParams(route, "tab");
type _OptionalParam = Expect<Equal<typeof tab, ComputedRef<string | undefined>>>;
const numericId = useRouteParams(route, "id", Number);
type _ParsedParam = Expect<Equal<typeof numericId, ComputedRef<number>>>;
// @ts-expect-error params are keyed by the route's path.
useRouteParams(route, "slug");
