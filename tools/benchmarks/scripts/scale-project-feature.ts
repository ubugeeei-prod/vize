/**
 * Feature pages, widget chains and the app root of the generated scale
 * projects. Plants follow scale-project-sfc.ts: one construct per line,
 * recorded by its 1-based line. In a Nuxt project the page is the feature's
 * panel component and its rows come from an auto-imported composable.
 */

import {
  type Ctx,
  type PlantKind,
  type Source,
  featureDir,
  layerName,
  widgetName,
} from "./scale-project-sfc.ts";

function builder() {
  const lines: string[] = [];
  const plants: Source["plants"] = [];
  return {
    lines,
    plants,
    add: (...more: string[]) => void lines.push(...more),
    plant: (kind: PlantKind, line: string) => {
      lines.push(line);
      plants.push({ kind, line: lines.length });
    },
  };
}

/** Pages form a binary tree: page k renders pages 2k+1 and 2k+2, like nested routes. */
export const childPages = (k: number, features: number): number[] =>
  [2 * k + 1, 2 * k + 2].filter((child) => child < features);

/** Rows a feature starts with; the seed varies it per feature. */
export const rowCount = (k: number, seed: number): number => 6 + ((k + seed) % 5);

export function page(
  k: number,
  features: number,
  planted: boolean,
  parityPlants: boolean,
  ctx: Ctx,
): Source {
  const dir = featureDir(k);
  const used = [
    ...["KitAction", "KitCard", "KitField", "KitList", "KitSelect", "KitTabs"].map((name) => [
      `kit/${name}.vue`,
      name,
    ]),
    [`${dir}/${layerName(0)}.vue`, layerName(0)],
    [`${dir}/${widgetName(1)}.vue`, widgetName(1)],
    ...childPages(k, features).map((child) => [`${featureDir(child)}/Page.vue`, `Page${child}`]),
  ].map(([logical, local]) => ({ local, ...ctx.component(logical as string, local as string) }));
  const tag = (local: string) => used.find((use) => use.local === local)?.tag ?? local;
  const children = childPages(k, features).map((child) => tag(`Page${child}`));
  const [layer0, widget1] = [tag(layerName(0)), tag(widgetName(1))];
  const out = builder();
  out.add(
    '<script setup lang="ts">',
    'import { ref } from "vue";',
    ...used.flatMap((use) => use.imports),
    `import { TABS_${k}, ${ctx.nuxt ? "" : `makeRows${k}, `}toneOf${k}, type Row${k}, type Tab${k} } from "${ctx.to(`${dir}/model.ts`)}";`,
    "",
    ...(ctx.nuxt
      ? [`const { rows, query } = useFeature${k}();`]
      : [
          `const rows = ref<Row${k}[]>(makeRows${k}(${rowCount(k, ctx.seed)}));`,
          'const query = ref("");',
        ]),
    `const picked = ref<Row${k} | undefined>(undefined);`,
    `const tab = ref<Tab${k}>("overview${k}");`,
    'const note = ref("");',
    "const noteTouched = ref(false);",
    "const count = ref(0);",
    `function onRowClick(row: Row${k}, index: number): void {`,
    "  picked.value = row;",
    "  count.value = index;",
    "}",
    ...(planted
      ? [
          "const climate = ref(42);",
          "function onPlantCommit(value: number): void {",
          "  count.value = value;",
          "}",
          "function takesLabel(value: string): string {",
          "  return value;",
          "}",
        ]
      : []),
    "</script>",
    "",
    "<template>",
    `  <KitCard title="Feature ${k}" :tone="picked ? toneOf${k}(picked) : 'neutral'" @toggle-collapse="(collapsed) => (count = collapsed ? 1 : 0)">`,
    `    <KitTabs v-model="tab" :tabs="TABS_${k}">`,
    "      <template #tab=\"{ tab: name, active }\"><GIcon :name=\"active ? 'star' : 'flag'\" /><span>{{ name }}</span></template>",
    '      <template #panel="{ tab: current }"><span>{{ current }}</span></template>',
    "    </KitTabs>",
    `    <${layer0} :rows="rows" v-model:query="query" :limit="20" @row-click="onRowClick">`,
    '      <template #row="{ row, index, level }">',
    `        <span>{{ row.score${k}.toFixed(level) }} {{ index }}</span>`,
    `        <GBadge :count="row.tags${k}.length" :tone="toneOf${k}(row)" />`,
    "      </template>",
    `    </${layer0}>`,
    '    <KitSelect v-model="picked" :options="rows" :label-of="(row) => row.label">',
    `      <template #option="{ option, label, active }"><GAvatar :user="{ name: label }" /><b v-if="active">{{ option.score${k} }}</b></template>`,
    "    </KitSelect>",
    `    <KitList :items="rows" sort-key="score${k}" @select="(item, key) => (count = item.id + (key ? 1 : 0))">`,
    '      <template #header="{ count: total }"><GBadge :count="total" /></template>',
    '      <template #default="{ item, value }"><span>{{ item.label }} {{ value?.toFixed(1) }}</span></template>',
    "    </KitList>",
    '    <KitField v-model="note" v-model:touched="noteTouched" label="Note" @blur-commit="(value) => (note = value.trim())" @clear-request="note = \'\'">',
    '      <template #hint="{ length, touched }"><GTooltip :text="`${length}`"><GSpinner v-if="!touched" /></GTooltip></template>',
    "    </KitField>",
    `    <${widget1} :rows="rows" v-model:filter="query" @apply-filter="(value) => (query = value)" />`,
    '    <KitAction action-label="Save" @save-item="(value) => (note = value)" />',
  );
  if (planted) {
    out.plant("generic", `    <KitSelect :options="rows" :label-of="(row) => row.score${k}" />`);
    out.plant(
      "slot",
      `    <${layer0} :rows="rows" query=""><template #row="{ row }">{{ row.missing${k} }}</template></${layer0}>`,
    );
    out.plant(
      "emit-template",
      '    <KitField v-model="note" label="Plant" @blur-commit="onPlantCommit" />',
    );
    out.plant("v-model", '    <KitField v-model="climate" label="Plant" />');
    out.plant("global", "    <GBadge :count=\"'many'\" />");
    // These two plants deliberately expose a known reference-mapper miss. Keep
    // them in correctness corpora, but not in the exact-parity performance corpus.
    if (!parityPlants) {
      out.plant(
        "listener-arrow",
        '    <KitAction action-label="Plant" @save-item="(value: number) => undefined" />',
      );
      out.plant(
        "listener-prop",
        '    <KitAction :action-label="1" @save-item="(value: number) => undefined" />',
      );
    }
    out.plant(
      "listener-statement",
      '    <KitAction action-label="Plant" @save-item="takesLabel(count)" />',
    );
  }
  out.add(
    "    <template #actions=\"{ close, tone }\"><GButton :variant=\"tone === 'danger' ? 'primary' : 'ghost'\" @press=\"close()\" /></template>",
    "  </KitCard>",
    ...children.map((child) => `  <${child} />`),
    "</template>",
  );
  return { lines: out.lines, plants: out.plants };
}

const widgetContract = [
  'const emit = defineEmits<{ "update:filter": [value: string]; "apply-filter": [value: string] }>();',
];

/**
 * Widget `index` of feature `k`, one of three shapes: a six-branch `v-if`
 * chain narrowing a discriminated union, a two-parameter generic list with
 * nested `v-for`, and a `defineModel` widget driving generic tabs.
 */
export function widget(
  k: number,
  index: number,
  last: boolean,
  planted: boolean,
  ctx: Ctx,
): Source {
  const dir = featureDir(k);
  const nextUse = last
    ? null
    : ctx.component(`${dir}/${widgetName(index + 1)}.vue`, widgetName(index + 1));
  const next = nextUse?.tag ?? null;
  const out = builder();
  const nextImport = nextUse?.imports ?? [];
  const model = ctx.to(`${dir}/model.ts`);
  const kit = (name: string) => ctx.component(`kit/${name}.vue`, name);
  const [list, field, tabs] = [kit("KitList"), kit("KitField"), kit("KitTabs")];
  const forward = next
    ? `<${next} :rows="rows" :filter="filter" @update:filter="(value) => emit('update:filter', value)" @apply-filter="(value) => emit('apply-filter', value)" />`
    : null;
  const shape = index % 3;
  out.add('<script setup lang="ts">');
  if (shape === 0) {
    out.add(
      'import { computed } from "vue";',
      `import type { Row${k} } from "${model}";`,
      ...nextImport,
      `const props = defineProps<{ rows: Row${k}[]; filter: string }>();`,
      ...widgetContract,
      "const first = computed(() => props.rows[0]);",
    );
  } else if (shape === 1) {
    out.add(
      'import { ref } from "vue";',
      ...list.imports,
      `import type { Row${k} } from "${model}";`,
      ...nextImport,
      `defineProps<{ rows: Row${k}[]; filter: string }>();`,
      ...widgetContract,
      "const lastId = ref<number | null>(null);",
      `function select(row: Row${k}): void {`,
      "  lastId.value = row.id;",
      '  emit("apply-filter", row.label);',
      "}",
    );
  } else {
    out.add(
      ...field.imports,
      ...tabs.imports,
      `import { TABS_${k}, type Row${k}, type Tab${k} } from "${model}";`,
      ...nextImport,
      `defineProps<{ rows: Row${k}[] }>();`,
      'const filter = defineModel<string>("filter", { required: true });',
      `const tab = defineModel<Tab${k}>("tab", { default: "overview${k}" });`,
      'const emit = defineEmits<{ "apply-filter": [value: string] }>();',
    );
  }
  if (planted)
    out.plant("emit-script", 'const plantedEmit = (): void => emit("apply-filter", 42);');
  out.add("</script>", "", "<template>");
  if (shape === 0) {
    out.add(
      '  <div v-for="row in rows" :key="row.id">',
      '    <GIcon v-if="row.status.kind === \'idle\'" name="clock" />',
      "    <GSpinner v-else-if=\"row.status.kind === 'loading'\" :size=\"row.status.progress > 50 ? 'lg' : 'sm'\" />",
      '    <GBadge v-else-if="row.status.kind === \'ready\'" :count="row.status.count" />',
      '    <GTooltip v-else-if="row.status.kind === \'stale\'" :text="String(row.status.since)" />',
      "    <span v-else-if=\"row.status.kind === 'failed'\">{{ row.status.reason.toUpperCase() }}</span>",
      '    <GAvatar v-else :user="{ name: row.status.by }" />',
      "  </div>",
      '  <p v-if="first">{{ first.label }} {{ filter }}</p>',
    );
  } else if (shape === 1) {
    out.add(
      `  <${list.tag} :items="rows" sort-key="tags${k}" @select="(item) => select(item)">`,
      '    <template #header="{ count }"><GBadge :count="count" tone="info" /></template>',
      '    <template #default="{ item, value }">',
      '      <span v-for="tag in value ?? []" :key="tag">{{ tag.length }} {{ item.label }} {{ filter }}</span>',
      "    </template>",
      `  </${list.tag}>`,
    );
  } else {
    out.add(
      `  <${tabs.tag} v-model="tab" :tabs="TABS_${k}">`,
      "    <template #tab=\"{ tab: name, active }\"><GIcon :name=\"active ? 'check' : 'cross'\" />{{ name }}</template>",
      '    <template #panel="{ tab: current }"><${field.tag} v-model="filter" :label="current" @blur-commit="(value) => emit(\'apply-filter\', value)" /></template>',
      `  </${tabs.tag}>`,
    );
  }
  if (next) {
    out.add(
      shape === 2
        ? `  <${next} :rows="rows" v-model:filter="filter" @apply-filter="(value) => emit('apply-filter', value)" />`
        : `  ${forward}`,
    );
  }
  out.add("</template>");
  return { lines: out.lines, plants: out.plants };
}

/** The single project's root; a Nuxt project's root is `app.vue` with its index page. */
export function app(ctx: Ctx): Source {
  const root = ctx.component(`${featureDir(0)}/Page.vue`, "Page0");
  return {
    lines: [
      '<script setup lang="ts">',
      ...root.imports,
      "</script>",
      "",
      "<template>",
      `  <${root.tag} />`,
      "</template>",
    ],
    plants: [],
  };
}
