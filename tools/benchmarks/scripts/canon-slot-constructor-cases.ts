/** Independent constructor and public-prop admission; every source is an original public fixture. */
export type SlotCase = {
  id: string;
  component: string;
  source: string;
  requiresAuthoredError: boolean;
};
export const slotSource = `<script setup lang="ts">
import WitnessChild from '../component';
type IsAny<T> = 0 extends 1 & T ? true : false;
type IsNever<T> = [T] extends [never] ? true : false;
type IsUnknown<T> = IsAny<T> extends true ? false : unknown extends T ? keyof T extends never ? true : false : false;
type Exact<A,B> = (<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2) ? true : false;
type SlotReport<T> = IsAny<T> extends true ? 'any' : IsNever<T> extends true ? 'never' : IsUnknown<T> extends true ? 'unknown' : Exact<T, number> extends true ? 'number' : 'other';
function slotReport<T>(_value: T): SlotReport<T> { throw new Error('witness'); }
function expectNumber(_value: 'number'): void {}
</script>
<template>
  <WitnessChild :value="1">
    <template #default="{ item }">{{ expectNumber(slotReport(item)) }}</template>
  </WitnessChild>
</template>
`;
const base = `import { defineComponent, type SlotsType, type VNode, type PropType } from 'vue';
const VueChild = defineComponent({
  props: { value: { type: Number, required: true }, render: Function as PropType<(value: number) => string> },
  slots: Object as SlotsType<{ default(props: { item: number }): VNode[] }>,
});
type Instance = InstanceType<typeof VueChild>;
type Typed<T> = Omit<Instance, '$props' | '$slots'> & {
  readonly $props: Readonly<{ value: T }>;
  readonly $slots: { default(props: { item: T }): VNode[] };
};
`;
const ctor = (signature: string): string =>
  base +
  "export declare const WitnessChild: { " +
  signature +
  " };\nexport default WitnessChild;\n";
export function slotCases(): SlotCase[] {
  const zero = ctor("new(): Instance");
  const result: SlotCase[] = [
    { id: "zero", component: zero, source: slotSource, requiresAuthoredError: false },
    {
      id: "optional",
      component: ctor("new(props?: { value: number }): Instance"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "rest",
      component: ctor("new(...props: unknown[]): Instance"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "generic-default-zero",
      component: ctor("new<T = number>(): Typed<T>"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "generic-props",
      component: ctor("new<T>(props: { value: T }): Typed<T>"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "overload-last-zero",
      component: ctor("new(props: { value: number }): Typed<number>; new(): Typed<string>"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "overload-first-zero",
      component: ctor("new(): Typed<string>; new(props: { value: number }): Typed<number>"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "missing-required",
      component: zero,
      source: slotSource.replace(' :value="1"', ""),
      requiresAuthoredError: true,
    },
    {
      id: "wrong-prop",
      component: zero,
      source: slotSource.replace(':value="1"', ":value=\"'bad'\""),
      requiresAuthoredError: true,
    },
    {
      id: "unknown-prop",
      component: zero,
      source: slotSource.replace(':value="1"', ':value="1" :extra="1"'),
      requiresAuthoredError: true,
    },
    {
      id: "callback-body",
      component: zero,
      source: slotSource.replace(
        ':value="1"',
        ':value="1" :render="(value: number) => value.missing"',
      ),
      requiresAuthoredError: true,
    },
    {
      id: "missing-symbol",
      component: zero,
      source: slotSource.replace(':value="1"', ':value="missing"'),
      requiresAuthoredError: true,
    },
    {
      id: "readonly-prop",
      component: zero,
      source: slotSource.replace(
        "</script>",
        "const instance = undefined as unknown as InstanceType<typeof WitnessChild>;\ninstance.$props.value = 2;\n</script>",
      ),
      requiresAuthoredError: true,
    },
    {
      id: "constructor-public-props-discrepancy",
      component: ctor("new(props: { value: number; internal: string }): Instance"),
      source: slotSource,
      requiresAuthoredError: false,
    },
    {
      id: "constructor-public-type-discrepancy",
      component: ctor("new(props: { value: string }): Instance"),
      source: slotSource,
      requiresAuthoredError: false,
    },
  ];
  for (const type of ["any", "unknown", "never", "string"]) {
    result.push({
      id: "slot-poison-" + type,
      component: zero.replace("item: number", "item: " + type),
      source: slotSource,
      requiresAuthoredError: true,
    });
  }
  for (const newline of ["\r\n"]) {
    result.push({
      id: "unicode-crlf",
      component: zero,
      source: slotSource
        .replace("<template>", "<template>\n<!-- 日本 🧭 -->")
        .replaceAll("\n", newline),
      requiresAuthoredError: false,
    });
  }
  return result;
}
