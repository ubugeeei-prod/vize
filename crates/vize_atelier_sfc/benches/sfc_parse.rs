//! Native Rust benchmarks for SFC parsing performance.
//!
//! Run with: cargo bench -p vize_atelier_sfc

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

const SIMPLE_SFC: &str = r#"<template>
  <div class="container">
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const title = ref('Hello')
const message = ref('World')
</script>

<style scoped>
.container {
  padding: 20px;
}
</style>
"#;

const MEDIUM_SFC: &str = r#"<template>
  <div class="app">
    <header class="header">
      <nav class="nav">
        <ul class="nav-list">
          <li v-for="item in navItems" :key="item.id" class="nav-item">
            <a :href="item.href" class="nav-link">{{ item.label }}</a>
          </li>
        </ul>
      </nav>
    </header>
    <main class="main">
      <section class="hero">
        <h1 class="hero-title">{{ heroTitle }}</h1>
        <p class="hero-description">{{ heroDescription }}</p>
        <button @click="handleClick" class="cta-button">
          {{ ctaText }}
        </button>
      </section>
      <section class="features">
        <div v-for="feature in features" :key="feature.id" class="feature-card">
          <img :src="feature.icon" :alt="feature.title" class="feature-icon" />
          <h3 class="feature-title">{{ feature.title }}</h3>
          <p class="feature-description">{{ feature.description }}</p>
        </div>
      </section>
    </main>
    <footer class="footer">
      <p>&copy; 2024 My App. All rights reserved.</p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { NavItem, Feature } from './types'

interface Props {
  initialTitle?: string
  showFeatures?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  initialTitle: 'Welcome',
  showFeatures: true,
})

const emit = defineEmits<{
  (e: 'click', value: string): void
  (e: 'update', payload: { id: number; data: unknown }): void
}>()

const heroTitle = ref(props.initialTitle)
const heroDescription = ref('Build amazing applications with Vue.js')
const ctaText = ref('Get Started')

const navItems = ref<NavItem[]>([
  { id: 1, href: '/', label: 'Home' },
  { id: 2, href: '/about', label: 'About' },
  { id: 3, href: '/contact', label: 'Contact' },
])

const features = ref<Feature[]>([
  { id: 1, icon: '/icons/fast.svg', title: 'Fast', description: 'Lightning fast performance' },
  { id: 2, icon: '/icons/easy.svg', title: 'Easy', description: 'Simple and intuitive API' },
  { id: 3, icon: '/icons/powerful.svg', title: 'Powerful', description: 'Feature-rich framework' },
])

const featureCount = computed(() => features.value.length)

function handleClick() {
  emit('click', 'cta-clicked')
}

watch(heroTitle, (newVal, oldVal) => {
  console.log(`Title changed from ${oldVal} to ${newVal}`)
})

onMounted(() => {
  console.log('Component mounted')
})
</script>

<style scoped>
.app {
  font-family: 'Inter', sans-serif;
  color: #333;
}

.header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 1rem 2rem;
}

.nav-list {
  display: flex;
  gap: 2rem;
  list-style: none;
  margin: 0;
  padding: 0;
}

.nav-link {
  color: white;
  text-decoration: none;
  font-weight: 500;
}

.nav-link:hover {
  opacity: 0.8;
}

.hero {
  text-align: center;
  padding: 4rem 2rem;
  background: #f8f9fa;
}

.hero-title {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.hero-description {
  font-size: 1.25rem;
  color: #666;
  margin-bottom: 2rem;
}

.cta-button {
  background: #667eea;
  color: white;
  border: none;
  padding: 1rem 2rem;
  font-size: 1rem;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: transform 0.2s;
}

.cta-button:hover {
  transform: translateY(-2px);
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  padding: 4rem 2rem;
}

.feature-card {
  background: white;
  padding: 2rem;
  border-radius: 1rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  text-align: center;
}

.feature-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 1rem;
}

.feature-title {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
}

.feature-description {
  color: #666;
}

.footer {
  background: #333;
  color: white;
  text-align: center;
  padding: 2rem;
}
</style>
"#;

const COMPLEX_SFC: &str = r#"<template>
  <Teleport to="body">
    <Transition name="modal" appear>
      <div v-if="isOpen" class="modal-overlay" @click.self="close">
        <div class="modal-container" role="dialog" aria-modal="true">
          <header class="modal-header">
            <slot name="header">
              <h2>{{ title }}</h2>
            </slot>
            <button class="close-btn" @click="close" aria-label="Close">
              <svg viewBox="0 0 24 24"><path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/></svg>
            </button>
          </header>
          <main class="modal-body">
            <slot>
              <p>{{ content }}</p>
            </slot>
          </main>
          <footer class="modal-footer">
            <slot name="footer">
              <button class="btn btn-secondary" @click="close">Cancel</button>
              <button class="btn btn-primary" @click="confirm" :disabled="loading">
                <span v-if="loading" class="spinner"></span>
                {{ confirmText }}
              </button>
            </slot>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useEventListener, useFocusTrap, useScrollLock } from '@vueuse/core'

interface Props {
  modelValue?: boolean
  title?: string
  content?: string
  confirmText?: string
  persistent?: boolean
  maxWidth?: string
  zIndex?: number
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  title: 'Dialog',
  content: '',
  confirmText: 'Confirm',
  persistent: false,
  maxWidth: '500px',
  zIndex: 1000,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'confirm'): void
  (e: 'cancel'): void
  (e: 'opened'): void
  (e: 'closed'): void
}>()

const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
})

const loading = ref(false)
const containerRef = ref<HTMLElement | null>(null)

const { activate: activateTrap, deactivate: deactivateTrap } = useFocusTrap(containerRef, {
  immediate: false,
  allowOutsideClick: true,
})

const { lock: lockScroll, unlock: unlockScroll } = useScrollLock()

function close() {
  if (props.persistent && loading.value) return
  isOpen.value = false
  emit('cancel')
}

async function confirm() {
  loading.value = true
  try {
    emit('confirm')
    await nextTick()
    isOpen.value = false
  } finally {
    loading.value = false
  }
}

function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value && !props.persistent) {
    close()
  }
}

watch(isOpen, async (newValue) => {
  if (newValue) {
    await nextTick()
    lockScroll()
    activateTrap()
    emit('opened')
  } else {
    deactivateTrap()
    unlockScroll()
    emit('closed')
  }
})

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
  unlockScroll()
})
</script>

<script lang="ts">
export default {
  name: 'Modal',
  inheritAttrs: false,
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: v-bind(zIndex);
}

.modal-container {
  background: white;
  border-radius: 12px;
  max-width: v-bind(maxWidth);
  width: 90%;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  padding: 0.5rem;
  cursor: pointer;
  border-radius: 50%;
  transition: background 0.2s;
}

.close-btn:hover {
  background: #f3f4f6;
}

.close-btn svg {
  width: 20px;
  height: 20px;
  fill: currentColor;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
  padding: 1.5rem;
  border-top: 1px solid #e5e7eb;
}

.btn {
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-secondary {
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  color: #374151;
}

.btn-secondary:hover {
  background: #e5e7eb;
}

.btn-primary {
  background: #3b82f6;
  border: none;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container {
  transform: scale(0.9) translateY(-20px);
}

.modal-leave-to .modal-container {
  transform: scale(0.9) translateY(20px);
}
</style>
"#;

fn bench_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("sfc_parse");
    group.throughput(Throughput::Bytes(SIMPLE_SFC.len() as u64));

    group.bench_function("simple", |b| {
        b.iter(|| {
            let options = SfcParseOptions::default();
            parse_sfc(black_box(SIMPLE_SFC), options).unwrap()
        })
    });

    group.finish();
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("sfc_parse");
    group.throughput(Throughput::Bytes(MEDIUM_SFC.len() as u64));

    group.bench_function("medium", |b| {
        b.iter(|| {
            let options = SfcParseOptions::default();
            parse_sfc(black_box(MEDIUM_SFC), options).unwrap()
        })
    });

    group.finish();
}

fn bench_parse_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("sfc_parse");
    group.throughput(Throughput::Bytes(COMPLEX_SFC.len() as u64));

    group.bench_function("complex", |b| {
        b.iter(|| {
            let options = SfcParseOptions::default();
            parse_sfc(black_box(COMPLEX_SFC), options).unwrap()
        })
    });

    group.finish();
}

fn bench_parse_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("sfc_parse_throughput");

    // Combine all SFCs for throughput testing
    let all_sources = [SIMPLE_SFC, MEDIUM_SFC, COMPLEX_SFC];
    let total_bytes: usize = all_sources.iter().map(|s| s.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("all_sizes", |b| {
        b.iter(|| {
            for source in &all_sources {
                let options = SfcParseOptions::default();
                parse_sfc(black_box(*source), options).unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_medium,
    bench_parse_complex,
    bench_parse_throughput
);
criterion_main!(benches);
