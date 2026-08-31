/**
 * SFC bodies for the large-scale corpus.
 *
 * Four variants are cycled so the corpus is not a pile of identical files.
 * Each variant exercises a different real Vue surface:
 *
 * 0. props with defaults + `v-model` + `v-for`/`v-if` + scoped style
 * 1. `defineEmits` + `watch` + dynamic `:style` + attribute-bound asset URL
 * 2. slots + `provide` + `v-once`/`v-show` + CSS Modules style block
 * 3. `defineOptions` + `<style lang="css" scoped>` with `:deep()` + `v-html`
 *
 * Every body embeds the caller-supplied `id` token. This is deliberate and
 * load-bearing: identical bodies let content-addressed compile caches (the
 * vite plugin's persistent pre-compile cache, `vize build --format stats`)
 * serve most of the corpus from a hash lookup, which would make the benchmark
 * measure cache hits instead of compilation. `tools/benchmarks/scripts/generate.mjs` documents the
 * same constraint for the flat corpus.
 */

const VARIANT_COUNT = 4;

function childBlock(childNames) {
  if (childNames.length === 0) {
    return "";
  }
  return `\n${childNames.map((name) => `      <${name} />`).join("\n")}`;
}

function childImports(children) {
  return children.map(({ name, specifier }) => `import ${name} from '${specifier}'`).join("\n");
}

function variant0(id, children, iconSpecifier) {
  const names = children.map((child) => child.name);
  return `<template>
  <section :class="['panel', { 'panel--active': active }]" :data-node="nodeId">
    <header class="panel__head">
      <img class="panel__logo" src="@bench-assets/logo.svg" alt="" width="16" height="16" />
      <h3>{{ heading }}</h3>
      <button type="button" @click="toggle">{{ active ? 'Collapse' : 'Expand' }}</button>
    </header>
    <input v-model="query" class="panel__filter" :placeholder="'Filter ' + nodeId" />
    <ul v-if="active" class="panel__list">
      <li v-for="row in visibleRows" :key="row.id" :class="{ 'is-hot': row.score > threshold }">
        <span>{{ row.label }}</span>
        <strong>{{ formatScore(row.score) }}</strong>
      </li>
    </ul>
    <p v-else class="panel__empty">{{ summary }}</p>
    <div class="panel__children">${childBlock(names)}
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { formatScore, makeRows } from '@tools/ui'
import icon from '${iconSpecifier}'
${childImports(children)}

interface Row {
  id: string
  label: string
  score: number
}

const props = withDefaults(defineProps<{ threshold?: number; label?: string }>(), {
  threshold: 50,
  label: 'node-${id}',
})

const nodeId = 'node-${id}'
const active = ref(true)
const query = ref('')
const rows = ref<Row[]>(makeRows('${id}', 6))
const visibleRows = computed(() => rows.value.filter((row) => row.label.includes(query.value)))
const heading = computed(() => icon.name + ' ' + props.label)
const summary = computed(() => rows.value.length + ' rows / ' + props.threshold)
const threshold = computed(() => props.threshold)

function toggle(): void {
  active.value = !active.value
}
</script>

<style scoped>
.panel { display: grid; gap: 8px; padding: 12px; border: 1px solid #d4d4d8; }
.panel--active { border-color: #2563eb; }
.panel__head { display: flex; align-items: center; gap: 8px; }
.panel__list { margin: 0; padding-left: 16px; }
.panel__list .is-hot { color: #b91c1c; font-weight: 600; }
.panel__children { display: grid; gap: 8px; }
</style>
`;
}

function variant1(id, children, iconSpecifier) {
  const names = children.map((child) => child.name);
  return `<template>
  <article class="card" :style="{ '--card-weight': weight }">
    <img class="card__icon" :src="logoUrl" :alt="icon.name" width="20" height="20" />
    <h4>{{ title }}</h4>
    <p v-if="weight > 3">{{ detail }}</p>
    <button type="button" @click="bump">weight {{ weight }}</button>
    <div class="card__children">${childBlock(names)}
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { clampWeight } from '@tools/ui'
import icon from '${iconSpecifier}'
import logoUrl from '@bench-assets/logo.svg'
${childImports(children)}

const emit = defineEmits<{ (event: 'weight-changed', value: number): void }>()

const weight = ref(1)
const title = computed(() => 'card-${id} @ ' + icon.name)
const detail = computed(() => 'heavy card-${id} (' + weight.value + ')')

watch(weight, (next) => {
  emit('weight-changed', clampWeight(next))
})

function bump(): void {
  weight.value = clampWeight(weight.value + 1)
}
</script>

<style scoped>
.card { padding: 12px; border-radius: 6px; background: #fafafa; }
.card__icon { vertical-align: middle; }
.card__children { display: flex; flex-direction: column; gap: 6px; }
</style>
`;
}

function variant2(id, children, iconSpecifier) {
  const names = children.map((child) => child.name);
  return `<template>
  <div :class="$style.shell">
    <slot name="lead">
      <span v-once>{{ leadText }}</span>
    </slot>
    <ol v-show="expanded">
      <li v-for="(entry, index) in entries" :key="entry">{{ index }}: {{ entry }}</li>
    </ol>
    <button type="button" @click="expanded = !expanded">{{ expanded ? 'less' : 'more' }}</button>
    <slot />
    <div :class="$style.kids">${childBlock(names)}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, provide, ref } from 'vue'
import { listEntries } from '@tools/ui'
import icon from '${iconSpecifier}'
${childImports(children)}

const expanded = ref(false)
const entries = computed<string[]>(() => listEntries('${id}', 4))
const leadText = 'shell-${id} ' + icon.name

provide('bench-shell-${id}', { expanded, entries })
</script>

<style module>
.shell { display: grid; gap: 6px; padding: 10px; border-left: 3px solid #6366f1; }
.kids { display: grid; gap: 6px; }
</style>
`;
}

function variant3(id, children, iconSpecifier) {
  const names = children.map((child) => child.name);
  return `<template>
  <fieldset class="group">
    <legend>{{ legend }}</legend>
    <label>
      <input type="checkbox" v-model="enabled" />
      enabled
    </label>
    <div class="group__body" v-html="markup"></div>
    <template v-if="enabled">
      <div class="group__children">${childBlock(names)}
      </div>
    </template>
  </fieldset>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { escapeMarkup } from '@tools/ui'
import icon from '${iconSpecifier}'
${childImports(children)}

defineOptions({ name: 'BenchGroup${id}' })

const enabled = ref(true)
const legend = computed(() => 'group-${id} / ' + icon.name)
const markup = computed(() => escapeMarkup('<b>group-${id}</b>'))
</script>

<style scoped>
.group { border: 1px dashed #a1a1aa; padding: 10px; }
.group__body :deep(b) { color: #0f766e; }
.group__children { display: grid; gap: 6px; }
</style>
`;
}

const VARIANTS = [variant0, variant1, variant2, variant3];

/**
 * Render one SFC.
 *
 * @param index global component index, selects the variant
 * @param id zero-padded uniqueness token embedded in the body
 * @param children `{ name, specifier }` for each child component import
 * @param iconSpecifier bare specifier of the per-component vendor icon module
 */
export function renderComponent(index, id, children, iconSpecifier) {
  return VARIANTS[index % VARIANT_COUNT](id, children, iconSpecifier);
}

export { VARIANT_COUNT };
