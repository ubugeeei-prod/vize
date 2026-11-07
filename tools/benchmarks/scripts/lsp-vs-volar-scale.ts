/**
 * Generated projects for `lsp-vs-volar.ts`: deterministic Vue apps whose
 * size grows while the edited SFC and its neighbourhood stay byte-identical.
 *
 * `scale-<n>k` is one app of exactly n thousand SFCs: the shared layer, the
 * core cluster around the edited SFC (lsp-vs-volar-scale-core.ts) and
 * unrelated features of 25 SFCs, each a 25-deep generic chain with scoped
 * slots and typed emits, routed lazily. `scale-monorepo` is an npm workspace
 * of 48 packages joined by tsconfig project references: shared libraries
 * exposed through package exports, 36 domain libraries whose models extend
 * each other in a chain, and six apps; the edited SFC sits in the first app.
 */

import {
  type FocusImports,
  CHAIN_LEVELS,
  CORE_SFCS,
  FOCUS_TEMPLATE_PROBE_AFTER,
  HOSTS,
  MAIN_SOURCE,
  MODEL_STEPS,
  UPPER_LEVELS,
  appSource,
  focusSource,
  hostSource,
  levelSource,
  modelIndexSource,
  ordersSource,
  routesSource,
  stepSource,
  upperSource,
} from "./lsp-vs-volar-scale-core.ts";
import {
  COMPOSABLES_INDEX,
  KIT,
  KIT_INDEX,
  PAGER_SOURCE,
  SESSION_STORE_SOURCE,
  STORES_INDEX,
  TYPES_SOURCE,
  globalComponentsSource,
  kitSource,
  orderStoreSource,
  selectionSource,
} from "./lsp-vs-volar-scale-kit.ts";

export type ProjectFiles = Map<string, string>;

const two = (index: number) => String(index).padStart(2, "0");
const json = (value: unknown) => `${JSON.stringify(value, null, 2)}\n`;

export const FEATURE_VIEWS = 25;

export interface FeatureOptions {
  /** Unique across the project: names its types, functions and SFCs. */
  id: string;
  views: number;
  /** The entity its items extend, and where that comes from. */
  base: { name: string; from: string };
  composables: string;
  stores: string;
  /** Explicit kit import in a library; `null` uses the global registration. */
  kit: string | null;
}

const VIEW_KIT = "UiBadge, UiButton, UiCard, UiList, UiTable, UiTabs";

/** A feature: model, composable and a chain of views, every third generic. */
export function featureFiles(options: FeatureOptions): ProjectFiles {
  const { id, base } = options;
  const files: ProjectFiles = new Map();
  files.set(
    "model.ts",
    `import type { ${base.name} } from "${base.from}";

export interface Item${id} extends ${base.name} {
  score${id}: number;
  kind: "alpha" | "beta";
}

export function rank${id}<T extends Item${id}>(items: readonly T[]): T[] {
  return [...items].sort((left, right) => right.score${id} - left.score${id});
}
`,
  );
  files.set(
    `use${id}.ts`,
    `import { computed } from "vue";

import { useSelection } from "${options.composables}";
import { useSessionStore } from "${options.stores}";
import type { Item${id} } from "./model";

export function use${id}() {
  const selection = useSelection<Item${id}>();
  const session = useSessionStore();
  const caption = computed(() => \`\${session.state.locale}:\${selection.count.value}\`);
  function toggleAll(items: readonly Item${id}[]): void {
    for (const item of items) selection.toggle(item);
  }
  return { caption, toggleAll, isSelected: selection.isSelected };
}
`,
  );
  for (let view = 0; view < options.views; view += 1) {
    const generic = view % 3 === 0;
    const T = generic ? "T" : `Item${id}`;
    const next = view + 1 < options.views ? `${id}View${two(view + 1)}` : null;
    const imports = [
      'import { computed, ref } from "vue";',
      "",
      `import { type Item${id}, rank${id} } from "./model";`,
      `import { use${id} } from "./use${id}";`,
    ];
    if (next) imports.push(`import ${next} from "./${next}.vue";`);
    if (options.kit) imports.push(`import { ${VIEW_KIT} } from "${options.kit}";`);
    const pick = generic ? "\n  pick: [item: T];" : "";
    const select = generic ? "emit('pick', item)" : "emit('open', item)";
    const badge = `<UiBadge :tone="isSelected(item) ? 'info' : 'warn'">{{ index }} {{ item.label }}</UiBadge>`;
    const item = generic ? `<slot name="badge" :item="item" :index="index">${badge}</slot>` : badge;
    files.set(
      `${id}View${two(view)}.vue`,
      `<script setup lang="ts"${generic ? ` generic="T extends Item${id}"` : ""}>
${imports.join("\n")}

const props = defineProps<{ items: readonly ${T}[]; title: string }>();
const emit = defineEmits<{
  open: [item: Item${id}];${pick}
  rank: [count: number];
  close: [reason: string];
}>();${generic ? "\ndefineSlots<{ badge?(props: { item: T; index: number }): any }>();" : ""}
const { caption, toggleAll, isSelected } = use${id}();
const ranked = computed(() => rank${id}(props.items));
const tab = ref<"list" | "table">("list");
</script>

<template>
  <UiCard :title="title">
    <template #header>
      <UiTabs v-model:active="tab" :tabs="['list', 'table']">
        <template #tab="{ name, active }">{{ active ? name.toUpperCase() : name }}</template>
      </UiTabs>
    </template>
    <UiList v-if="tab === 'list'" :items="ranked" @select="(item) => ${select}">
      <template #item="{ item, index }">${item}</template>
    </UiList>
    <UiTable v-else :rows="ranked" :columns="['label', 'score${id}']" @open="(row) => emit('open', row)">
      <template #cell="{ column, value }">{{ column }}: {{ value }}</template>
    </UiTable>${
      next
        ? `
    <${next} :items="items" title="next" @open="(item) => emit('open', item)" @rank="(count) => emit('rank', count + 1)" @close="(reason) => emit('close', reason)" />`
        : ""
    }
    <UiButton @press="toggleAll(items)">{{ caption }}</UiButton>
  </UiCard>
</template>
`,
    );
  }
  return files;
}

function place(files: ProjectFiles, dir: string, entries: ProjectFiles) {
  for (const [path, text] of entries) files.set(`${dir}/${path}`, text);
}

function featureIds(count: number, prefix: string, digits: number) {
  return Array.from(
    { length: count },
    (_, index) => `${prefix}${String(index).padStart(digits, "0")}`,
  );
}

const COMPILER_OPTIONS = {
  target: "ES2022",
  module: "ESNext",
  moduleResolution: "bundler",
  lib: ["ES2022", "DOM"],
  strict: true,
  jsx: "preserve",
  skipLibCheck: true,
  types: [],
};

/**
 * The edited SFC's neighbourhood inside an app's `src`: hosts, chain above,
 * root and entry. `model` is the model's specifier from `src/core/<dir>` and from `src`.
 */
function coreApp(
  files: ProjectFiles,
  src: string,
  imports: FocusImports,
  model: { core: string; root: string },
) {
  files.set(`${src}/core/OrderBoard.vue`, focusSource(imports));
  for (let index = 0; index < HOSTS; index += 1)
    files.set(`${src}/core/hosts/BoardHost${two(index)}.vue`, hostSource(index, model.core));
  for (let index = 0; index < UPPER_LEVELS; index += 1)
    files.set(`${src}/core/upper/Upper${two(index)}.vue`, upperSource(index, model.core));
  files.set(`${src}/App.vue`, appSource(model.root));
  files.set(`${src}/main.ts`, MAIN_SOURCE);
}

/** One app of exactly `sfcs` SFCs; only the features past the core grow with it. */
export function scaleProject(sfcs: number, vue: string): ProjectFiles {
  const filler = sfcs - KIT.length - CORE_SFCS;
  if (!Number.isSafeInteger(filler) || filler < 0)
    throw new Error(`a scale project needs at least ${KIT.length + CORE_SFCS} SFCs`);
  const files: ProjectFiles = new Map();
  files.set(
    "package.json",
    json({ name: "lsp-scale", private: true, type: "module", dependencies: { vue } }),
  );
  files.set(
    "tsconfig.json",
    json({
      compilerOptions: { ...COMPILER_OPTIONS, noEmit: true },
      include: ["src/**/*.ts", "src/**/*.vue"],
    }),
  );
  files.set("src/shared/types.ts", TYPES_SOURCE);
  files.set("src/shared/composables/index.ts", COMPOSABLES_INDEX);
  files.set("src/shared/composables/usePager.ts", PAGER_SOURCE);
  files.set("src/shared/composables/useSelection.ts", selectionSource("../types"));
  files.set("src/shared/stores/index.ts", STORES_INDEX);
  files.set("src/shared/stores/orders.ts", orderStoreSource("../types"));
  files.set("src/shared/stores/session.ts", SESSION_STORE_SOURCE);
  for (const name of KIT) files.set(`src/shared/kit/${name}.vue`, kitSource(name, "../types"));
  files.set(
    "src/components.d.ts",
    globalComponentsSource((name) => `./shared/kit/${name}.vue`),
  );
  for (let index = 0; index < MODEL_STEPS; index += 1)
    files.set(`src/core/model/step-${two(index)}.ts`, stepSource(index, "../../shared/types"));
  files.set("src/core/model/orders.ts", ordersSource("../../shared/types"));
  files.set("src/core/model/index.ts", modelIndexSource());
  for (let index = 0; index < CHAIN_LEVELS; index += 1)
    files.set(`src/core/chain/Level${two(index)}.vue`, levelSource(index, "../model", null));
  const imports = {
    chain: "./chain",
    model: "./model",
    composables: "../shared/composables",
    stores: "../shared/stores",
  };
  coreApp(files, "src", imports, { core: "../model", root: "./core/model" });
  const roots: string[] = [];
  featureIds(Math.ceil(filler / FEATURE_VIEWS), "F", 3).forEach((id, index) => {
    const views = Math.min(FEATURE_VIEWS, filler - index * FEATURE_VIEWS);
    const base = { name: "Entity", from: "../../shared/types" };
    const shared = { composables: "../../shared/composables", stores: "../../shared/stores" };
    place(files, `src/features/${id}`, featureFiles({ id, views, base, ...shared, kit: null }));
    roots.push(`features/${id}/${id}View00.vue`);
  });
  files.set("src/routes.ts", routesSource(roots));
  return files;
}

/** A normal connected project whose edited SFC alone is deliberately enormous. */
export function giantSfcProject(vue: string, blocks = 10_000): ProjectFiles {
  if (!Number.isSafeInteger(blocks) || blocks < 1)
    throw new Error(`a giant SFC needs a positive block count, got ${blocks}`);
  const files = scaleProject(KIT.length + CORE_SFCS, vue);
  const path = "src/core/OrderBoard.vue";
  const source = files.get(path)!;
  const giant = Array.from(
    { length: blocks },
    (_, index) =>
      `    <article data-giant-block="${index}">{{ boardSummary.total + ${index} }} / {{ visible.length }}</article>`,
  ).join("\n");
  files.set(
    path,
    source.replace(FOCUS_TEMPLATE_PROBE_AFTER, `${giant}\n${FOCUS_TEMPLATE_PROBE_AFTER}`),
  );
  return files;
}

export const MONOREPO_DOMAINS = 36;
export const MONOREPO_APPS = 6;
const APP_FEATURES = 2;
const APP_FEATURE_VIEWS = 20;

interface Workspace {
  dir: string;
  name: string;
  deps: string[];
  /** Libraries expose their source through package exports. */
  library: boolean;
}

function workspacePackage(files: ProjectFiles, member: Workspace, dirs: Map<string, string>) {
  const up = member.dir
    .split("/")
    .map(() => "..")
    .join("/");
  const exports = { ".": "./src/index.ts", "./*.vue": "./src/*.vue" };
  files.set(
    `${member.dir}/package.json`,
    json({
      name: member.name,
      version: "0.0.0",
      private: true,
      type: "module",
      ...(member.library && { exports }),
      dependencies: Object.fromEntries(member.deps.map((dep) => [dep, "*"])),
    }),
  );
  // Every member is composite, so the root solution and each reference build with `tsc -b`.
  const options = { composite: true, emitDeclarationOnly: true, rootDir: "src", outDir: "dist" };
  files.set(
    `${member.dir}/tsconfig.json`,
    json({
      extends: `${up}/tsconfig.base.json`,
      compilerOptions: options,
      include: ["src/**/*.ts", "src/**/*.vue"],
      references: member.deps.map((dep) => ({ path: `${up}/${dirs.get(dep)}` })),
    }),
  );
}

/** Shared libraries, a chain of domain libraries and apps, joined by project references. */
export function monorepoProject(vue: string): ProjectFiles {
  const files: ProjectFiles = new Map();
  const members: Workspace[] = [];
  const add = (dir: string, deps: string[], library = true) =>
    members.push({ dir, name: `@mono/${dir.split("/")[1]}`, deps, library });
  const kit = "@mono/ui-kit";
  const common = ["@mono/types", "@mono/composables", "@mono/stores"];

  files.set("packages/types/src/index.ts", TYPES_SOURCE);
  add("packages/types", []);
  files.set("packages/composables/src/index.ts", COMPOSABLES_INDEX);
  files.set("packages/composables/src/usePager.ts", PAGER_SOURCE);
  files.set("packages/composables/src/useSelection.ts", selectionSource("@mono/types"));
  add("packages/composables", ["@mono/types"]);
  files.set("packages/stores/src/index.ts", STORES_INDEX);
  files.set("packages/stores/src/orders.ts", orderStoreSource("@mono/types"));
  files.set("packages/stores/src/session.ts", SESSION_STORE_SOURCE);
  add("packages/stores", ["@mono/types"]);
  for (const name of KIT)
    files.set(`packages/ui-kit/src/${name}.vue`, kitSource(name, "@mono/types"));
  files.set("packages/ui-kit/src/index.ts", KIT_INDEX);
  add("packages/ui-kit", ["@mono/types"]);
  for (let index = 0; index < MODEL_STEPS; index += 1)
    files.set(`packages/model/src/step-${two(index)}.ts`, stepSource(index, "@mono/types"));
  files.set("packages/model/src/orders.ts", ordersSource("@mono/types"));
  files.set("packages/model/src/index.ts", modelIndexSource());
  add("packages/model", ["@mono/types"]);
  const levels = Array.from({ length: CHAIN_LEVELS }, (_, index) => `Level${two(index)}`);
  levels.forEach((name, index) =>
    files.set(`packages/board/src/${name}.vue`, levelSource(index, "@mono/model", kit)),
  );
  files.set(
    "packages/board/src/index.ts",
    levels
      .map((name) => `export { default as ${name} } from "./${name}.vue";`)
      .join("\n")
      .concat("\n"),
  );
  add("packages/board", ["@mono/model", kit]);

  featureIds(MONOREPO_DOMAINS, "D", 2).forEach((id, index) => {
    const dir = `packages/domain-${two(index)}`;
    const parent = index === 0 ? null : `@mono/domain-${two(index - 1)}`;
    const base = parent
      ? { name: `ItemD${two(index - 1)}`, from: parent }
      : { name: "Entity", from: "@mono/types" };
    const feature = featureFiles({
      id,
      views: FEATURE_VIEWS,
      base,
      composables: "@mono/composables",
      stores: "@mono/stores",
      kit,
    });
    place(files, `${dir}/src`, feature);
    files.set(
      `${dir}/src/index.ts`,
      `export * from "./model";\nexport { use${id} } from "./use${id}";\nexport { default as ${id}Root } from "./${id}View00.vue";\n`,
    );
    add(dir, [...common, kit, ...(parent ? [parent] : [])]);
  });

  for (let app = 0; app < MONOREPO_APPS; app += 1) {
    const dir = `apps/app-${app}`;
    const domains = Array.from({ length: 6 }, (_, slot) => (app * 7 + slot * 5) % MONOREPO_DOMAINS);
    const used = [...new Set(domains)]
      .sort((a, b) => a - b)
      .map((index) => `@mono/domain-${two(index)}`);
    files.set(
      `${dir}/src/components.d.ts`,
      globalComponentsSource((name) => `${kit}/${name}.vue`),
    );
    const roots: string[] = [];
    featureIds(APP_FEATURES, `A${app}F`, 1).forEach((id, slot) => {
      const parent = used[slot % used.length];
      const base = { name: `ItemD${parent.slice(-2)}`, from: parent };
      const feature = featureFiles({
        id,
        views: APP_FEATURE_VIEWS,
        base,
        composables: "@mono/composables",
        stores: "@mono/stores",
        kit: null,
      });
      place(files, `${dir}/src/views/${id}`, feature);
      roots.push(`views/${id}/${id}View00.vue`);
    });
    if (app === 0) {
      const imports = {
        chain: "@mono/board",
        model: "@mono/model",
        composables: "@mono/composables",
        stores: "@mono/stores",
      };
      coreApp(files, `${dir}/src`, imports, { core: "@mono/model", root: "@mono/model" });
      files.set(`${dir}/src/routes.ts`, routesSource(roots));
      add(dir, [...common, "@mono/model", kit, "@mono/board", ...used], false);
      continue;
    }
    const names = used.map((dep) => `D${dep.slice(-2)}`);
    files.set(
      `${dir}/src/App.vue`,
      `<script setup lang="ts">
${names.map((id, slot) => `import { ${id}Root, type Item${id} } from "${used[slot]}";`).join("\n")}
${roots.map((path) => `import ${path.split("/").at(-1)!.replace(".vue", "")} from "./${path}";`).join("\n")}

${names.map((id) => `const items${id}: Item${id}[] = [];`).join("\n")}
</script>

<template>
  <main>
${names.map((id) => `    <${id}Root :items="items${id}" title="${id}" @close="(reason) => reason.length" />`).join("\n")}
${roots.map((path) => `    <${path.split("/").at(-1)!.replace(".vue", "")} :items="[]" title="local" />`).join("\n")}
  </main>
</template>
`,
    );
    files.set(
      `${dir}/src/main.ts`,
      `import { createApp } from "vue";\n\nimport App from "./App.vue";\n\ncreateApp(App).mount("#app");\n`,
    );
    add(dir, [...common, kit, ...used], false);
  }

  const dirs = new Map(members.map((member) => [member.name, member.dir]));
  for (const member of members) workspacePackage(files, member, dirs);
  files.set("tsconfig.base.json", json({ compilerOptions: COMPILER_OPTIONS }));
  files.set(
    "tsconfig.json",
    json({ files: [], references: members.map((member) => ({ path: `./${member.dir}` })) }),
  );
  files.set(
    "package.json",
    json({
      name: "lsp-scale-monorepo",
      private: true,
      type: "module",
      workspaces: ["packages/*", "apps/*"],
      dependencies: { vue },
    }),
  );
  return files;
}
