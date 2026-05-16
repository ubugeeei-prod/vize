//! Benchmark for vize_patina linter.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vize_carton::append;
use vize_patina::rules::musea::MuseaLinter;
use vize_patina::rules::script::{NoInternalImports, PreferImportFromVue, ScriptLinter};
use vize_patina::Linter;

fn bench_lint_template(c: &mut Criterion) {
    let template = r#"
        <div class="container">
            <span v-if="show">{{ message }}</span>
            <button @click="handleClick">Click me</button>
            <ul>
                <li v-for="item in items" :key="item.id">{{ item.name }}</li>
            </ul>
        </div>
    "#;

    let linter = Linter::new();

    let mut group = c.benchmark_group("template");
    group.throughput(Throughput::Bytes(template.len() as u64));

    group.bench_function("lint_small", |b| {
        b.iter(|| linter.lint_template(black_box(template), "test.vue"))
    });

    group.finish();
}

fn bench_lint_large_template(c: &mut Criterion) {
    // Generate a larger template
    let mut template = String::from("<div>\n");
    for i in 0..100 {
        append!(
            template,
            r#"  <div v-if="show{i}">
    <span>{{ message{i} }}</span>
    <button @click="handle{i}">Button {i}</button>
  </div>
"#,
        );
    }
    template.push_str("</div>");

    let linter = Linter::new();

    let mut group = c.benchmark_group("template");
    group.throughput(Throughput::Bytes(template.len() as u64));

    group.bench_function("lint_large", |b| {
        b.iter(|| linter.lint_template(black_box(&template), "test.vue"))
    });

    group.finish();
}

fn bench_script_rules(c: &mut Criterion) {
    let script = r#"
import { ref, computed, onMounted } from 'vue'
import { useStore } from '@vue/reactivity'
import { h, createApp } from '@vue/runtime-dom'
import { something } from '@vue/shared'
import lodash from 'lodash'
import axios from 'axios'

const count = ref(0)
const doubled = computed(() => count.value * 2)

onMounted(() => {
  console.log('mounted')
})

export function setup() {
  return { count, doubled }
}
"#;

    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(PreferImportFromVue));
    linter.add_rule(Box::new(NoInternalImports));

    let mut group = c.benchmark_group("script");
    group.throughput(Throughput::Bytes(script.len() as u64));

    group.bench_function("lint", |b| b.iter(|| linter.lint(black_box(script), 0)));

    group.bench_function("has_vue_imports", |b| {
        b.iter(|| ScriptLinter::has_vue_imports(black_box(script)))
    });

    group.finish();
}

fn bench_musea_rules(c: &mut Criterion) {
    let art_file = r#"
<art title="Button Component" component="./Button.vue">
  <variant name="primary">
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="outlined">
    <Button variant="outlined">Outlined Button</Button>
  </variant>
  <variant name="disabled">
    <Button disabled>Disabled Button</Button>
  </variant>
  <variant name="with-icon">
    <Button><Icon name="check" /> With Icon</Button>
  </variant>
</art>
"#;

    let linter = MuseaLinter::new();

    let mut group = c.benchmark_group("musea");
    group.throughput(Throughput::Bytes(art_file.len() as u64));

    group.bench_function("lint", |b| b.iter(|| linter.lint(black_box(art_file))));

    group.finish();
}

criterion_group!(
    benches,
    bench_lint_template,
    bench_lint_large_template,
    bench_script_rules,
    bench_musea_rules
);
criterion_main!(benches);
