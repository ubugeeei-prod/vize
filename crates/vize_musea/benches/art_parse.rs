//! Benchmarks for Art file parsing performance.
//!
//! Run with: cargo bench -p vize_musea
//!
//! These benchmarks measure the performance of the arena-allocated,
//! zero-copy parser for Art files.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vize_carton::append;
use vize_carton::Bump;
use vize_musea::{parse_art, transform_to_csf, transform_to_vue, ArtParseOptions};

// =============================================================================
// Test Data
// =============================================================================

/// Minimal Art file - single variant, no metadata
const SIMPLE_ART: &str = r#"<art title="Button">
  <variant name="Default">
    <Button>Click me</Button>
  </variant>
</art>
"#;

/// Medium Art file - multiple variants with metadata
const MEDIUM_ART: &str = r#"<art title="Button" description="A versatile button component" component="./Button.vue" category="atoms" tags="ui,input,interactive">
  <variant name="Primary" default>
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="Outline">
    <Button variant="outline">Outline Button</Button>
  </variant>
  <variant name="Ghost">
    <Button variant="ghost">Ghost Button</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Disabled Button</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>

<style scoped>
.art-container {
  padding: 20px;
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}
</style>
"#;

/// Complex Art file - many variants with args, viewport, skip-vrt
const COMPLEX_ART: &str = r#"<art title="Card" description="A flexible card component with multiple layout options" component="./Card.vue" category="molecules" tags="layout,container,content,interactive" status="ready" order="10">
  <variant name="Default" default>
    <Card>
      <template #header>
        <h3>Card Title</h3>
      </template>
      <p>Card content goes here. This is a simple card with default styling.</p>
      <template #footer>
        <Button>Action</Button>
      </template>
    </Card>
  </variant>
  <variant name="With Image" args='{"image":"/placeholder.jpg","imageAlt":"Placeholder"}'>
    <Card :image="args.image" :image-alt="args.imageAlt">
      <template #header>
        <h3>Featured Card</h3>
      </template>
      <p>A card with a featured image at the top.</p>
    </Card>
  </variant>
  <variant name="Horizontal" args='{"layout":"horizontal"}'>
    <Card :layout="args.layout">
      <template #media>
        <img src="/thumbnail.jpg" alt="Thumbnail" />
      </template>
      <h3>Horizontal Layout</h3>
      <p>Content appears beside the media in horizontal layout.</p>
    </Card>
  </variant>
  <variant name="Interactive" args='{"clickable":true,"hoverable":true}'>
    <Card :clickable="args.clickable" :hoverable="args.hoverable" @click="handleClick">
      <h3>Interactive Card</h3>
      <p>Click or hover to see the interaction effects.</p>
    </Card>
  </variant>
  <variant name="Loading" args='{"loading":true}'>
    <Card :loading="args.loading">
      <h3>Loading State</h3>
      <p>Shows skeleton loading animation.</p>
    </Card>
  </variant>
  <variant name="Mobile View" viewport="375x667">
    <Card>
      <h3>Mobile Optimized</h3>
      <p>This variant shows how the card looks on mobile devices.</p>
    </Card>
  </variant>
  <variant name="Tablet View" viewport="768x1024">
    <Card>
      <h3>Tablet View</h3>
      <p>Optimized layout for tablet-sized screens.</p>
    </Card>
  </variant>
  <variant name="High DPI" viewport="375x667@2">
    <Card>
      <h3>Retina Display</h3>
      <p>Testing on high DPI displays.</p>
    </Card>
  </variant>
  <variant name="Dark Theme" skip-vrt>
    <Card class="dark-theme">
      <h3>Dark Theme</h3>
      <p>Card with dark theme styling. Skipped in VRT due to theme variations.</p>
    </Card>
  </variant>
  <variant name="Custom Styling" skip-vrt args='{"borderRadius":"16px","shadow":"xl"}'>
    <Card :style="{ borderRadius: args.borderRadius }" :shadow="args.shadow">
      <h3>Custom Styled</h3>
      <p>Demonstrates custom styling options.</p>
    </Card>
  </variant>
</art>

<script setup lang="ts">
import { ref } from 'vue'
import Card from './Card.vue'
import Button from '../Button/Button.vue'

const handleClick = () => {
  console.log('Card clicked')
}
</script>

<style scoped>
.art-container {
  padding: 24px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
  background: #f5f5f5;
}

.dark-theme {
  --card-bg: #1a1a1a;
  --card-text: #ffffff;
  --card-border: #333333;
}
</style>

<style>
/* Global styles for the gallery */
.musea-variant {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}
</style>
"#;

/// Massive Art file for stress testing - 50 variants
fn generate_massive_art() -> String {
    let mut art = String::from(
        r#"<art title="Stress Test" description="Performance stress test with many variants" component="./Component.vue" category="test" tags="stress,performance,benchmark">"#,
    );
    art.push('\n');

    for i in 0..50 {
        append!(
            art,
            r#"  <variant name="Variant {}" {}args='{{"index":{}}}'>
    <Component :index="args.index">Content for variant {}</Component>
  </variant>
"#,
            i,
            if i == 0 { "default " } else { "" },
            i,
            i
        );
    }

    art.push_str("</art>\n");

    art.push_str(
        r#"
<script setup lang="ts">
import Component from './Component.vue'
</script>
"#,
    );

    art
}

// =============================================================================
// Benchmarks
// =============================================================================

fn bench_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_parse");
    group.throughput(Throughput::Bytes(SIMPLE_ART.len() as u64));

    group.bench_function("simple", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor = parse_art(
                &allocator,
                black_box(SIMPLE_ART),
                ArtParseOptions::default(),
            )
            .unwrap();
            // Consume to prevent optimization
            black_box(descriptor.metadata.title);
        })
    });

    group.finish();
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_parse");
    group.throughput(Throughput::Bytes(MEDIUM_ART.len() as u64));

    group.bench_function("medium", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor = parse_art(
                &allocator,
                black_box(MEDIUM_ART),
                ArtParseOptions::default(),
            )
            .unwrap();
            black_box(descriptor.variants.len());
        })
    });

    group.finish();
}

fn bench_parse_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_parse");
    group.throughput(Throughput::Bytes(COMPLEX_ART.len() as u64));

    group.bench_function("complex", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor = parse_art(
                &allocator,
                black_box(COMPLEX_ART),
                ArtParseOptions::default(),
            )
            .unwrap();
            black_box(descriptor.variants.len());
        })
    });

    group.finish();
}

fn bench_parse_massive(c: &mut Criterion) {
    let massive = generate_massive_art();
    let mut group = c.benchmark_group("art_parse");
    group.throughput(Throughput::Bytes(massive.len() as u64));

    group.bench_function("massive_50_variants", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor =
                parse_art(&allocator, black_box(&massive), ArtParseOptions::default()).unwrap();
            black_box(descriptor.variants.len());
        })
    });

    group.finish();
}

fn bench_arena_reuse(c: &mut Criterion) {
    let sources = [SIMPLE_ART, MEDIUM_ART, COMPLEX_ART];
    let total_bytes: usize = sources.iter().map(|s| s.len()).sum();

    let mut group = c.benchmark_group("art_parse_arena");
    group.throughput(Throughput::Bytes(total_bytes as u64));

    // Benchmark with fresh allocator per parse
    group.bench_function("fresh_allocator", |b| {
        b.iter(|| {
            for source in &sources {
                let allocator = Bump::new();
                parse_art(&allocator, black_box(*source), ArtParseOptions::default()).unwrap();
            }
        })
    });

    // Benchmark with shared allocator (reset between parses)
    group.bench_function("shared_allocator", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            for source in &sources {
                parse_art(&allocator, black_box(*source), ArtParseOptions::default()).unwrap();
                // Note: In real usage, you'd reset or let descriptors go out of scope
            }
        })
    });

    group.finish();
}

fn bench_transform_csf(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_transform");

    // Parse once outside the benchmark
    let allocator = Bump::new();
    let descriptor = parse_art(&allocator, COMPLEX_ART, ArtParseOptions::default()).unwrap();

    group.bench_function("to_csf", |b| {
        b.iter(|| transform_to_csf(black_box(&descriptor)))
    });

    group.finish();
}

fn bench_transform_vue(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_transform");

    // Parse once outside the benchmark
    let allocator = Bump::new();
    let descriptor = parse_art(&allocator, COMPLEX_ART, ArtParseOptions::default()).unwrap();

    group.bench_function("to_vue", |b| {
        b.iter(|| transform_to_vue(black_box(&descriptor)))
    });

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("art_full_pipeline");
    group.throughput(Throughput::Bytes(COMPLEX_ART.len() as u64));

    group.bench_function("parse_and_transform_csf", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor = parse_art(
                &allocator,
                black_box(COMPLEX_ART),
                ArtParseOptions::default(),
            )
            .unwrap();
            transform_to_csf(&descriptor)
        })
    });

    group.bench_function("parse_and_transform_vue", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            let descriptor = parse_art(
                &allocator,
                black_box(COMPLEX_ART),
                ArtParseOptions::default(),
            )
            .unwrap();
            transform_to_vue(&descriptor)
        })
    });

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    // Generate 100 art files for throughput testing
    let arts: Vec<String> = (0..100)
        .map(|i| {
            format!(
                r#"<art title="Component{}" component="./Component{}.vue">
  <variant name="Default" default>
    <Component{}>Content</Component{}>
  </variant>
  <variant name="Alt">
    <Component{} variant="alt">Alt</Component{}>
  </variant>
</art>
"#,
                i, i, i, i, i, i
            )
        })
        .collect();

    let total_bytes: usize = arts.iter().map(|s| s.len()).sum();

    let mut group = c.benchmark_group("art_throughput");
    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.throughput(Throughput::Elements(100));

    group.bench_function("100_files", |b| {
        b.iter(|| {
            let allocator = Bump::new();
            for art in &arts {
                parse_art(&allocator, black_box(art), ArtParseOptions::default()).unwrap();
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
    bench_parse_massive,
    bench_arena_reuse,
    bench_transform_csf,
    bench_transform_vue,
    bench_full_pipeline,
    bench_throughput,
);
criterion_main!(benches);
