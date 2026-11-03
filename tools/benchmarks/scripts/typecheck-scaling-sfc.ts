/**
 * Source text for the type-check scaling corpus (typecheck-scaling-corpus.ts).
 *
 * Every builder returns the file's lines and the 1-based lines of the type
 * errors it plants, so the gate knows where each tool must report one. A
 * planted line holds exactly one planted construct and nothing else that can
 * fail. `to` maps a logical corpus path ("kit/KitTable.vue") to the import
 * specifier from the file being built.
 */

export const PLANT_KINDS = [
  "script",
  "emit-script",
  "generic",
  "slot",
  "emit-template",
  "v-model",
  "global",
] as const;
export type PlantKind = (typeof PLANT_KINDS)[number];
export type Source = { lines: string[]; plants: { kind: PlantKind; line: number }[] };
export type Spec = (logical: string) => string;

/** Kit components: generic tables, selects, lists and tabs, plus v-model fields and cards. */
export const KIT = ["KitTable", "KitSelect", "KitList", "KitField", "KitCard", "KitTabs"] as const;
/** Components registered through the `GlobalComponents` augmentation. */
export const GLOBALS = ["GBadge", "GIcon", "GTooltip", "GButton", "GAvatar", "GSpinner"] as const;
/** Generic layers per feature; each imports the next, the last renders KitTable. */
export const LAYER_DEPTH = 10;
/** Widgets per feature; each imports the next. */
export const WIDGETS = 9;

const source = (lines: string[]): Source => ({ lines, plants: [] });
const pad = (value: number) => String(value).padStart(2, "0");
export const layerName = (index: number) => `Layer${pad(index)}`;
export const widgetName = (index: number) => `Widget${pad(index)}`;

export function sharedTypes(): Source {
  return source([
    "export interface Entity {",
    "  id: number;",
    "  label: string;",
    "}",
    'export type Tone = "neutral" | "info" | "warn" | "danger";',
    'export type IconName = "check" | "cross" | "clock" | "star" | "flag" | "bell";',
    "export interface Column<T> {",
    "  key: keyof T & string;",
    "  title: string;",
    "}",
    "export type Status =",
    '  | { kind: "idle" }',
    '  | { kind: "loading"; progress: number }',
    '  | { kind: "ready"; count: number }',
    '  | { kind: "stale"; since: number }',
    '  | { kind: "failed"; reason: string }',
    '  | { kind: "archived"; by: string };',
  ]);
}

const sfc = (script: string[], template: string[], generic?: string): string[] => [
  `<script setup lang="ts"${generic ? ` generic="${generic}"` : ""}>`,
  ...script,
  "</script>",
  "",
  "<template>",
  ...template.map((line) => `  ${line}`),
  "</template>",
];

export function kitComponent(name: (typeof KIT)[number], to: Spec): Source {
  const types = to("shared/types.ts");
  switch (name) {
    case "KitTable":
      return source(
        sfc(
          [
            `import type { Column, Entity } from "${types}";`,
            "",
            "defineProps<{ rows: T[]; columns: Column<T>[]; selected?: T | null }>();",
            "const emit = defineEmits<{",
            '  "row-click": [row: T, index: number];',
            '  "update:selected": [row: T | null];',
            "}>();",
            "defineSlots<{",
            "  cell(props: { row: T; column: Column<T>; index: number }): any;",
            "  empty(props: Record<string, never>): any;",
            "}>();",
            "",
            "function pick(row: T, index: number): void {",
            '  emit("row-click", row, index);',
            '  emit("update:selected", row);',
            "}",
          ],
          [
            "<table>",
            '  <tbody v-if="rows.length > 0">',
            '    <tr v-for="(row, index) in rows" :key="row.id" :class="{ active: row === selected }" @click="pick(row, index)">',
            '      <td v-for="column in columns" :key="column.key">',
            '        <slot name="cell" :row="row" :column="column" :index="index" />',
            "      </td>",
            "    </tr>",
            "  </tbody>",
            '  <caption v-else><slot name="empty" /></caption>',
            "</table>",
          ],
          "T extends Entity",
        ),
      );
    case "KitSelect":
      return source(
        sfc(
          [
            'import { computed } from "vue";',
            "",
            "const props = defineProps<{",
            "  options: readonly T[];",
            "  modelValue?: T;",
            "  labelOf: (option: T) => string;",
            "  placeholder?: string;",
            "}>();",
            "const emit = defineEmits<{",
            '  "update:modelValue": [value: T];',
            '  "open-change": [open: boolean];',
            "}>();",
            "defineSlots<{",
            "  option(props: { option: T; label: string; active: boolean }): any;",
            "}>();",
            "",
            "const labels = computed(() => props.options.map((option) => props.labelOf(option)));",
            "function choose(option: T): void {",
            '  emit("update:modelValue", option);',
            '  emit("open-change", false);',
            "}",
          ],
          [
            "<ul :aria-label=\"placeholder ?? 'select'\">",
            '  <li v-for="(option, index) in options" :key="index" @click="choose(option)">',
            '    <slot name="option" :option="option" :label="labels[index] ?? \'\'" :active="option === modelValue" />',
            "  </li>",
            "</ul>",
          ],
          "T",
        ),
      );
    case "KitList":
      return source(
        sfc(
          [
            `import type { Entity } from "${types}";`,
            "",
            "defineProps<{ items: T[]; sortKey?: K }>();",
            "const emit = defineEmits<{ select: [item: T, key: K | undefined] }>();",
            "defineSlots<{",
            "  default(props: { item: T; index: number; value: T[K] | undefined }): any;",
            "  header(props: { count: number }): any;",
            "}>();",
          ],
          [
            "<div>",
            '  <header><slot name="header" :count="items.length" /></header>',
            '  <article v-for="(item, index) in items" :key="item.id" @click="emit(\'select\', item, sortKey)">',
            '    <slot :item="item" :index="index" :value="sortKey === undefined ? undefined : item[sortKey]" />',
            "  </article>",
            "</div>",
          ],
          "T extends Entity, K extends keyof T",
        ),
      );
    case "KitField":
      return source(
        sfc(
          [
            "const model = defineModel<string>({ required: true });",
            'const touched = defineModel<boolean>("touched", { default: false });',
            "defineProps<{ label: string; maxLength?: number }>();",
            'const emit = defineEmits<{ "blur-commit": [value: string]; "clear-request": [] }>();',
            "defineSlots<{ hint(props: { length: number; touched: boolean }): any }>();",
            "",
            "function commit(): void {",
            "  touched.value = true;",
            '  emit("blur-commit", model.value);',
            "}",
          ],
          [
            "<label>",
            "  <span>{{ label }}</span>",
            '  <input v-model="model" :maxlength="maxLength" @blur="commit" @keydown.esc="emit(\'clear-request\')" />',
            '  <slot name="hint" :length="model.length" :touched="touched" />',
            "</label>",
          ],
        ),
      );
    case "KitCard":
      return source(
        sfc(
          [
            `import type { Tone } from "${types}";`,
            "",
            "const props = withDefaults(defineProps<{ title: string; tone?: Tone; collapsible?: boolean }>(), {",
            '  tone: "neutral",',
            "  collapsible: false,",
            "});",
            'const emit = defineEmits<{ close: []; "toggle-collapse": [collapsed: boolean] }>();',
            "defineSlots<{",
            "  default(props: Record<string, never>): any;",
            "  actions(props: { close: () => void; tone: Tone }): any;",
            "}>();",
            "",
            "let collapsed = false;",
            "function toggle(): void {",
            "  collapsed = !collapsed;",
            '  emit("toggle-collapse", collapsed);',
            "}",
          ],
          [
            "<section :class=\"['card', props.tone]\">",
            '  <h2 @click="toggle">{{ title }}</h2>',
            "  <slot />",
            '  <footer><slot name="actions" :close="() => emit(\'close\')" :tone="props.tone" /></footer>',
            "</section>",
          ],
        ),
      );
    case "KitTabs":
      return source(
        sfc(
          [
            "const props = defineProps<{ tabs: readonly K[]; modelValue: K }>();",
            'const emit = defineEmits<{ "update:modelValue": [tab: K] }>();',
            "defineSlots<{",
            "  tab(props: { tab: K; active: boolean }): any;",
            "  panel(props: { tab: K }): any;",
            "}>();",
          ],
          [
            "<nav>",
            '  <button v-for="tab in tabs" :key="tab" type="button" @click="emit(\'update:modelValue\', tab)">',
            '    <slot name="tab" :tab="tab" :active="tab === props.modelValue" />',
            "  </button>",
            "</nav>",
            '<slot name="panel" :tab="modelValue" />',
          ],
          "K extends string",
        ),
      );
  }
}

export function globalComponent(name: (typeof GLOBALS)[number], to: Spec): Source {
  const types = to("shared/types.ts");
  const bodies: Record<(typeof GLOBALS)[number], [string[], string[]]> = {
    GBadge: [
      [`import type { Tone } from "${types}";`, "defineProps<{ count: number; tone?: Tone }>();"],
      ['<b :title="tone">{{ count.toFixed(0) }}</b>'],
    ],
    GIcon: [
      [
        `import type { IconName } from "${types}";`,
        "defineProps<{ name: IconName; size?: number }>();",
      ],
      ['<i :class="`icon-${name}`" :style="{ fontSize: `${size ?? 16}px` }" />'],
    ],
    GTooltip: [
      [
        "defineProps<{ text: string }>();",
        "defineSlots<{ default(props: Record<string, never>): any }>();",
      ],
      ['<span :title="text"><slot /></span>'],
    ],
    GButton: [
      [
        'defineProps<{ variant?: "primary" | "ghost" }>();',
        "const emit = defineEmits<{ press: [event: MouseEvent] }>();",
      ],
      ['<button type="button" :class="variant" @click="(event) => emit(\'press\', event)" />'],
    ],
    GAvatar: [
      ["defineProps<{ user: { name: string; url?: string } }>();"],
      ['<img :alt="user.name" :src="user.url ?? \'/avatar.png\'" />'],
    ],
    GSpinner: [
      ['defineProps<{ size?: "sm" | "md" | "lg" }>();'],
      ["<span :class=\"['spinner', size ?? 'md']\" />"],
    ],
  };
  const [script, template] = bodies[name];
  return source(sfc(script, template));
}

/** The `GlobalComponents` augmentation, as unplugin-vue-components writes it. */
export function globalDeclarations(to: Spec): Source {
  return source([
    "export {};",
    'declare module "vue" {',
    "  export interface GlobalComponents {",
    ...GLOBALS.map(
      (name) => `    ${name}: (typeof import("${to(`global/${name}.vue`)}"))["default"];`,
    ),
    "  }",
    "}",
  ]);
}

export function featureModel(k: number, to: Spec): Source {
  return source([
    `import type { Entity, Status, Tone } from "${to("shared/types.ts")}";`,
    "",
    `export interface Row${k} extends Entity {`,
    `  score${k}: number;`,
    `  tags${k}: string[];`,
    "  status: Status;",
    "}",
    `export type Tab${k} = "overview${k}" | "details${k}" | "history${k}";`,
    `export const TABS_${k}: readonly Tab${k}[] = ["overview${k}", "details${k}", "history${k}"];`,
    `export function makeRows${k}(count: number): Row${k}[] {`,
    "  return Array.from({ length: count }, (_, index) => ({",
    "    id: index,",
    `    label: \`row${k}-\${index}\`,`,
    `    score${k}: index * ${k + 1},`,
    `    tags${k}: [],`,
    '    status: index % 2 === 0 ? { kind: "idle" } : { kind: "ready", count: index },',
    "  }));",
    "}",
    `export function toneOf${k}(row: Row${k}): Tone {`,
    `  return row.score${k} > 10 ? "warn" : "info";`,
    "}",
  ]);
}

const layerContract: string[] = [
  "const props = defineProps<{ rows: T[]; query: string; limit?: number }>();",
  "const emit = defineEmits<{",
  '  "row-click": [row: T, index: number];',
  '  "update:query": [value: string];',
  "}>();",
  "defineSlots<{",
  "  row(props: { row: T; index: number; level: number }): any;",
  "}>();",
];

/** Layer `index` of feature `k`: generic over the feature row, slot and emits forwarded. */
export function layer(k: number, index: number, planted: boolean, to: Spec): Source {
  const generic = `T extends Row${k}`;
  const model = `import type { Row${k} } from "./model";`;
  if (index < LAYER_DEPTH - 1) {
    const next = layerName(index + 1);
    return source(
      sfc(
        [
          'import { computed } from "vue";',
          `import ${next} from "./${next}.vue";`,
          model,
          "",
          ...layerContract,
          "",
          "const visible = computed(() =>",
          "  props.rows.filter((row) => row.label.includes(props.query)).slice(0, props.limit ?? 50),",
          ");",
          "const queryModel = computed({",
          "  get: () => props.query,",
          '  set: (value: string) => emit("update:query", value),',
          "});",
        ],
        [
          `<section :aria-level="${index + 1}">`,
          `  <${next} :rows="visible" v-model:query="queryModel" :limit="limit" @row-click="(row, index) => emit('row-click', row, index)">`,
          '    <template #row="{ row, index, level }">',
          '      <slot name="row" :row="row" :index="index" :level="level + 1" />',
          "    </template>",
          `  </${next}>`,
          "</section>",
        ],
        generic,
      ),
    );
  }
  const script = [
    'import { ref } from "vue";',
    `import type { Column } from "${to("shared/types.ts")}";`,
    `import KitTable from "${to("kit/KitTable.vue")}";`,
    model,
    "",
    ...layerContract,
    "",
    "const columns: Column<T>[] = [",
    '  { key: "label", title: "Label" },',
    `  { key: "score${k}", title: "Score" },`,
    "];",
    "const lastId = ref<number | null>(null);",
  ];
  const lines = sfc(
    planted ? [...script, `const planted${k}: number = "deep-${k}";`] : script,
    [
      `<KitTable :rows="rows" :columns="columns" :selected="null" @update:selected="(row) => (lastId = row ? row.id : null)" @row-click="(row, index) => emit('row-click', row, index)">`,
      '  <template #cell="{ row, column, index }">',
      '    <slot name="row" :row="row" :index="index" :level="0" />',
      `    <GBadge :count="row.score${k}" :tone="row.status.kind === 'failed' ? 'danger' : 'info'" />`,
      "    <span>{{ column.title }}</span>",
      "  </template>",
      "  <template #empty>",
      '    <GSpinner size="sm" />',
      "  </template>",
      "</KitTable>",
    ],
    generic,
  );
  return {
    lines,
    plants: planted ? [{ kind: "script", line: script.length + 2 }] : [],
  };
}
