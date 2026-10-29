/** Public P0#11020 input and independent slot precision witnesses. */
export type CasingCase = {
  id: string;
  component: string;
  source: string;
  stage: "valid" | "invalid" | "poisoned" | "repaired";
  expectedCode: number | null;
};
const COMPONENT =
  "import { defineComponent, type SlotsType, type VNode, type PropType } from 'vue';\nconst VueChild = defineComponent({\n  props: { value: { type: Number, required: true }, render: Function as PropType<(value: number) => string> },\n  slots: Object as SlotsType<{ default(props: { item: number }): VNode[] }>,\n});\ntype Instance = InstanceType<typeof VueChild>;\ntype Typed<T> = Omit<Instance, '$props' | '$slots'> & {\n  readonly $props: Readonly<{ value: T }>;\n  readonly $slots: { default(props: { item: T }): VNode[] };\n};\nexport const WitnessChild = VueChild;\nexport default WitnessChild;\n";
const SOURCE =
  "<script setup lang=\"ts\">\nimport witnessChild from '../component';\ntype IsAny<T> = 0 extends 1 & T ? true : false;\ntype IsNever<T> = [T] extends [never] ? true : false;\ntype IsUnknown<T> = IsAny<T> extends true ? false : unknown extends T ? keyof T extends never ? true : false : false;\ntype Exact<A,B> = (<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2) ? true : false;\ntype SlotReport<T> = IsAny<T> extends true ? 'any' : IsNever<T> extends true ? 'never' : IsUnknown<T> extends true ? 'unknown' : Exact<T, number> extends true ? 'number' : 'other';\nfunction slotReport<T>(_value: T): SlotReport<T> { throw new Error('witness'); }\nfunction expectNumber(_value: 'number'): void {}\n</script>\n<template>\n  <witness-child :value=\"'bad'\">\n    <template #default=\"{ item }\">{{ expectNumber(slotReport(item)) }}</template>\n  </witness-child>\n</template>\n";
export function componentCasingCases(): CasingCase[] {
  const rows: CasingCase[] = [];
  const spellings = [
    ["pascal-pascal", "WitnessChild", "WitnessChild"],
    ["pascal-camel", "WitnessChild", "witnessChild"],
    ["pascal-kebab", "WitnessChild", "witness-child"],
    ["camel-camel", "witnessChild", "witnessChild"],
    ["camel-kebab", "witnessChild", "witness-child"],
    ["camel-kebab-crlf", "witnessChild", "witness-child"],
    ["type-only-collision", "witnessChild", "witness-child"],
    ["value-priority", "witnessChild", "witness-child"],
  ] as const;
  for (const [id, binding, tag] of spellings) {
    let source = SOURCE.replace("import witnessChild from", "import " + binding + " from")
      .replaceAll("<witness-child", "<" + tag)
      .replaceAll("</witness-child", "</" + tag);
    if (id === "camel-kebab-crlf")
      source = source
        .replace("<template>\n  ", "<template>\n  <!-- 日本 🧭 -->")
        .replaceAll("\n", "\r\n");
    let component = COMPONENT;
    if (id === "type-only-collision") {
      source = source.replace(
        '<script setup lang="ts">',
        "<script setup lang=\"ts\">\nimport type { AliasOnly as WitnessChild } from '../component';",
      );
      component += "export type AliasOnly = never;\n";
    }
    if (id === "value-priority") {
      source = source.replace(
        "import witnessChild from '../component';",
        "import { WitnessChild, OtherChild as witnessChild } from '../component';",
      );
      component += "export const OtherChild = defineComponent({ props: { value: String } });\n";
    }
    for (const stage of ["valid", "invalid", "poisoned", "repaired"] as const) {
      rows.push({
        id: id + "-" + stage,
        component:
          stage === "poisoned"
            ? component.replace("default(props: { item: number })", "default(props: { item: any })")
            : component,
        source: stage === "invalid" ? source : source.replace(":value=\"'bad'\"", ':value="1"'),
        stage,
        expectedCode: stage === "invalid" ? 2322 : stage === "poisoned" ? 2345 : null,
      });
    }
  }
  return rows;
}
