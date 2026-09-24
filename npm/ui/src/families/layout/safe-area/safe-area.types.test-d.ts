/** Compile-only assertions for the public SafeArea contract. */

import {
  SafeArea,
  useSafeAreaInsets,
  type SafeAreaEdge,
  type SafeAreaEdgeInsets,
} from "./safe-area.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const controller = useSafeAreaInsets();

type _EdgeIsClosed = Expect<Equal<SafeAreaEdge, "bottom" | "left" | "right" | "top">>;
type _Insets = Expect<Equal<typeof controller.insets.value, SafeAreaEdgeInsets>>;

const props: InstanceType<typeof SafeArea>["$props"] = { edges: ["top"], apply: "margin" };

// @ts-expect-error edges are closed.
const wrongEdge: InstanceType<typeof SafeArea>["$props"] = { edges: ["start"] };

// @ts-expect-error apply modes are closed.
const wrongApply: InstanceType<typeof SafeArea>["$props"] = { apply: "border" };

void props;
void wrongApply;
void wrongEdge;
