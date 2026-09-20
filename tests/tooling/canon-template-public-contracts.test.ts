import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  check,
  workspace,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc, vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

const assertions = `type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;`;

test("loop tuple shapes and forbidden callbacks match vue-tsc diagnostic locations", async () => {
  const directory = project("loop-and-callback-diagnostics-", {
    "Child.vue": `<script setup lang="ts" generic="T">defineProps<{ value: T; pick: never }>();</script>`,
    "App.vue": `<script setup lang="ts">
import Child from './Child.vue';
const anything: any = null;
const records: Record<string, number> = {};
function takesNumber(value: number) { return value; }
</script><template>
<Child :value="1" :pick="() => 1" />
<div v-for="(item, key, index) in anything">{{ item }}{{ key < 1 }}{{ takesNumber(index) }}</div>
<div v-for="(item, key, index) in records">{{ item.toFixed() }}{{ key.toUpperCase() }}{{ takesNumber(index) }}</div>
</template>`,
  });
  try {
    const expected = vueTscDiagnostics(directory).sort(compareIdentity);
    assert.equal(expected.length, 3);
    assert.deepEqual(
      (await check(directory)).map(diagnosticIdentity).sort(compareIdentity),
      expected,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

function project(prefix: string, files: Record<string, string>): string {
  const directory = workspace(prefix);
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        jsx: "preserve",
        noEmit: true,
        skipLibCheck: true,
      },
      vueCompilerOptions: { jsxSlots: true, strictTemplates: true },
      include: ["*.vue", "*.ts", "*.tsx"],
    }),
  );
  for (const [name, source] of Object.entries(files))
    fs.writeFileSync(path.join(directory, name), source);
  return directory;
}

test("listeners satisfy required props without erasing authored handler errors", async () => {
  const directory = project("required-listener-props-", {
    "Generic.vue": `<script setup lang="ts" generic="T">defineProps<{ value: T; onFoo: (value: T) => void }>();</script>`,
    "App.vue": `<script setup lang="ts">
import Generic from './Generic.vue';
declare const External: new () => { $props: { onFoo: () => void } };
</script><template>
<External @foo="() => {}" />
<Generic :value="1" @foo="value => value.toFixed()" />
<Generic :value="1" />
<Generic :value="1" @foo="(value: string) => {}" />
<Generic :value="1" :on-foo="123" @foo="() => {}" />
</template>`,
  });
  try {
    const expected = vueTscDiagnostics(directory).sort(compareIdentity);
    assert.equal(expected.length, 3);
    assert.deepEqual(
      (await check(directory)).map(diagnosticIdentity).sort(compareIdentity),
      expected,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("inferred slot contracts retain loop, scoped slot and generic payload types", async () => {
  const directory = project("inferred-slot-contract-", {
    "env.d.ts": `declare namespace JSX { interface ElementChildrenAttribute { children: {}; } }`,
    "Host.vue": `<script setup lang="ts">defineSlots<{ default(props: { item: { name: string } }): unknown }>();</script>`,
    "Child.vue": `<script setup lang="ts">
import Host from './Host.vue';
defineProps<{ rows: { id: number }[] }>();
</script><template>
<slot name="activator" :isActive="false" />
<div v-for="row in rows"><slot name="row" :item="row" /></div>
<Host v-slot="{ item }"><slot name="nested" :value="item.name" /></Host>
<slot name="empty" />
</template>`,
    "Generic.vue": `<script setup lang="ts" generic="T">defineProps<{ item: T }>();</script><template><slot :value="item" /></template>`,
    "consumer.tsx": `/// <reference types="vue/jsx" />
import Child from './Child.vue';
import Generic from './Generic.vue';
import type { ComponentSlots } from 'vue-component-type-helpers';
${assertions}
type Slots = ComponentSlots<typeof Child>;
type Row = Assert<Equal<Parameters<NonNullable<Slots['row']>>[0], { item: { id: number } }>>;
type Nested = Assert<Equal<Parameters<NonNullable<Slots['nested']>>[0], { value: string }>>;
type Empty = Assert<Equal<Parameters<NonNullable<Slots['empty']>>[0], {}>>;
type GenericSlots = ComponentSlots<typeof Generic<number>>;
type GenericValue = Assert<Equal<Parameters<NonNullable<GenericSlots['default']>>[0], { value: number }>>;
const child = <Child rows={[]}>{{ activator: props => props.isActive.valueOf(), row: props => props.item.id.toFixed(), nested: props => props.value.toUpperCase() }}</Child>;
// @ts-expect-error
const wrong = <Child rows={[]}>{{ activator: (props: { isActive: string }) => props.isActive }}</Child>;
`,
  });
  try {
    assertVueTsc(directory);
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("root element types flow through ordinary and generic exposes and template refs", async () => {
  const directory = project("root-element-contract-", {
    "Base.vue": `<!-- @inferComponentDollarEl true --><script setup lang="ts"></script><template><a /></template>`,
    "Child.vue": `<!-- @inferComponentDollarEl true --><script setup lang="ts">import Base from './Base.vue';</script><template><Base v-if="true" /><Transition v-else><img /></Transition></template>`,
    "Generic.vue": `<!-- @inferComponentDollarEl true --><script setup lang="ts" generic>defineExpose({ run() {} });</script><template><svg><a /></svg></template>`,
    "Fragment.vue": `<!-- @inferComponentDollarEl true --><script setup lang="ts" generic></script><template><a /><img /></template>`,
    "App.vue": `<script setup lang="ts">
import { useTemplateRef } from 'vue';
import Child from './Child.vue';
import Generic from './Generic.vue';
import Fragment from './Fragment.vue';
import type { ComponentExposed } from 'vue-component-type-helpers';
${assertions}
type ChildEl = Assert<Equal<InstanceType<typeof Child>['$el'], HTMLAnchorElement | HTMLImageElement>>;
type GenericEl = Assert<Equal<ComponentExposed<typeof Generic>['$el'], SVGSVGElement>>;
type FragmentExposed = Assert<Equal<ComponentExposed<typeof Fragment>, {}>>;
const child = useTemplateRef('child');
const generic = useTemplateRef('generic');
type RefEl = Assert<Equal<NonNullable<typeof generic.value>['$el'], SVGSVGElement>>;
generic.value?.run();
// @ts-expect-error
generic.value?.missing;
// @ts-expect-error
child.value?.$el.notAnElementProperty;
</script><template><Child ref="child" /><Generic ref="generic" /></template>`,
  });
  try {
    assertVueTsc(directory);
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("typed slot outlets retain overloaded signatures and check empty required payloads", async () => {
  const directory = project("overloaded-slot-outlet-", {
    "Slots.vue": `<script setup lang="ts" generic="T extends object, F">
import type { VNode } from 'vue';
type DynamicFieldSlots<T, F> = Record<\`\${keyof T extends string ? keyof T : never}-field\` | (string & {}), (props: { field: F; state: T }) => VNode[]>;
type DynamicFormFieldSlots<T> = Record<\`\${keyof T extends string ? keyof T : never}-label\` | (string & {}), (props?: {}) => VNode[]>;
type Slots = { header?(props?: {}): VNode[] } & DynamicFieldSlots<T, F> & DynamicFormFieldSlots<T>;
const slots = defineSlots<Slots>();
</script><template><slot name="header" /></template>`,
    "Required.vue": `<script setup lang="ts" generic>
defineSlots<{ required(props: { item: number }): unknown; empty(): unknown; 'last-columns'(props: {}): unknown }>();
</script><template>
<slot name="empty" />
<slot name="required" :item="1" />
<!-- @vue-expect-error -->
<slot name="required" />
<!-- @vue-expect-error -->
<slot name="required" :item="'wrong'" />
</template>`,
    "App.vue": `<script setup lang="ts">import Required from './Required.vue';</script><template>
<Required><template #last-columns />
<!-- @vue-expect-error -->
<template #last-columns-absent />
</Required>
</template>`,
  });
  try {
    assertVueTsc(directory);
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
