/**
 * The edited SFC of the generated `lsp-vs-volar` projects and its real
 * neighbourhood: a 24-step model chain and a 12-level generic component chain
 * below it, 8 hosts and a 12-level chain above it. Every size emits these
 * bytes unchanged, so the edited SFC, its anchors and its dependents never
 * vary with project size.
 */

import type { RequestSpec } from "./lsp-vs-volar-fixtures.ts";
import { anchorIn } from "./lsp-vs-volar-text.ts";

export const MODEL_STEPS = 24;
export const CHAIN_LEVELS = 12;
export const HOSTS = 8;
export const UPPER_LEVELS = 12;

const two = (index: number) => String(index).padStart(2, "0");

export function stepSource(index: number, types: string): string {
  const id = two(index);
  if (index === 0) {
    return `import type { Entity, Money } from "${types}";

export interface Step00 extends Entity {
  total: Money;
  stage00: number;
}

export function weigh00(value: number): number {
  return value + 1;
}
`;
  }
  const previous = two(index - 1);
  return `import { type Step${previous}, weigh${previous} } from "./step-${previous}";

export interface Step${id} extends Step${previous} {
  stage${id}: number;
}

export function weigh${id}(value: number): number {
  return weigh${previous}(value) + ${index};
}
`;
}

export function ordersSource(types: string): string {
  const last = two(MODEL_STEPS - 1);
  return `import type { Currency } from "${types}";

import { type Step${last}, weigh${last} } from "./step-${last}";

export type OrderStatus = "open" | "paid" | "void";

export interface OrderRow extends Step${last} {
  status: OrderStatus;
  customer: string;
}

export interface OrderSummary {
  total: number;
  currency: Currency;
  open: number;
}

export function summarizeOrders<Row extends OrderRow>(rows: readonly Row[]): OrderSummary {
  let total = 0;
  let open = 0;
  for (const row of rows) {
    total += weigh${last}(row.total.amount);
    if (row.status === "open") open += 1;
  }
  return { total, currency: rows[0]?.total.currency ?? "EUR", open };
}
`;
}

export function modelIndexSource(): string {
  const steps = Array.from(
    { length: MODEL_STEPS },
    (_, index) => `export * from "./step-${two(index)}";`,
  );
  return `export * from "./orders";\n${steps.join("\n")}\n`;
}

/** `kit` imports the kit explicitly (a library); `null` uses the global registration (an app). */
export function levelSource(index: number, model: string, kit: string | null): string {
  const id = two(index);
  const imports = [`import { type OrderRow, weigh${id} } from "${model}";`];
  if (kit)
    imports.push(
      `import { ${index === 0 ? "UiButton, UiCard, UiList" : "UiCard"} } from "${kit}";`,
    );
  const inner =
    index === 0
      ? `    <UiList :items="items" @select="(item) => emit('pick', item)">
      <template #item="{ item, index }">
        <slot name="item" :item="item" :index="index" />
      </template>
      <template #empty>
        <UiButton variant="ghost" @press="emit('clear')">clear</UiButton>
      </template>
    </UiList>`
      : `    <Level${two(index - 1)} :items="items" :depth="depth + 1" @pick="(item) => emit('pick', item)" @depth="(value) => emit('depth', value)" @clear="emit('clear')">
      <template #item="{ item, index }">
        <slot name="item" :item="item" :index="index" />
      </template>
    </Level${two(index - 1)}>`;
  if (index > 0) imports.push(`import Level${two(index - 1)} from "./Level${two(index - 1)}.vue";`);
  return `<script setup lang="ts" generic="T extends OrderRow">
${imports.join("\n")}

const props = defineProps<{ items: readonly T[]; depth: number }>();
const emit = defineEmits<{ pick: [item: T]; depth: [value: number]; clear: [] }>();
defineSlots<{ item?(props: { item: T; index: number }): any }>();
const weight = weigh${id}(props.depth);
</script>

<template>
  <UiCard title="level ${id}" dense>
${inner}
    <small>{{ weight }}</small>
  </UiCard>
</template>
`;
}

export interface FocusImports {
  /** The directory holding `Level11.vue`. */
  chain: string;
  model: string;
  composables: string;
  stores: string;
}

/** In the body: a generic setup's imports must lead it for Vue's language tools to hoist them. */
export const FOCUS_SCRIPT_PROBE_AFTER = "const expanded = ref(false);\n";
export const FOCUS_TEMPLATE_PROBE_AFTER =
  "    <p>{{ boardSummary.total }} {{ boardSummary.currency }}</p>\n";

/** The edited SFC: generic, with typed emits, scoped slots and a generic child chain. */
export function focusSource(imports: FocusImports): string {
  const level = `Level${two(CHAIN_LEVELS - 1)}`;
  return `<script setup lang="ts" generic="Row extends OrderRow">
import { computed, ref } from "vue";

import ${level} from "${imports.chain}/${level}.vue";
import { usePager, useSelection } from "${imports.composables}";
import { type OrderRow, summarizeOrders } from "${imports.model}";
import { useOrderStore } from "${imports.stores}";

const props = defineProps<{ rows: readonly Row[]; title: string; pageSize?: number }>();
const emit = defineEmits<{
  select: [row: Row];
  page: [index: number, size: number];
  rename: [id: Row["id"], label: string];
  clear: [];
}>();
defineSlots<{
  header?(props: { title: string; total: number }): any;
  row?(props: { row: Row; selected: boolean }): any;
}>();

const store = useOrderStore();
const selection = useSelection<Row>();
const boardSummary = computed(() => summarizeOrders(props.rows));
const { visible, size, go } = usePager(() => props.rows, props.pageSize ?? 20);
${FOCUS_SCRIPT_PROBE_AFTER}
function choose(row: Row): void {
  selection.toggle(row);
  emit("select", row);
}

function turn(index: number): void {
  go(index);
  store.turn(index);
  emit("page", index, size);
}
</script>

<template>
  <UiCard :title="title" :dense="expanded">
    <template #header>
      <slot name="header" :title="title" :total="boardSummary.total" />
      <UiButton variant="ghost" @press="expanded = !expanded">{{ store.state.query }}</UiButton>
    </template>
    <${level} :items="visible" :depth="${CHAIN_LEVELS - 1}" @pick="choose" @depth="turn" @clear="emit('clear')">
      <template #item="{ item }">
        <slot name="row" :row="item" :selected="selection.isSelected(item)" />
      </template>
    </${level}>
    <UiTable :rows="props.rows" :columns="['label', 'customer']" @open="choose">
      <template #cell="{ row, column, value }">{{ column }}={{ value }} ({{ row.status }})</template>
    </UiTable>
${FOCUS_TEMPLATE_PROBE_AFTER}  </UiCard>
</template>
`;
}

export function hostSource(index: number, model: string): string {
  return `<script setup lang="ts">
import { ref } from "vue";

import type { OrderRow } from "${model}";
import OrderBoard from "../OrderBoard.vue";

const rows = ref<OrderRow[]>([]);
const picked = ref<string | null>(null);

function onSelect(row: OrderRow): void {
  picked.value = row.customer;
}
</script>

<template>
  <OrderBoard
    :rows="rows"
    title="host ${two(index)}"
    :page-size="${index + 5}"
    @select="onSelect"
    @page="(index, size) => (picked = \`\${index}/\${size}\`)"
    @rename="(id, label) => (picked = \`\${id}:\${label}\`)"
    @clear="picked = null"
  >
    <template #header="{ title, total }">
      <h3>{{ title }} {{ total.toFixed(2) }}</h3>
    </template>
    <template #row="{ row, selected }">
      <UiBadge :tone="selected ? 'info' : 'warn'">{{ row.customer }} {{ picked }}</UiBadge>
    </template>
  </OrderBoard>
</template>
`;
}

export function upperSource(index: number, model: string): string {
  const inner = index === 0 ? "OrderBoard" : `Upper${two(index - 1)}`;
  const from = index === 0 ? "../OrderBoard.vue" : `./Upper${two(index - 1)}.vue`;
  const slots =
    index === 0
      ? `
      <template #row="{ row, selected }">
        <UiTooltip :text="row.customer">{{ selected ? row.label : row.status }}</UiTooltip>
      </template>
    `
      : "";
  return `<script setup lang="ts" generic="R extends OrderRow">
import type { OrderRow } from "${model}";
import ${inner} from "${from}";

defineProps<{ rows: readonly R[]; title: string }>();
const emit = defineEmits<{ select: [row: R]; clear: [] }>();
</script>

<template>
  <section>
    <${inner} :rows="rows" :title="title" @select="(row) => emit('select', row)" @clear="emit('clear')">${slots}</${inner}>
  </section>
</template>
`;
}

export function appSource(model: string): string {
  const hosts = Array.from({ length: HOSTS }, (_, index) => `BoardHost${two(index)}`);
  const upper = `Upper${two(UPPER_LEVELS - 1)}`;
  return `<script setup lang="ts">
import type { OrderRow } from "${model}";
${hosts.map((name) => `import ${name} from "./core/hosts/${name}.vue";`).join("\n")}
import ${upper} from "./core/upper/${upper}.vue";

const rows: OrderRow[] = [];
</script>

<template>
  <main>
    <${upper} :rows="rows" title="orders" @select="(row) => row.customer" @clear="() => undefined" />
${hosts.map((name) => `    <${name} />`).join("\n")}
  </main>
</template>
`;
}

export const MAIN_SOURCE = `import { createApp } from "vue";

import App from "./App.vue";
import { routes } from "./routes";

const app = createApp(App);
app.provide("routes", routes);
app.mount("#app");
`;

/** Lazy routes to the rest of the app: imports nothing of the edited SFC's neighbourhood. */
export function routesSource(views: string[]): string {
  const entries = views.map(
    (path) =>
      `  { path: "/${path.split("/").at(-1)!.replace(".vue", "")}", component: () => import("./${path}") },`,
  );
  return `export const routes = [\n${entries.join("\n")}\n];\n`;
}

/** Core SFCs: the chain below, the edited SFC, its hosts, the chain above and the app root. */
export const CORE_SFCS = CHAIN_LEVELS + 1 + HOSTS + UPPER_LEVELS + 1;

/** What a correct answer holds at each anchor of `focusSource`; paths end the expected targets. */
export function focusRequests(paths: { chain: string; model: string }): RequestSpec[] {
  const level = `Level${two(CHAIN_LEVELS - 1)}`;
  const tag = `<${level} :items="visible" :depth="${CHAIN_LEVELS - 1}"`;
  return [
    {
      kind: "hover",
      name: "script binding",
      anchor: anchorIn("const boardSummary = computed(", "boardSummary"),
      expect: ["boardSummary:"],
    },
    {
      kind: "hover",
      name: "template expression",
      anchor: anchorIn("<p>{{ boardSummary.total }}", "boardSummary"),
      expect: ["boardSummary:"],
    },
    { kind: "hover", name: "component tag", anchor: anchorIn(tag, level), expect: [level] },
    { kind: "hover", name: "component prop", anchor: anchorIn(tag, "depth"), expect: ["depth"] },
    {
      kind: "completion",
      name: "script member",
      anchor: anchorIn("  selection.toggle(row);", "toggle"),
      expect: ["toggle", "isSelected", "count"],
    },
    {
      kind: "completion",
      name: "template member",
      anchor: anchorIn("{{ boardSummary.currency }}", "currency"),
      expect: ["currency", "total", "open"],
    },
    {
      kind: "definition",
      name: "component tag",
      anchor: anchorIn(tag, level),
      expectPath: `${paths.chain}/${level}.vue`,
    },
    {
      kind: "definition",
      name: "imported function",
      anchor: anchorIn("computed(() => summarizeOrders(props.rows))", "summarizeOrders"),
      expectPath: `${paths.model}/orders.ts`,
    },
    {
      kind: "references",
      name: "local binding",
      anchor: anchorIn("const boardSummary = computed(", "boardSummary"),
      min: 4,
    },
    {
      kind: "references",
      name: "imported function",
      anchor: anchorIn("computed(() => summarizeOrders(props.rows))", "summarizeOrders"),
      min: 3,
    },
    {
      kind: "rename",
      name: "local binding",
      anchor: anchorIn("const boardSummary = computed(", "boardSummary"),
      newName: "boardTotals",
      min: 4,
    },
    { kind: "semanticTokens", name: "document", min: 1 },
  ];
}
