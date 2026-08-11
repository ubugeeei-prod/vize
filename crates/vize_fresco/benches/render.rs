//! Render benchmarks.
#![allow(deprecated)]

use std::io;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

use vize_fresco::component::{VirtualListNavigation, VirtualListState};
use vize_fresco::layout::{FlexStyle, LayoutEngine};
use vize_fresco::terminal::{Backend, Buffer, Style};
use vize_fresco::text::{TextWidth, TextWrap, WrapMode};

fn benchmark_buffer_set_string(c: &mut Criterion) {
    let mut buffer = Buffer::new(80, 24);
    let style = Style::default();

    c.bench_function("buffer_set_string_ascii", |b| {
        b.iter(|| {
            buffer.set_string(0, 0, black_box("Hello, World!"), style);
        });
    });

    c.bench_function("buffer_set_string_cjk", |b| {
        b.iter(|| {
            buffer.set_string(0, 0, black_box("こんにちは世界"), style);
        });
    });
}

fn benchmark_text_width(c: &mut Criterion) {
    c.bench_function("text_width_ascii", |b| {
        b.iter(|| TextWidth::width(black_box("Hello, World! This is a test string.")));
    });

    c.bench_function("text_width_cjk", |b| {
        b.iter(|| TextWidth::width(black_box("こんにちは世界！これはテスト文字列です。")));
    });

    c.bench_function("text_width_mixed", |b| {
        b.iter(|| TextWidth::width(black_box("Hello 世界! Mixed テスト string.")));
    });
}

fn benchmark_text_wrap(c: &mut Criterion) {
    let long_text = "This is a long piece of text that needs to be wrapped to fit within a certain width. It contains multiple sentences and should demonstrate the wrapping algorithm's performance.";

    c.bench_function("text_wrap_word", |b| {
        b.iter(|| TextWrap::wrap(black_box(long_text), 40, WrapMode::Word));
    });

    c.bench_function("text_wrap_char", |b| {
        b.iter(|| TextWrap::wrap(black_box(long_text), 40, WrapMode::Char));
    });

    let japanese_text = "これは長いテキストで、ある幅に収まるように折り返す必要があります。複数の文を含み、折り返しアルゴリズムの性能を示すはずです。";

    c.bench_function("text_wrap_cjk", |b| {
        b.iter(|| TextWrap::wrap(black_box(japanese_text), 40, WrapMode::Char));
    });
}

fn benchmark_layout(c: &mut Criterion) {
    c.bench_function("layout_simple", |b| {
        b.iter(|| {
            let mut engine = LayoutEngine::new();

            let root_style = FlexStyle::default();
            let root = engine.new_node(&root_style);
            engine.set_root(root);

            for _ in 0..10 {
                let child = engine.new_node(&root_style);
                engine.add_child(root, child);
            }

            engine.compute(80.0, 24.0);
        });
    });

    c.bench_function("layout_deep", |b| {
        b.iter(|| {
            let mut engine = LayoutEngine::new();

            let style = FlexStyle::default();
            let root = engine.new_node(&style);
            engine.set_root(root);

            let mut parent = root;
            for _ in 0..10 {
                let child = engine.new_node(&style);
                engine.add_child(parent, child);
                parent = child;
            }

            engine.compute(80.0, 24.0);
        });
    });
}

fn benchmark_buffer_diff(c: &mut Criterion) {
    let mut buf1 = Buffer::new(80, 24);
    let mut buf2 = Buffer::new(80, 24);
    let style = Style::default();

    // Fill with some content
    for y in 0..24 {
        buf1.set_string(0, y, "Hello, World! This is line content.", style);
        buf2.set_string(0, y, "Hello, World! This is line content.", style);
    }

    // Make some changes
    buf2.set_string(0, 5, "Changed line here!", style);
    buf2.set_string(0, 10, "Another changed line!", style);

    c.bench_function("buffer_diff", |b| {
        b.iter(|| {
            let diffs: Vec<_> = buf1.diff(&buf2).collect();
            black_box(diffs);
        });
    });
}

fn benchmark_virtual_list(c: &mut Criterion) {
    let keys: Vec<u64> = (0..10_000).collect();
    let mut group = c.benchmark_group("virtual_list_10k");
    group.throughput(Throughput::Elements(keys.len() as u64));

    let mut reconciliation = VirtualListState::with_overscan(20, 3);
    let _ = reconciliation.reconcile(&keys);
    let _ = reconciliation.select_index(&keys, 9_000);
    group.bench_function("reconcile", |b| {
        b.iter(|| black_box(reconciliation.reconcile(black_box(&keys))));
    });

    group.throughput(Throughput::Elements(1));
    let mut navigation = VirtualListState::with_overscan(20, 3);
    let _ = navigation.reconcile(&keys);
    group.bench_function("navigation", |b| {
        b.iter(|| {
            if navigation.selected_index() == Some(keys.len() - 1) {
                let _ = navigation.navigate(&keys, VirtualListNavigation::First);
            }
            black_box(navigation.navigate(black_box(&keys), VirtualListNavigation::Next))
        });
    });

    group.bench_function("window", |b| {
        b.iter(|| black_box(navigation.window()));
    });
    group.finish();
}

fn benchmark_injected_backend_output(c: &mut Criterion) {
    let mut backend = Backend::with_writer(120, 40, io::sink());
    backend.buffer_mut().set_string(0, 0, "A", Style::new());
    backend.flush().unwrap();
    let mut alternate = false;

    c.bench_function("terminal_output/one_changed_cell", |b| {
        b.iter(|| {
            alternate = !alternate;
            backend
                .buffer_mut()
                .set_string(0, 0, if alternate { "B" } else { "A" }, Style::new());
            black_box(backend.flush_measured().unwrap())
        });
    });
}

criterion_group!(
    benches,
    benchmark_buffer_set_string,
    benchmark_text_width,
    benchmark_text_wrap,
    benchmark_layout,
    benchmark_buffer_diff,
    benchmark_virtual_list,
    benchmark_injected_backend_output,
);
criterion_main!(benches);
