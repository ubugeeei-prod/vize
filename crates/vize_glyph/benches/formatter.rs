//! Benchmarks for vize_glyph formatter

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use vize_glyph::{
    Allocator, FormatOptions, format_script, format_sfc, format_sfc_with_allocator, format_template,
};

const SIMPLE_SFC: &str = r#"<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)

function increment() {
  count.value++
}
</script>

<template>
  <div class="container">
    <h1>{{ count }}</h1>
    <p>Doubled: {{ doubled }}</p>
    <button @click="increment">+1</button>
  </div>
</template>

<style scoped>
.container {
  padding: 20px;
  background: #f0f0f0;
}

h1 {
  color: #333;
}
</style>
"#;

const LARGE_SCRIPT: &str = r#"
import { ref, computed, watch, onMounted, onUnmounted, reactive, toRefs, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useStore } from 'vuex'

const router = useRouter()
const route = useRoute()
const store = useStore()

const state = reactive({
  count: 0,
  name: 'Vue',
  items: [],
  loading: false,
  error: null,
})

const { count, name, items, loading, error } = toRefs(state)

const doubled = computed(() => count.value * 2)
const tripled = computed(() => count.value * 3)
const quadrupled = computed(() => count.value * 4)

function increment() {
  count.value++
}

function decrement() {
  count.value--
}

async function fetchData() {
  loading.value = true
  try {
    const response = await fetch('/api/data')
    items.value = await response.json()
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}

watch(count, (newVal, oldVal) => {
  console.log(`Count changed from ${oldVal} to ${newVal}`)
})

onMounted(() => {
  fetchData()
})

onUnmounted(() => {
  console.log('Component unmounted')
})
"#;

const COMPLEX_TEMPLATE: &str = r#"
<div class="app-container" id="main-app" data-testid="app">
  <header class="header" :class="{ 'header--sticky': isSticky }">
    <nav class="nav">
      <ul class="nav-list">
        <li v-for="item in navItems" :key="item.id" class="nav-item">
          <router-link :to="item.path" class="nav-link">{{ item.label }}</router-link>
        </li>
      </ul>
    </nav>
  </header>
  <main class="main-content">
    <section v-if="loading" class="loading-section">
      <div class="spinner"></div>
      <p>Loading...</p>
    </section>
    <section v-else-if="error" class="error-section">
      <p class="error-message">{{ error }}</p>
      <button @click="retry" class="btn btn-primary">Retry</button>
    </section>
    <section v-else class="content-section">
      <article v-for="post in posts" :key="post.id" class="post-card">
        <h2 class="post-title">{{ post.title }}</h2>
        <p class="post-excerpt">{{ post.excerpt }}</p>
        <footer class="post-footer">
          <span class="post-date">{{ formatDate(post.date) }}</span>
          <router-link :to="`/posts/${post.id}`" class="read-more">Read more</router-link>
        </footer>
      </article>
    </section>
  </main>
  <footer class="footer">
    <p>&copy; 2024 My App</p>
  </footer>
</div>
"#;

fn benchmark_format_sfc(c: &mut Criterion) {
    let options = FormatOptions::default();

    let mut group = c.benchmark_group("format_sfc");
    group.throughput(Throughput::Bytes(SIMPLE_SFC.len() as u64));

    group.bench_function("simple_sfc", |b| {
        b.iter(|| format_sfc(black_box(SIMPLE_SFC), black_box(&options)).unwrap())
    });

    group.finish();
}

fn benchmark_format_sfc_with_allocator(c: &mut Criterion) {
    let options = FormatOptions::default();
    let allocator = Allocator::with_capacity(8192);

    let mut group = c.benchmark_group("format_sfc_with_allocator");
    group.throughput(Throughput::Bytes(SIMPLE_SFC.len() as u64));

    group.bench_function("simple_sfc_reuse_allocator", |b| {
        b.iter(|| {
            format_sfc_with_allocator(
                black_box(SIMPLE_SFC),
                black_box(&options),
                black_box(&allocator),
            )
            .unwrap()
        })
    });

    group.finish();
}

fn benchmark_format_script(c: &mut Criterion) {
    let options = FormatOptions::default();

    let mut group = c.benchmark_group("format_script");
    group.throughput(Throughput::Bytes(LARGE_SCRIPT.len() as u64));

    group.bench_function("large_script", |b| {
        b.iter(|| format_script(black_box(LARGE_SCRIPT), black_box(&options)).unwrap())
    });

    group.finish();
}

fn benchmark_format_template(c: &mut Criterion) {
    let options = FormatOptions::default();

    let mut group = c.benchmark_group("format_template");
    group.throughput(Throughput::Bytes(COMPLEX_TEMPLATE.len() as u64));

    group.bench_function("complex_template", |b| {
        b.iter(|| format_template(black_box(COMPLEX_TEMPLATE), black_box(&options)).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_format_sfc,
    benchmark_format_sfc_with_allocator,
    benchmark_format_script,
    benchmark_format_template,
);

criterion_main!(benches);
