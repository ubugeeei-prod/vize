//! Authored sources for the Options API declaration matrix (#4010).
//!
//! Every member `consumer.tsx`, [`SOURCE_CONSUMER`] and [`PARENT`] assert was
//! first emitted by `vue-tsc --emitDeclarationOnly` over these exact files, so
//! the expectations are the upstream surface rather than a hand-written guess.

/// The issue's own reproduction.
pub(super) const OPTIONS: &str = r#"<script lang="ts">
export default {
  data() {
    return { count: 1 };
  },
};
</script>

<template>{{ count.toFixed(0) }}</template>
"#;

/// `data`: primitive, nested object, array, tuple, widening and `as const`.
pub(super) const DATA: &str = r#"<script lang="ts">
export default {
  data() {
    return {
      count: 1,
      label: 'ready',
      flag: true,
      nested: { inner: { depth: 2 } },
      list: [1, 2, 3],
      pair: ['a', 1] as [string, number],
      widened: 'text' as string,
      literal: 'exact' as const,
      maybe: null as string | null,
    };
  },
};
</script>

<template>{{ count.toFixed(0) }}{{ label }}{{ nested.inner.depth }}{{ list.length }}</template>
"#;

/// `computed`: getter, getter/setter pair, and a cross-computed dependency.
pub(super) const COMPUTED: &str = r#"<script lang="ts">
export default {
  data() {
    return { count: 2 };
  },
  computed: {
    doubled(): number {
      return this.count * 2;
    },
    quadrupled(): number {
      return this.doubled * 2;
    },
    label: {
      get(): string {
        return String(this.count);
      },
      set(next: string) {
        this.count = Number(next);
      },
    },
  },
};
</script>

<template>{{ doubled }}{{ quadrupled }}{{ label }}</template>
"#;

/// `methods`: parameters, an overload-like union parameter, async and
/// generator methods, and `this` dependencies.
pub(super) const METHODS: &str = r#"<script lang="ts">
export default {
  data() {
    return { count: 3 };
  },
  methods: {
    add(step: number): number {
      return this.count + step;
    },
    describe(value: string | number): string {
      return typeof value === 'string' ? value : value.toFixed(1);
    },
    async load(id: string): Promise<{ id: string }> {
      return { id };
    },
    *walk(): Generator<number, void, unknown> {
      yield this.count;
    },
  },
};
</script>

<template>{{ add(1) }}{{ describe('x') }}</template>
"#;

/// Runtime props, typed emits, `inject`/`provide`.
pub(super) const PROPS_EMITS: &str = r#"<script lang="ts">
import type { PropType } from 'vue';

export default {
  props: {
    title: { type: String, required: true },
    size: { type: Number, default: 1 },
    items: { type: Array as PropType<string[]>, required: true },
  },
  emits: {
    pick: (value: string) => value.length > 0,
    clear: () => true,
  },
  inject: {
    theme: { from: 'theme', default: 'light' },
  },
  provide() {
    return { scope: 'props-emits' };
  },
  computed: {
    heading(): string {
      return `${this.title}:${this.size}`;
    },
  },
  methods: {
    emitPick(): void {
      this.$emit('pick', this.title);
    },
  },
};
</script>

<template>{{ heading }}{{ items.length }}</template>
"#;

/// An imported `defineComponent` whose `setup` return joins `data`/`methods`.
pub(super) const SETUP_RETURN: &str = r#"<script lang="ts">
import { computed, defineComponent, ref } from 'vue';

export default defineComponent({
  setup() {
    const total = ref(0);
    const doubledTotal = computed(() => total.value * 2);
    return { total, doubledTotal };
  },
  data() {
    return { seed: 'setup' };
  },
  methods: {
    bump(step: number): void {
      this.total += step;
    },
  },
});
</script>

<template>{{ total }}{{ doubledTotal }}{{ seed }}</template>
"#;

/// `extends` plus `mixins`, both imported from sibling modules.
pub(super) const INHERITED: &str = r#"<script lang="ts">
import { defineComponent } from 'vue';
import base from './base';
import greeter from './greeter';

export default defineComponent({
  extends: base,
  mixins: [greeter],
  data() {
    return { own: 1 };
  },
});
</script>

<template>{{ own }}</template>
"#;

pub(super) const BASE: &str = r#"import { defineComponent } from 'vue';

export default defineComponent({
  data() {
    return { inheritedCount: 5 };
  },
});
"#;

pub(super) const GREETER: &str = r#"import { defineComponent } from 'vue';

export default defineComponent({
  methods: {
    greet(): string {
      return 'hello';
    },
  },
});
"#;

/// Script consumer inside the checked project (source lane).
pub(super) const SOURCE_CONSUMER: &str = r#"import Computed from './Computed.vue';
import Data from './Data.vue';
import Methods from './Methods.vue';
import Options from './Options.vue';

type IsAny<T> = 0 extends 1 & T ? true : false;

declare const options: InstanceType<typeof Options>;
declare const data: InstanceType<typeof Data>;
declare const computed: InstanceType<typeof Computed>;
declare const methods: InstanceType<typeof Methods>;

export const sourceCount: number = options.count;
export const sourceNested: number = data.nested.inner.depth;
export const sourceDoubled: number = computed.doubled;
export const sourceAdd: number = methods.add(1);
export const sourceCountIsAny: false = null as unknown as IsAny<typeof options.count>;
// @ts-expect-error a script consumer keeps the data member's exact type
export const sourceCountAsString: string = options.count;
// @ts-expect-error a script consumer keeps the method parameter's exact type
export const sourceAddWrong: number = methods.add('1');
"#;

/// Template refs and parent component refs (source lane).
pub(super) const PARENT: &str = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
import Computed from './Computed.vue'
import Methods from './Methods.vue'
import Options from './Options.vue'

const optionsRef = useTemplateRef<InstanceType<typeof Options>>('options')
const computedRef = useTemplateRef<InstanceType<typeof Computed>>('computed')
const methodsRef = useTemplateRef<InstanceType<typeof Methods>>('methods')

const parentCount: number | undefined = optionsRef.value?.count
const parentDoubled: number | undefined = computedRef.value?.doubled
const parentAdd: number | undefined = methodsRef.value?.add(1)

// @ts-expect-error a template ref keeps the data member's exact type
const parentCountAsString: string | undefined = optionsRef.value?.count
// @ts-expect-error a template ref keeps the method parameter's exact type
const parentAddWrong: number | undefined = methodsRef.value?.add('1')

defineExpose({ parentCount, parentDoubled, parentAdd, parentCountAsString, parentAddWrong })
</script>

<template>
  <Options ref="options" />
  <Computed ref="computed" />
  <Methods ref="methods" />
</template>
"#;

/// A component whose template never touches an instance member, so declaration
/// emit succeeds with the Options API binding form both on and off.
pub(super) const SHAPE_ONLY: &str = r#"<script lang="ts">
export default {
  data() {
    return { count: 1 };
  },
  computed: {
    doubled(): number {
      return this.count * 2;
    },
  },
  methods: {
    add(step: number): number {
      return this.count + step;
    },
  },
};
</script>

<template><span /></template>
"#;

/// Declaration-only package consumer, compiled against the emitted `.d.ts`.
pub(super) const DOWNSTREAM: &str = include_str!("consumer.tsx");

pub(super) const PROJECT_FILES: &[(&str, &str)] = &[
    ("src/Options.vue", OPTIONS),
    ("src/Data.vue", DATA),
    ("src/Computed.vue", COMPUTED),
    ("src/Methods.vue", METHODS),
    ("src/PropsEmits.vue", PROPS_EMITS),
    ("src/SetupReturn.vue", SETUP_RETURN),
    ("src/Inherited.vue", INHERITED),
    ("src/base.ts", BASE),
    ("src/greeter.ts", GREETER),
    ("src/Consumer.ts", SOURCE_CONSUMER),
    ("src/Parent.vue", PARENT),
];
