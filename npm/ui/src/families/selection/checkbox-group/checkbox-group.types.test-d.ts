/** Compile-only assertions for the public CheckboxGroup contract. */

import {
  CheckboxGroup,
  CheckboxGroupItem,
  CheckboxGroupSelectAll,
  type CheckboxGroupExpose,
  type CheckboxGroupItemState,
  type CheckboxGroupKey,
  type CheckboxGroupSlotState,
  type CheckboxGroupState,
} from "./checkbox-group.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface User {
  readonly id: number;
  readonly name: string;
}

type GroupProps<Value> = Parameters<typeof CheckboxGroup<Value>>[0];
type ItemProps<Value> = Parameters<typeof CheckboxGroupItem<Value>>[0];

/** Infers the value type exactly as a template usage would. */
declare function inferGroupValue<Value>(props: GroupProps<Value>): Value;

declare const users: readonly User[];
declare const control: CheckboxGroupExpose<User>;
declare const slot: CheckboxGroupSlotState<"a" | "b">;

const inferredUser = inferGroupValue({ options: users });
const inferredLiteral = inferGroupValue({ options: ["a", "b"] as const });

type _InfersObjectValues = Expect<Equal<typeof inferredUser, User>>;
type _InfersLiteralValues = Expect<Equal<typeof inferredLiteral, "a" | "b">>;
type _StateIsClosed = Expect<Equal<CheckboxGroupState, "all" | "disabled" | "none" | "some">>;
type _ItemStateIsClosed = Expect<
  Equal<CheckboxGroupItemState, "checked" | "indeterminate" | "unchecked">
>;
type _ExposeValues = Expect<Equal<typeof control.values, readonly User[]>>;
type _SlotValues = Expect<Equal<typeof slot.values, readonly ("a" | "b")[]>>;
type _KeyAcceptsIds = Expect<Equal<Extract<CheckboxGroupKey, number>, number>>;

const typedProps: GroupProps<User> = {
  options: users,
  modelValue: [],
  by: (user) => user.id,
  getFormValue: (user, index) => `${user.id}-${index}`,
  isOptionDisabled: (user) => user.name.length === 0,
  "onUpdate:modelValue": (value: readonly User[]) => value,
  onChange: (value: readonly User[], toggled: User | null, selected: boolean) => {
    void value;
    void toggled;
    void selected;
  },
};
const itemProps: ItemProps<User> = { value: { id: 1, name: "Ada" }, disabled: true };
const selectAllProps: InstanceType<typeof CheckboxGroupSelectAll>["$props"] = { ariaLabel: "All" };

control.setSelected({ id: 1, name: "Ada" }, true);
control.setAll(false);

// @ts-expect-error selected values must match the option type.
const mismatched: GroupProps<User> = { options: users, modelValue: ["Ada"] };

// @ts-expect-error `by` receives the option type.
const wrongBy: GroupProps<User> = { options: users, by: (user: string) => user };

// @ts-expect-error options are required.
const missingOptions: GroupProps<User> = {};

// @ts-expect-error the imperative API is typed by the option type.
control.setSelected("Ada", true);

void itemProps;
void missingOptions;
void mismatched;
void selectAllProps;
void typedProps;
void wrongBy;
