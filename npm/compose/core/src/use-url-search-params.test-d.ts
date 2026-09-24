/** Compile-only assertions for the `use-url-search-params` type contracts. */

import { parseSearchParams, searchParam, useUrlSearchParams } from "./use-url-search-params.ts";
import type { InferSearchParams } from "./use-url-search-params.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const schema = {
  q: searchParam.string(),
  page: searchParam.number(1),
  open: searchParam.boolean(),
  sort: searchParam.enum(["new", "top"], "new"),
  tags: searchParam.array(),
  ids: searchParam.array(searchParam.number()),
  range: searchParam.json({ from: 0, to: 10 }),
};

type _SchemaInfersEveryKey = Expect<
  Equal<
    InferSearchParams<typeof schema>,
    {
      q: string;
      page: number;
      open: boolean;
      sort: "new" | "top";
      tags: string[];
      ids: number[];
      range: { from: number; to: number };
    }
  >
>;

const { params } = useUrlSearchParams(schema);
type _ParamsAreTyped = Expect<Equal<typeof params.sort, "new" | "top">>;
const parsed = parseSearchParams(schema, "?page=2");
type _ParseIsTyped = Expect<Equal<typeof parsed.ids, number[]>>;

// @ts-expect-error enum values are a closed union.
params.sort = "old";
// @ts-expect-error numbers stay numbers.
params.page = "2";
// @ts-expect-error the enum default must be one of the values.
searchParam.enum(["a", "b"], "c");
// @ts-expect-error history modes are a closed union.
useUrlSearchParams(schema, { mode: "assign" });
