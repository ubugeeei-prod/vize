/**
 * Shared layer of the generated `lsp-vs-volar` projects: entity types,
 * composables, stores and a component kit (generic list, table, select and
 * tabs with scoped slots, typed emits and models). Sources are pure functions
 * of their import specifiers, so the single-project and monorepo layouts
 * share every byte except those.
 */

export const KIT = [
  "UiBadge",
  "UiButton",
  "UiCard",
  "UiDialog",
  "UiField",
  "UiList",
  "UiSelect",
  "UiTable",
  "UiTabs",
  "UiTooltip",
] as const;

export type KitName = (typeof KIT)[number];

export const TYPES_SOURCE = `export interface Entity {
  id: string;
  label: string;
}

export type Currency = "EUR" | "USD" | "JPY";

export interface Money {
  amount: number;
  currency: Currency;
}
`;

export function selectionSource(types: string): string {
  return `import { computed, shallowRef } from "vue";

import type { Entity } from "${types}";

export function useSelection<T extends Entity>() {
  const selected = shallowRef<ReadonlySet<T["id"]>>(new Set());
  function toggle(item: T): void {
    const next = new Set(selected.value);
    if (next.has(item.id)) next.delete(item.id);
    else next.add(item.id);
    selected.value = next;
  }
  const isSelected = (item: T): boolean => selected.value.has(item.id);
  const count = computed(() => selected.value.size);
  return { selected, toggle, isSelected, count };
}
`;
}

export const PAGER_SOURCE = `import { computed, ref, toValue, type MaybeRefOrGetter } from "vue";

export function usePager<T>(source: MaybeRefOrGetter<readonly T[]>, size: number) {
  const page = ref(0);
  const pages = computed(() => Math.max(1, Math.ceil(toValue(source).length / size)));
  const visible = computed(() => toValue(source).slice(page.value * size, (page.value + 1) * size));
  function go(index: number): void {
    page.value = Math.min(Math.max(0, index), pages.value - 1);
  }
  return { page, pages, visible, size, go };
}
`;

export const COMPOSABLES_INDEX = `export * from "./usePager";
export * from "./useSelection";
`;

export function orderStoreSource(types: string): string {
  return `import { reactive, readonly } from "vue";

import type { Currency } from "${types}";

export interface OrderFilters {
  query: string;
  currency: Currency | null;
  page: number;
}

const state = reactive<OrderFilters>({ query: "", currency: null, page: 0 });

export function useOrderStore() {
  return {
    state: readonly(state),
    setQuery(query: string): void {
      state.query = query;
    },
    setCurrency(currency: Currency | null): void {
      state.currency = currency;
    },
    turn(page: number): void {
      state.page = page;
    },
  };
}
`;
}

export const SESSION_STORE_SOURCE = `import { computed, reactive, readonly } from "vue";

export interface SessionState {
  user: string | null;
  locale: "en" | "ja" | "fr";
  roles: string[];
}

const state = reactive<SessionState>({ user: null, locale: "en", roles: [] });

export function useSessionStore() {
  const signedIn = computed(() => state.user !== null);
  return {
    state: readonly(state),
    signedIn,
    signIn(user: string, roles: string[]): void {
      state.user = user;
      state.roles = roles;
    },
    setLocale(locale: SessionState["locale"]): void {
      state.locale = locale;
    },
  };
}
`;

export const STORES_INDEX = `export * from "./orders";
export * from "./session";
`;

const KIT_SOURCES: Record<KitName, (types: string) => string> = {
  UiBadge: () => `<script setup lang="ts">
defineProps<{ tone: "info" | "warn" | "error" }>();
defineSlots<{ default?(): any }>();
</script>

<template>
  <span :class="tone"><slot /></span>
</template>
`,
  UiButton: () => `<script setup lang="ts">
withDefaults(defineProps<{ variant?: "primary" | "ghost" | "danger"; disabled?: boolean }>(), {
  variant: "primary",
  disabled: false,
});
const emit = defineEmits<{ press: [event: MouseEvent]; focus: []; blur: [] }>();
</script>

<template>
  <button :class="variant" :disabled="disabled" @click="emit('press', $event)" @focus="emit('focus')" @blur="emit('blur')">
    <slot />
  </button>
</template>
`,
  UiCard: () => `<script setup lang="ts">
defineProps<{ title: string; dense?: boolean }>();
defineSlots<{ header?(): any; default?(): any; footer?(props: { title: string }): any }>();
</script>

<template>
  <section :class="{ dense }">
    <header><slot name="header">{{ title }}</slot></header>
    <slot />
    <footer><slot name="footer" :title="title" /></footer>
  </section>
</template>
`,
  UiDialog: () => `<script setup lang="ts">
const open = defineModel<boolean>("open", { default: false });
defineProps<{ heading: string }>();
const emit = defineEmits<{ confirm: []; cancel: [reason: string] }>();
defineSlots<{ default?(): any; actions?(props: { close: () => void }): any }>();
function close(): void {
  open.value = false;
  emit("cancel", "closed");
}
</script>

<template>
  <dialog :open="open">
    <h2>{{ heading }}</h2>
    <slot />
    <slot name="actions" :close="close">
      <button @click="emit('confirm')">ok</button>
    </slot>
  </dialog>
</template>
`,
  UiField: () => `<script setup lang="ts">
const props = defineProps<{ label: string; error?: string }>();
defineSlots<{ default?(props: { id: string; invalid: boolean }): any }>();
const id = \`field-\${props.label}\`;
</script>

<template>
  <label :for="id">{{ label }}</label>
  <slot :id="id" :invalid="error !== undefined" />
  <small v-if="error">{{ error }}</small>
</template>
`,
  UiList: (types) => `<script setup lang="ts" generic="T extends Entity">
import type { Entity } from "${types}";

defineProps<{ items: readonly T[]; selected?: T["id"] }>();
const emit = defineEmits<{
  select: [item: T];
  reorder: [from: number, to: number];
  "update:selected": [id: T["id"]];
}>();
defineSlots<{ item?(props: { item: T; index: number; active: boolean }): any; empty?(): any }>();
function pick(item: T): void {
  emit("select", item);
  emit("update:selected", item.id);
}
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="item.id" @click="pick(item)" @dragend="emit('reorder', index, 0)">
      <slot name="item" :item="item" :index="index" :active="item.id === selected">{{ item.label }}</slot>
    </li>
    <li v-if="!items.length"><slot name="empty" /></li>
  </ul>
</template>
`,
  UiSelect: () => `<script setup lang="ts" generic="V extends string | number">
const model = defineModel<V>({ required: true });
defineProps<{ options: readonly { value: V; label: string }[] }>();
const emit = defineEmits<{ open: []; close: [value: V] }>();
defineSlots<{ option?(props: { option: { value: V; label: string }; selected: boolean }): any }>();
</script>

<template>
  <ul @mouseenter="emit('open')" @mouseleave="emit('close', model)">
    <li v-for="option in options" :key="option.value" @click="model = option.value">
      <slot name="option" :option="option" :selected="option.value === model">{{ option.label }}</slot>
    </li>
  </ul>
</template>
`,
  UiTable: (
    types,
  ) => `<script setup lang="ts" generic="Row extends Entity, Key extends keyof Row & string">
import type { Entity } from "${types}";

defineProps<{ rows: readonly Row[]; columns: readonly Key[] }>();
const emit = defineEmits<{ sort: [key: Key, direction: "asc" | "desc"]; open: [row: Row] }>();
defineSlots<{ cell?(props: { row: Row; column: Key; value: Row[Key] }): any }>();
</script>

<template>
  <table>
    <thead>
      <tr>
        <th v-for="column in columns" :key="column" @click="emit('sort', column, 'asc')">{{ column }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="row in rows" :key="row.id" @dblclick="emit('open', row)">
        <td v-for="column in columns" :key="column">
          <slot name="cell" :row="row" :column="column" :value="row[column]" />
        </td>
      </tr>
    </tbody>
  </table>
</template>
`,
  UiTabs: () => `<script setup lang="ts" generic="K extends string">
const active = defineModel<K>("active", { required: true });
defineProps<{ tabs: readonly K[] }>();
defineSlots<{ tab?(props: { name: K; active: boolean }): any }>();
</script>

<template>
  <nav>
    <button v-for="name in tabs" :key="name" @click="active = name">
      <slot name="tab" :name="name" :active="name === active">{{ name }}</slot>
    </button>
  </nav>
</template>
`,
  UiTooltip: () => `<script setup lang="ts">
withDefaults(defineProps<{ text: string; placement?: "top" | "bottom" }>(), { placement: "top" });
defineSlots<{ default?(): any }>();
</script>

<template>
  <span :title="text" :data-placement="placement"><slot /></span>
</template>
`,
};

export function kitSource(name: KitName, types: string): string {
  return KIT_SOURCES[name](types);
}

/** Barrel for explicit imports: `import { UiCard } from "<kit>"`. */
export const KIT_INDEX = KIT.map((name) => `export { default as ${name} } from "./${name}.vue";`)
  .join("\n")
  .concat("\n");

/** Global registration, as unplugin-vue-components writes it; `path` maps a kit name to its import. */
export function globalComponentsSource(path: (name: KitName) => string): string {
  const entries = KIT.map((name) => `    ${name}: typeof import("${path(name)}")["default"];`);
  return `export {};

declare module "vue" {
  export interface GlobalComponents {
${entries.join("\n")}
  }
}
`;
}
