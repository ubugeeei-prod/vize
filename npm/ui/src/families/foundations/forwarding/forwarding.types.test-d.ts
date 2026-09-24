/** Compile-only assertions for prop, emit, and expose forwarding helpers. */

import type { ComputedRef, EmitFn } from "vue";

import {
  toHandlerKey,
  useEmitAsProps,
  useForwardExpose,
  useForwardProps,
  useForwardPropsEmits,
} from "./forwarding.ts";
import type { EmitEventArgs, EmitEventName, EmitHandlerKey, ForwardedProps } from "./forwarding.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

// The same type `defineEmits<{ ... }>()` returns.
declare const emit: EmitFn<{
  "update:modelValue": [value: number];
  close: [];
  "value-change": [value: number, previous: number];
}>;

type _EventNamesAreInferred = Expect<
  Equal<EmitEventName<typeof emit>, "update:modelValue" | "close" | "value-change">
>;
type _PayloadsAreInferred = Expect<
  Equal<EmitEventArgs<typeof emit, "value-change">, [value: number, previous: number]>
>;
type _HandlerKeysFollowVue = Expect<
  Equal<
    EmitHandlerKey<"update:modelValue" | "value-change">,
    "onUpdate:modelValue" | "onValueChange"
  >
>;
type _ToHandlerKeyIsLiteral = Expect<
  Equal<ReturnType<typeof toHandlerKey<"value-change">>, "onValueChange">
>;

const handlers = useEmitAsProps(emit, ["update:modelValue", "value-change"]);
type _HandlersAreTyped = Expect<
  Equal<
    typeof handlers,
    {
      readonly "onUpdate:modelValue": (value: number) => void;
      readonly onValueChange: (value: number, previous: number) => void;
    }
  >
>;
// @ts-expect-error forwarding an undeclared event is rejected.
useEmitAsProps(emit, ["open"]);
// @ts-expect-error handler payloads are typed.
handlers["onUpdate:modelValue"]("1");

declare const props: {
  readonly label: string;
  readonly size?: "sm" | "lg";
  readonly count: number | undefined;
};
const forwarded = useForwardProps(props);
type _ForwardedPropsDropUndefined = Expect<
  Equal<typeof forwarded, ComputedRef<ForwardedProps<typeof props>>>
>;
type _ForwardedValuesExcludeUndefined = Expect<
  Equal<NonNullable<typeof forwarded.value.count>, number>
>;

const both = useForwardPropsEmits(props, emit, ["close"]);
both.value.onClose satisfies () => void;
both.value.label satisfies string | undefined;

const expose = useForwardExpose<{ readonly focus: () => void; readonly count: number }>();
expose.exposed.$el satisfies Element | null;
expose.exposed.focus satisfies (() => void) | undefined;
// @ts-expect-error unknown exposed members do not exist.
void expose.exposed.blur;
// @ts-expect-error the forwarded API is readonly.
expose.exposed.count = 1;
