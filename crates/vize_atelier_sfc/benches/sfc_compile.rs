//! Native Rust benchmarks for SFC compilation performance.
//!
//! Run with: cargo bench -p vize_atelier_sfc --bench sfc_compile

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vize_atelier_sfc::{
    compile_sfc, parse_sfc, ScriptCompileOptions, SfcCompileOptions, SfcParseOptions,
    StyleCompileOptions, TemplateCompileOptions,
};

const SIMPLE_TEMPLATE_ONLY: &str = r#"<template>
  <section class="hero">
    <h1>{{ title }}</h1>
    <p>{{ subtitle }}</p>
  </section>
</template>

<style scoped>
.hero {
  padding: 24px;
}
</style>
"#;

const MEDIUM_SCRIPT_SETUP: &str = r#"<template>
  <div class="app-shell">
    <header class="topbar">
      <button @click="toggleMenu" class="menu-trigger">{{ menuLabel }}</button>
      <nav class="tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="{ active: tab.id === activeTab }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>
    </header>
    <main class="content">
      <article v-if="activePanel" class="panel">
        <h2>{{ activePanel.title }}</h2>
        <p>{{ activePanel.description }}</p>
        <ul>
          <li v-for="item in activePanel.items" :key="item.id">{{ item.label }}</li>
        </ul>
      </article>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type PanelItem = { id: number; label: string }
type Panel = { id: string; title: string; description: string; items: PanelItem[] }

const menuLabel = ref('Open menu')
const activeTab = ref('overview')
const tabs = ref([
  { id: 'overview', label: 'Overview' },
  { id: 'usage', label: 'Usage' },
  { id: 'api', label: 'API' },
])

const panels = ref<Panel[]>([
  {
    id: 'overview',
    title: 'Overview',
    description: 'A compact compile benchmark for script setup.',
    items: [
      { id: 1, label: 'Fast parsing' },
      { id: 2, label: 'Inline template output' },
    ],
  },
  {
    id: 'usage',
    title: 'Usage',
    description: 'Exercises v-for, v-if, and event handlers.',
    items: [
      { id: 3, label: 'Stateful tabs' },
      { id: 4, label: 'Computed panel lookup' },
    ],
  },
])

const activePanel = computed(() => panels.value.find((panel) => panel.id === activeTab.value))

function toggleMenu() {
  menuLabel.value = menuLabel.value === 'Open menu' ? 'Close menu' : 'Open menu'
}
</script>

<style scoped>
.app-shell {
  display: grid;
  gap: 16px;
}

.tabs {
  display: flex;
  gap: 8px;
}

.panel {
  border: 1px solid #ddd;
  padding: 16px;
}
</style>
"#;

const COMPLEX_SCRIPT_SETUP: &str = r#"<template>
  <section class="dashboard">
    <header class="dashboard-header">
      <div>
        <p class="eyebrow">{{ eyebrow }}</p>
        <h1>{{ title }}</h1>
      </div>
      <button @click="refresh" :disabled="loading" class="refresh">
        {{ loading ? 'Refreshing…' : 'Refresh' }}
      </button>
    </header>

    <div class="summary-grid">
      <article
        v-for="metric in metrics"
        :key="metric.id"
        class="metric-card"
        :style="{ borderColor: metric.color }"
      >
        <span class="metric-label">{{ metric.label }}</span>
        <strong class="metric-value">{{ metric.value }}</strong>
        <em class="metric-change">{{ metric.change }}</em>
      </article>
    </div>

    <section v-if="visibleProjects.length" class="projects">
      <article v-for="project in visibleProjects" :key="project.id" class="project-card">
        <header>
          <h2>{{ project.name }}</h2>
          <span>{{ project.owner }}</span>
        </header>
        <p>{{ project.summary }}</p>
        <ul>
          <li v-for="tag in project.tags" :key="tag">{{ tag }}</li>
        </ul>
      </article>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

interface Metric {
  id: number
  label: string
  value: string
  change: string
  color: string
}

interface Project {
  id: number
  name: string
  owner: string
  summary: string
  tags: string[]
  featured: boolean
}

const eyebrow = ref('Operations')
const title = ref('System overview')
const loading = ref(false)
const showFeaturedOnly = ref(false)

const metrics = ref<Metric[]>([
  { id: 1, label: 'Deployments', value: '128', change: '+12%', color: '#22c55e' },
  { id: 2, label: 'Latency', value: '84ms', change: '-9%', color: '#3b82f6' },
  { id: 3, label: 'Errors', value: '3', change: '-42%', color: '#ef4444' },
])

const projects = ref<Project[]>([
  {
    id: 1,
    name: 'Atlas',
    owner: 'Platform',
    summary: 'Core dashboard application',
    tags: ['dashboard', 'vue', 'rust'],
    featured: true,
  },
  {
    id: 2,
    name: 'Comet',
    owner: 'Growth',
    summary: 'Acquisition experiments and reporting',
    tags: ['growth', 'analytics'],
    featured: false,
  },
  {
    id: 3,
    name: 'Beacon',
    owner: 'Infra',
    summary: 'Realtime service health explorer',
    tags: ['infra', 'realtime'],
    featured: true,
  },
])

const visibleProjects = computed(() =>
  showFeaturedOnly.value
    ? projects.value.filter((project) => project.featured)
    : projects.value,
)

function refresh() {
  loading.value = !loading.value
}
</script>

<style scoped>
.dashboard {
  display: grid;
  gap: 20px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.metric-card,
.project-card {
  border: 1px solid #d4d4d8;
  border-radius: 16px;
  padding: 16px;
}
</style>
"#;

const PROP_HEAVY_LIST: &str = r#"<template>
  <section class="catalog">
    <ProductCard
      v-for="product in products"
      :key="product.id"
      class="catalog-card"
      style="--gap: 12px; --radius: 8px"
      :class="{ active: product.id === activeId, featured: product.featured }"
      :style="{ borderColor: product.color, opacity: product.disabled ? 0.5 : 1 }"
      :id="`product-${product.id}`"
      :title="product.title"
      :aria-label="product.title"
      :data-rank="product.rank"
      :data-state="product.disabled ? 'disabled' : 'ready'"
      @click="select(product)"
      @click.stop="track(product.id)"
      @keydown.enter="open(product)"
      @mouseenter="hovered = product.id"
      @mouseleave="hovered = null"
    >
      <template #meta="{ price, inventory }">
        <span>{{ price }}</span>
        <span>{{ inventory }}</span>
      </template>
    </ProductCard>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Product {
  id: number
  title: string
  rank: number
  color: string
  featured: boolean
  disabled: boolean
}

const activeId = ref(1)
const hovered = ref<number | null>(null)
const products = ref<Product[]>([
  { id: 1, title: 'Canvas', rank: 1, color: '#0ea5e9', featured: true, disabled: false },
  { id: 2, title: 'Pigment', rank: 2, color: '#f97316', featured: false, disabled: false },
  { id: 3, title: 'Frame', rank: 3, color: '#22c55e', featured: true, disabled: true },
])

function select(product: Product) {
  activeId.value = product.id
}

function track(id: number) {
  hovered.value = id
}

function open(product: Product) {
  activeId.value = product.id
}
</script>
"#;

fn compile_options(filename: &'static str) -> SfcCompileOptions {
    SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.into()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.into()),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn benchmark_compile_case(c: &mut Criterion, name: &str, filename: &'static str, source: &str) {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("failed to parse benchmark SFC");

    let source_len = source.len() as u64;
    let mut group = c.benchmark_group("sfc_compile");
    group.throughput(Throughput::Bytes(source_len));
    group.bench_function(name, |b| {
        b.iter(|| {
            let result = compile_sfc(black_box(&descriptor), black_box(compile_options(filename)))
                .expect("failed to compile benchmark SFC");
            black_box(result);
        });
    });
    group.finish();
}

fn bench_compile_template_only(c: &mut Criterion) {
    benchmark_compile_case(c, "template_only", "TemplateOnly.vue", SIMPLE_TEMPLATE_ONLY);
}

fn bench_compile_medium_script_setup(c: &mut Criterion) {
    benchmark_compile_case(
        c,
        "script_setup_medium",
        "MediumScriptSetup.vue",
        MEDIUM_SCRIPT_SETUP,
    );
}

fn bench_compile_complex_script_setup(c: &mut Criterion) {
    benchmark_compile_case(
        c,
        "script_setup_complex",
        "ComplexScriptSetup.vue",
        COMPLEX_SCRIPT_SETUP,
    );
}

fn bench_compile_prop_heavy_list(c: &mut Criterion) {
    benchmark_compile_case(c, "prop_heavy_list", "PropHeavyList.vue", PROP_HEAVY_LIST);
}

criterion_group!(
    benches,
    bench_compile_template_only,
    bench_compile_medium_script_setup,
    bench_compile_complex_script_setup,
    bench_compile_prop_heavy_list,
);
criterion_main!(benches);
