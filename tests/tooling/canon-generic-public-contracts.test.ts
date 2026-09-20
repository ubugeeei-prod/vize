import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc } from "./support/vue-tsc-oracle.ts";

test(
  "generic SFCs expose Vue's callable props, emits, slots and unwrapped expose contract",
  { timeout: 60_000 },
  async () => {
    const directory = workspace("generic-public-contract-");
    try {
      fs.writeFileSync(
        path.join(directory, "vize.config.json"),
        JSON.stringify({ typeChecker: { jsxTypecheck: true } }),
      );
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            module: "ESNext",
            target: "ESNext",
            moduleResolution: "Bundler",
            jsx: "preserve",
            jsxImportSource: "vue",
            noEmit: true,
            skipLibCheck: true,
          },
          include: ["*.vue", "*.tsx"],
        }),
      );
      fs.writeFileSync(
        path.join(directory, "Generic.vue"),
        `<script setup lang="ts" generic="T">
import { ref, type Ref } from 'vue';
defineProps<{ foo: T }>();
defineEmits<{ (e: 'bar', value: T): void }>();
defineSlots<{ default?: (props: T) => unknown }>();
defineExpose({ baz: {} as T, buz: ref(1) as Ref<1> });
</script>`,
      );
      fs.writeFileSync(
        path.join(directory, "consumer.tsx"),
        `import Generic from './Generic.vue';
import type { ComponentExposed, ComponentProps, ComponentSlots, ComponentEmit } from 'vue-component-type-helpers';
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
type Exposed = Assert<Equal<ComponentExposed<typeof Generic>, { baz: unknown; buz: 1 }>>;
type NumberExposed = Assert<Equal<ComponentExposed<typeof Generic<number>>, { baz: number; buz: 1 }>>;
type Props = Assert<Equal<ComponentProps<typeof Generic<number>>['foo'], number>>;
type Emit = Assert<Equal<ComponentEmit<typeof Generic<number>>, (e: 'bar', value: number) => void>>;
type Slots = Assert<Equal<ComponentSlots<typeof Generic<number>>, { default?: (props: number) => unknown }>>;
Generic({ foo: 1, onBar(value) { value.toFixed(); } });
const rendered = <Generic foo={1} onBar={value => value.toFixed()} />;
// @ts-expect-error
new Generic();
// @ts-expect-error
Generic.props;
// @ts-expect-error
Generic({ foo: 1, onBar(value: string) {} });
// @ts-expect-error
const invalid = <Generic foo={1} onBar={(value: string) => {}} />;
void rendered;
`,
      );
      assertVueTsc(directory);
      assert.deepEqual(await check(directory), []);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

test(
  "generic props retain setup aliases and static attributes infer emitted payloads",
  { timeout: 60_000 },
  async () => {
    const directory = workspace("generic-alias-contract-");
    try {
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            strict: true,
            module: "ESNext",
            target: "ESNext",
            moduleResolution: "Bundler",
            noEmit: true,
            skipLibCheck: true,
          },
          include: ["*.vue", "*.ts"],
        }),
      );
      fs.writeFileSync(
        path.join(directory, "Child.vue"),
        `<script setup lang="ts" generic="T extends string | number">
type InputProps = { type?: 'input'; value: (value: T) => void; typeDefinition: T };
type SelectProps = { type: 'select'; value: (value: T[]) => void; typeDefinition: T };
defineProps<InputProps | SelectProps>();
</script>`,
      );
      fs.writeFileSync(
        path.join(directory, "Emitter.vue"),
        `<script setup lang="ts" generic="T">
defineProps<{ modelValue: T }>();
defineEmits<{ (event: 'update:model-value', value: T): void }>();
</script>`,
      );
      fs.writeFileSync(
        path.join(directory, "Rows.vue"),
        `<script setup lang="ts" generic="T">
type Props = { rows: T[] };
defineProps<Props>();
</script><template><div v-for="row in rows"><slot name="row" :item="row" /></div></template>`,
      );
      fs.writeFileSync(
        path.join(directory, "Dynamic.vue"),
        `<script setup lang="ts" generic="T extends { key: string }, U">
type Props = { columns: T[]; rows: U[] };
defineProps<Props>();
</script><template>
<div v-for="column in columns"><slot :name="\`col(\${column.key})\`" v-bind="column" /></div>
<div v-for="(row, index) in rows"><slot :name="\`row(\${index})\`" v-bind="row" /></div>
</template>`,
      );
      fs.writeFileSync(
        path.join(directory, "App.vue"),
        `<script setup lang="ts">
import Child from './Child.vue';
import Emitter from './Emitter.vue';
import Rows from './Rows.vue';
import Dynamic from './Dynamic.vue';
import { exactType } from './exact';
const str: string = '';
const num: number = 1;
const rows: { id: number }[] = [];
const columns: { key: string; title: string }[] = [];
</script><template>
<Child type="select" :type-definition="str" :value="value => exactType(value, {} as string[])" />
<Child type="input" :type-definition="num" :value="value => exactType(value, {} as number)" />
<Emitter model-value="text" @update:model-value="value => exactType(value, {} as string)" />
<Dynamic :columns="columns" :rows="rows">
<template #col(count)="column">{{ exactType(column, {} as { key: string; title: string }) }}</template>
<template #row(0)="row">{{ exactType(row, {} as { id: number }) }}</template>
</Dynamic>
<Rows :rows="rows" v-slot:row="{ item }">{{ exactType(item, {} as { id: number }) }}
<!-- @vue-expect-error -->
{{ item.missing }}
</Rows>
<!-- @vue-expect-error -->
<Child type="select" :type-definition="str" :value="(value: number[]) => {}" />
</template>`,
      );
      fs.writeFileSync(
        path.join(directory, "exact.ts"),
        `type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
export declare function exactType<A, B>(actual: A, expected: B & (Equal<A, B> extends true ? unknown : never)): void;`,
      );
      assertVueTsc(directory);
      assert.deepEqual(await check(directory), []);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
