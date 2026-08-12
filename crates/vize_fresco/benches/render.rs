//! Render benchmarks.
#![allow(deprecated)]

use std::io;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};

use vize_fresco::component::{
    BoxNode, DiagnosticPresentation, DiagnosticPresentationKind, DiagnosticPresentationProfile,
    DiagnosticTone, DiagnosticWorkspaceKeymap, DiagnosticWorkspaceState, TextNode,
    VirtualListNavigation, VirtualListState,
};
use vize_fresco::headless::{
    HeadlessPresentation, HeadlessRenderer, HeadlessSemanticNode, SemanticRole,
};
use vize_fresco::input::{Key, KeyEvent};
use vize_fresco::layout::{Dimension, FlexStyle, LayoutEngine};
use vize_fresco::render::{FrameCoalescer, FrameRenderer, RenderTree};
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

fn benchmark_buffer_ascii_line(c: &mut Criterion) {
    const ASCII_LINE: &str =
        "01234567890123456789012345678901234567890123456789012345678901234567890123456789";
    let mut buffer = Buffer::new(80, 1);
    let mut group = c.benchmark_group("buffer_set_string_ascii_line");
    group.throughput(Throughput::Bytes(ASCII_LINE.len() as u64));
    group.bench_function("80_columns", |b| {
        b.iter(|| buffer.set_string(0, 0, black_box(ASCII_LINE), Style::new()));
    });
    group.finish();
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

fn benchmark_diagnostic_workspace(c: &mut Criterion) {
    let keys = (0_u64..10_000).collect::<Vec<_>>();
    let mut workspace = DiagnosticWorkspaceState::<u64, u64>::new(120, 40);
    let _ = workspace.reconcile_findings(&keys);
    let _ = workspace.select_finding(&keys, 9_000);
    let mut group = c.benchmark_group("diagnostic_workspace_10k");

    group.throughput(Throughput::Elements(1));
    group.bench_function("navigation", |b| {
        b.iter(|| {
            if workspace.findings().selected_index() == Some(keys.len() - 1) {
                let _ = workspace.navigate_findings(&keys, VirtualListNavigation::First);
            }
            black_box(workspace.navigate_findings(black_box(&keys), VirtualListNavigation::Next))
        });
    });
    let mut narrow = false;
    group.bench_function("responsive_resize", |b| {
        b.iter(|| {
            narrow = !narrow;
            black_box(workspace.resize(if narrow { 60 } else { 120 }, 40))
        });
    });
    group.bench_function("virtual_window", |b| {
        b.iter(|| black_box(workspace.finding_window()));
    });
    group.finish();
}

fn benchmark_headless_snapshot(c: &mut Criterion) {
    let mut tree = RenderTree::new();
    let root_id = tree.next_id();
    tree.insert_root(
        BoxNode::new()
            .column()
            .width_percent(100.0)
            .height_percent(100.0)
            .build(root_id),
    );
    let mut semantics = vec![HeadlessSemanticNode::new(
        root_id,
        SemanticRole::Application,
        "Doctor",
    )];
    for _ in 0..50 {
        let id = tree.next_id();
        let mut node = TextNode::new("Finding").build(id);
        node.style.width = Dimension::Percent(100.0);
        node.style.height = Dimension::Points(1.0);
        node.style.flex_shrink = 0.0;
        tree.insert(node);
        tree.add_child(root_id, id);
        semantics.push(HeadlessSemanticNode::new(
            id,
            SemanticRole::ListItem,
            "Finding",
        ));
    }
    let presentation = HeadlessPresentation::new().with_semantics(semantics);
    let mut renderer = HeadlessRenderer::new(120, 40).unwrap();
    let mut group = c.benchmark_group("headless_snapshot");
    group.throughput(Throughput::Elements(120 * 40));
    group.bench_function("120x40_50_semantic_nodes", |b| {
        b.iter(|| {
            black_box(
                renderer
                    .render(black_box(&mut tree), black_box(&presentation))
                    .unwrap(),
            )
        });
    });
    group.finish();
}

fn benchmark_diagnostic_keymap(c: &mut Criterion) {
    let keymap = DiagnosticWorkspaceKeymap::default();
    let navigation = KeyEvent::key(Key::Down);
    let action = KeyEvent::char('/');
    let mut group = c.benchmark_group("diagnostic_keymap");

    group.throughput(Throughput::Elements(1));
    group.bench_function("resolve_navigation", |b| {
        b.iter(|| black_box(keymap.resolve(black_box(&navigation))));
    });
    group.bench_function("resolve_action", |b| {
        b.iter(|| black_box(keymap.resolve(black_box(&action))));
    });
    group.finish();
}

fn benchmark_diagnostic_presentation(c: &mut Criterion) {
    let presentation = DiagnosticPresentation::new(
        DiagnosticPresentationKind::Severity,
        "Critical",
        DiagnosticTone::Negative,
    )
    .unwrap();
    let wide = DiagnosticPresentationProfile::unicode();
    let narrow = DiagnosticPresentationProfile::ascii().with_compact(true);
    let mut group = c.benchmark_group("diagnostic_presentation");

    group.throughput(Throughput::Elements(1));
    group.bench_function("semantic_node", |b| {
        b.iter(|| black_box(presentation.semantic_node(black_box(42))));
    });
    group.bench_function("wide_text", |b| {
        b.iter(|| black_box(presentation.text(black_box(wide))));
    });
    group.bench_function("narrow_ascii_text", |b| {
        b.iter(|| black_box(presentation.text(black_box(narrow))));
    });
    group.finish();
}

fn benchmark_frame_telemetry(c: &mut Criterion) {
    let mut tree = RenderTree::new();
    let root = tree.next_id();
    tree.insert_root(
        BoxNode::new()
            .column()
            .width_percent(100.0)
            .height_percent(100.0)
            .build(root),
    );
    let row = tree.next_id();
    let mut node = TextNode::new("Selected finding").build(row);
    node.style.width = Dimension::Percent(100.0);
    node.style.height = Dimension::Points(1.0);
    tree.insert(node);
    tree.add_child(root, row);

    let mut backend = Backend::with_writer(120, 40, io::sink());
    let mut renderer = FrameRenderer::new();
    let mut coalescer = FrameCoalescer::new();
    coalescer.request_frame();
    renderer
        .render_pending(&mut tree, &mut backend, &mut coalescer)
        .unwrap();
    let mut selected = false;
    let mut group = c.benchmark_group("frame_telemetry");
    group.throughput(Throughput::Elements(tree.node_count() as u64));

    group.bench_function("selection_update", |b| {
        b.iter(|| {
            selected = !selected;
            tree.get_mut(row).unwrap().appearance.bold = selected;
            coalescer.request_frame();
            black_box(
                renderer
                    .render_pending(&mut tree, &mut backend, &mut coalescer)
                    .unwrap(),
            )
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_buffer_set_string,
    benchmark_buffer_ascii_line,
    benchmark_text_width,
    benchmark_text_wrap,
    benchmark_layout,
    benchmark_buffer_diff,
    benchmark_virtual_list,
    benchmark_injected_backend_output,
    benchmark_diagnostic_workspace,
    benchmark_headless_snapshot,
    benchmark_diagnostic_keymap,
    benchmark_diagnostic_presentation,
    benchmark_frame_telemetry,
);
criterion_main!(benches);
