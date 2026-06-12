//! Benchmark: rule-IR projection vs template-only rule traversal.
//!
//! Goal (issue #1503): show the zero-copy markup-IR adapter
//! ([`MarkupDocument::visit_with`]) adds no meaningful overhead compared with
//! running the same single rule through the established template-only path
//! ([`Linter::lint_template`]).
//!
//! Both arms parse the same Vue template with the same parser and run the
//! `a11y/img-alt` rule exactly once per element, so the delta is the cost of
//! the IR projection layer itself (the borrow-based facade + `MarkupRule`
//! dispatch) rather than parsing or diagnostic bookkeeping.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vize_carton::Allocator;
use vize_carton::String;
use vize_carton::append;
use vize_patina::ir::TemplateSyntax;
use vize_patina::markup::{MarkupContext, MarkupDocument};
use vize_patina::rules::a11y::ImgAlt;
use vize_patina::{JsxLang, LintContext, Linter, RuleRegistry};

/// A representative template with a mix of elements, bindings, and `<img>`
/// nodes (the rule's trigger), large enough to exercise traversal.
fn sample_template() -> String {
    let mut template = String::from("<div class=\"gallery\">\n");
    for i in 0..80 {
        append!(
            template,
            r#"  <figure class="item">
    <img src="/photo{i}.jpg" />
    <figcaption :title="caption{i}">Photo {i}</figcaption>
    <button @click="open{i}">Open</button>
  </figure>
"#,
        );
    }
    template.push_str("</div>");
    template
}

/// Template-only baseline: a `Linter` with just `a11y/img-alt` registered,
/// driven through the existing `lint_template` path (parse + `LintVisitor`).
fn lint_template_only(linter: &Linter, source: &str) -> usize {
    linter.lint_template(source, "bench.vue").warning_count
}

/// Rule-IR path: parse the same template, then drive `a11y/img-alt` as a
/// `MarkupRule` over the zero-copy [`MarkupDocument`].
fn lint_via_markup_ir(source: &str) -> usize {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(allocator.as_bump(), source);
    let (root, _errors) = parser.parse();
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let rule = ImgAlt;
    let mut lint = LintContext::new(&allocator, source, "bench.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(&rule, &mut ctx);
    lint.warning_count()
}

/// A representative JSX module: a component returning a gallery with many
/// `<img>` nodes (the rule's trigger), bindings, and events — large enough to
/// exercise the projection / lowering traversal.
fn sample_jsx() -> String {
    let mut module = String::from("const Gallery = () => (\n  <div className=\"gallery\">\n");
    for i in 0..80 {
        append!(
            module,
            r#"    <figure className="item">
      <img src={{`/photo{i}.jpg`}} />
      <figcaption title={{`caption{i}`}}>Photo {i}</figcaption>
      <button onClick={{() => open({i})}}>Open</button>
    </figure>
"#,
        );
    }
    module.push_str("  </div>\n);");
    module
}

/// JSX lint over the **zero-cost rule IR**: `a11y/img-alt` is markup-capable, so
/// `lint_jsx` parses the OXC program once and runs the rule straight over the
/// borrow-based [`MarkupDocument::from_jsx`] facade — no template reconstruction.
fn lint_jsx_via_ir(linter: &Linter, source: &str) -> usize {
    linter
        .lint_jsx(source, "bench.jsx", JsxLang::Jsx)
        .warning_count
}

/// JSX lint over the **lowering fallback**: `vue/a11y-img-alt` has only a legacy
/// `Rule` impl, so the same `<img>` alt check is served by lowering the JSX to a
/// synthetic relief template AST first — the allocation-heavy reconstruction the
/// IR path avoids.
fn lint_jsx_via_lowering(linter: &Linter, source: &str) -> usize {
    linter
        .lint_jsx(source, "bench.jsx", JsxLang::Jsx)
        .warning_count
}

fn bench_jsx_ir_vs_lowering(c: &mut Criterion) {
    use vize_patina::rules::vue::A11yImgAlt;

    let module = sample_jsx();

    // IR arm: a markup-capable rule (runs over the OXC projection).
    let mut ir_registry = RuleRegistry::new();
    ir_registry.register(Box::new(ImgAlt));
    let ir_linter = Linter::with_registry(ir_registry);

    // Lowering arm: an equivalent unmigrated rule (forces the lowering fallback).
    let mut lowering_registry = RuleRegistry::new();
    lowering_registry.register(Box::new(A11yImgAlt));
    let lowering_linter = Linter::with_registry(lowering_registry);

    // Sanity: both arms must flag the same number of `<img>` nodes, otherwise
    // the comparison is meaningless.
    let expected = lint_jsx_via_ir(&ir_linter, &module);
    assert_eq!(lint_jsx_via_lowering(&lowering_linter, &module), expected);
    assert!(expected > 0, "fixture should trigger the rule");

    let mut group = c.benchmark_group("jsx_lint");
    group.throughput(Throughput::Bytes(module.len() as u64));

    group.bench_function("ir_projection", |b| {
        b.iter(|| lint_jsx_via_ir(&ir_linter, black_box(&module)))
    });

    group.bench_function("lowering_fallback", |b| {
        b.iter(|| lint_jsx_via_lowering(&lowering_linter, black_box(&module)))
    });

    group.finish();
}

fn bench_markup_ir_vs_template(c: &mut Criterion) {
    let template = sample_template();

    // Baseline linter holds exactly one rule so both arms do equal work.
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ImgAlt));
    let linter = Linter::with_registry(registry);

    // Sanity: both paths must agree on the diagnostic count, otherwise the
    // comparison is meaningless.
    let expected = lint_template_only(&linter, &template);
    assert_eq!(lint_via_markup_ir(&template), expected);
    assert!(expected > 0, "fixture should trigger the rule");

    let mut group = c.benchmark_group("markup_ir");
    group.throughput(Throughput::Bytes(template.len() as u64));

    group.bench_function("template_only", |b| {
        b.iter(|| lint_template_only(&linter, black_box(&template)))
    });

    group.bench_function("rule_ir", |b| {
        b.iter(|| lint_via_markup_ir(black_box(&template)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_markup_ir_vs_template,
    bench_jsx_ir_vs_lowering
);
criterion_main!(benches);
