//! Allocation budget for `parse_art` on the S0/S1 reader (Davinci P4-13).
//!
//! Each case measures one whole `parse_art` call — the SFC split, the S1
//! tree of the `<art>` block, the variants and the `defineArt()` reader —
//! over a pinned input shaped like the Musea benchmark corpus
//! (`tools/benchmarks/scripts/musea-corpus.mjs`) and the in-repo gallery.
//! `budgets.toml [bench]` pins the exact `allocs` of each, so a change that
//! re-introduces work (the full script-setup analysis for `defineArt()`, the
//! per-style `v-bind()` scan, an owned copy per block) fails the CI
//! allocation gate; wall time stays report-only until the reference runner
//! records it.

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_musea::{Allocator, ArtParseOptions, parse_art};
use vize_s0::cstr;

/// The corpus's majority shape: `defineArt()` metadata, three variants, two
/// style blocks (file 1 of the Musea benchmark corpus).
const DEFINE_ART: &str = r#"<script setup lang="ts">
import { computed, ref } from "vue";
import BenchComponent1 from "./BenchComponent1.vue";
import {
  benchPrimaryTokens,
  benchSurfaceTokens,
  benchToneFor,
} from "./bench-tokens";
import "./styles/bench-tokens.css";

defineArt("./BenchComponent1.vue", {
  title: "BenchComponent1",
  description: "Generated art file 1 for the Musea benchmark lane.",
  category: "Bench",
  status: "draft",
  tags: ["bench", "musea", "variant-1"],
  order: 1,
});

const { label, tone } = benchPrimaryTokens;
const [firstSurface, secondSurface] = benchSurfaceTokens;
const pressed = ref(false);
const caption = computed(() => `${label} / ${tone} / 1`);

function toggle(): void {
  pressed.value = !pressed.value;
}
</script>

<art>
  <variant name="Default" default>
    <BenchComponent1 :pressed="pressed" :tone="benchToneFor(0)" :surface="firstSurface" @click="toggle">
      {{ caption }}
    </BenchComponent1>
  </variant>

  <variant name="Pressed">
    <BenchComponent1 :pressed="pressed" :tone="benchToneFor(1)" :surface="secondSurface" @click="toggle">
      {{ caption }}
    </BenchComponent1>
  </variant>

  <variant name="Muted" skip-vrt>
    <BenchComponent1 :pressed="pressed" :tone="benchToneFor(2)" :surface="firstSurface" @click="toggle">
      {{ caption }}
    </BenchComponent1>
  </variant>
</art>

<style scoped>
.bench-component-1 {
  color: var(--bench-primary);
  padding: 1px 2px;
  border-radius: 1px;
}
</style>

<style lang="scss" scoped>
.bench-component-1 {
  &:hover {
    color: var(--bench-accent);
  }
}
</style>
"#;

/// The legacy shape: metadata on the `<art>` open tag, `args` / `viewport`
/// variants, and no `defineArt()` call.
const LEGACY_ATTRS: &str = r#"<script setup lang="ts">
import Card from "./Card.vue";
</script>

<art title="Card" component="./Card.vue" category="molecules" tags="layout,content" status="ready" order="10">
  <variant name="Default" default>
    <Card><h3>Card Title</h3><p>Body</p></Card>
  </variant>
  <variant name="With Image" args='{"image":"/placeholder.jpg","imageAlt":"Placeholder"}'>
    <Card :image="args.image" :image-alt="args.imageAlt"><p>Featured</p></Card>
  </variant>
  <variant name="Mobile" viewport="375x667@2">
    <Card><p>Mobile</p></Card>
  </variant>
</art>

<style scoped>
.card { padding: 20px; }
</style>
"#;

/// An inline `<art>` block in a real gallery component.
const INLINE: &str = include_str!("../../../examples/vite-musea/src/components/Badge.vue");

fn bench_case(criterion: &mut Criterion, id: &str, fixture: &str, source: &'static str) {
    bench_stage_with_metrics(criterion, id, fixture, |window| {
        let allocator = Allocator::new();
        window.measure(|| {
            let art = parse_art(&allocator, source, ArtParseOptions::default())
                .expect("the fixture is a valid Art file");
            (
                art.variants.len(),
                art.styles.len(),
                art.metadata.tags.len(),
            )
        })
    });
}

fn davinci_art(criterion: &mut Criterion) {
    bench_case(
        criterion,
        &cstr!("musea_parse_art_define_art"),
        "synthetic:musea-corpus-define-art",
        DEFINE_ART,
    );
    bench_case(
        criterion,
        &cstr!("musea_parse_art_legacy_attrs"),
        "synthetic:musea-legacy-attrs",
        LEGACY_ATTRS,
    );
    bench_case(
        criterion,
        &cstr!("musea_parse_art_inline"),
        "examples/vite-musea/src/components/Badge.vue",
        INLINE,
    );
}

criterion_group!(davinci_art_group, davinci_art);
davinci_harness::main!(davinci_art_group);
