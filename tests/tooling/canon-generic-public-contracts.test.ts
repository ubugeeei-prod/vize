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
