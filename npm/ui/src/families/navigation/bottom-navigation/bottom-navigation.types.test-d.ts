/** Compile-only assertions for the public BottomNavigation contract. */

import { BottomNavigation, BottomNavigationItem, TabBar } from "./bottom-navigation.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type NavProps<Value extends string> = Parameters<typeof BottomNavigation<Value>>[0];

/** Infers destination values exactly as a template usage would. */
declare function inferDestination<Value extends string>(props: NavProps<Value>): Value;

const inferred = inferDestination({ destinations: ["home", "search"] });

type _InfersUnion = Expect<Equal<typeof inferred, "home" | "search">>;
type _TabBarAlias = Expect<Equal<typeof TabBar, typeof BottomNavigation>>;

const props: NavProps<"home" | "search"> = {
  destinations: ["home", "search"],
  modelValue: "home",
  onSelect: (value: "home" | "search", event: MouseEvent) => {
    void value;
    void event;
  },
};
const itemProps: InstanceType<typeof BottomNavigationItem>["$props"] = { value: "home", badge: 2 };

const wrongModel: NavProps<"home" | "search"> = {
  destinations: ["home", "search"],
  // @ts-expect-error the model must be a destination.
  modelValue: "x",
};

// @ts-expect-error items need a value.
const missingValue: InstanceType<typeof BottomNavigationItem>["$props"] = {};

void itemProps;
void missingValue;
void props;
void wrongModel;
