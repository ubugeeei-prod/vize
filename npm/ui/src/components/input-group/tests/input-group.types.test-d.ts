import { InputGroup, InputGroupInput } from "../index.ts";
import type { InputGroupInputProps, InputGroupProps } from "../index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _InputTypesStayClosed = Expect<
  Equal<
    NonNullable<InputGroupInputProps["type"]>,
    "text" | "email" | "url" | "search" | "tel" | "password"
  >
>;
type _GroupPropsFeedTheSfc = Expect<
  InputGroupProps extends InstanceType<typeof InputGroup>["$props"] ? true : false
>;
type _InputPropsFeedTheSfc = Expect<
  InputGroupInputProps extends InstanceType<typeof InputGroupInput>["$props"] ? true : false
>;

export const group = { label: "Website", disabled: true } satisfies InputGroupProps;
export const field = { type: "url", modelValue: "vize.dev" } satisfies InputGroupInputProps;

// @ts-expect-error input types are a closed public contract.
export const badType: InputGroupInputProps = { type: "file" };
