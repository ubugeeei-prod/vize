import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc } from "./support/vue-tsc-oracle.ts";

const assertions = `type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;`;

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
      vueCompilerOptions: { jsxSlots: true },
      include: ["*.vue", "*.ts", "*.tsx"],
    }),
  );
  for (const [name, source] of Object.entries(files))
    fs.writeFileSync(path.join(directory, name), source);
  return directory;
}

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
