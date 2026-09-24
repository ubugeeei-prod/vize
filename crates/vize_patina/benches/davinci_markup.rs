//! Allocation regression probes for the markup facade passes: the S2 JSX
//! pass and the S2-backed template pass (P4-7a, switched in P4-7b).
//!
//! Setup (lower, contexts, registry) stays outside the stage window; the
//! measured section is exactly the per-rule `visit_with` loop `lint_jsx`
//! drives over the S2 projection for rules that do not need the list/branch
//! partition. The one-root fixture must perform **zero** allocations in that
//! window — the exact `allocs` budget makes any reintroduced per-rule root
//! vector (or child spill) fail closed.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "the bench asserts its fixture's shape by panicking"
)]

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_patina::ir::TemplateSyntax;
use vize_patina::markup::{MarkupContext, MarkupDocument, S2Markup, S2Template};
use vize_patina::{JsxLang, LintContext, RuleRegistry};
use vize_s0::{Allocator, cstr};

/// One-root, diagnostic-free JSX module: a single outermost element with a
/// nested child mix (static attribute, event, text) so the walk exercises
/// element, binding, and text projection without firing any rule.
const ONE_ROOT: &str = r#"const Gallery = () => (
  <section className="gallery">
    <img src="/photo.jpg" alt="A photo" />
    <button type="button" onClick={() => open()}>Open</button>
  </section>
);
"#;

fn davinci_markup(criterion: &mut Criterion) {
    let registry = RuleRegistry::default();
    let id = cstr!("patina_jsx_markup_one_root");
    bench_stage_with_metrics(criterion, &id, "synthetic:jsx-one-root-gallery", |window| {
        let allocator = Allocator::new();
        let lowered =
            vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), ONE_ROOT, JsxLang::Jsx);
        assert!(
            lowered
                .diagnostics
                .iter()
                .all(|diagnostic| !diagnostic.is_error()),
            "fixture must lower cleanly"
        );
        let projected = lowered.roots[0]
            .s2
            .as_ref()
            .expect("the gallery root must project to S2");
        let markup = S2Markup::from_projected_root(projected);
        let mut lint = LintContext::new(&allocator, ONE_ROOT, "bench.jsx");
        let document = MarkupDocument::from_s2(&markup, TemplateSyntax::Vue);
        let visited = {
            let mut markup_ctx = MarkupContext::new(&mut lint, &document);
            window.measure(|| {
                let mut visited = 0u32;
                for rule in registry.rules() {
                    if rule.jsx_needs_lowering() {
                        continue;
                    }
                    if let Some(markup_rule) = rule.as_markup_rule() {
                        document.visit_with(markup_rule, &mut markup_ctx);
                        visited += 1;
                    }
                }
                visited
            })
        };
        assert!(visited > 0, "the direct-IR pass must drive markup rules");
        assert_eq!(
            lint.warning_count() + lint.error_count(),
            0,
            "the one-root fixture must stay diagnostic-free so the alloc \
                 budget witnesses traversal, not reporting"
        );
        visited
    });
}

/// The same one-root gallery as a Vue template, for the S2 facade.
const ONE_ROOT_TEMPLATE: &str = r#"<section class="gallery">
  <img src="/photo.jpg" alt="A photo" />
  <button type="button" @click="open()">Open</button>
</section>
"#;

/// P4-7a: the per-rule `visit_with` loop over the S2-backed facade. Parse and
/// the S1→S2 lowering stay in setup; the measured window is the traversal
/// only. The view is zero-copy — element, binding and text projection borrow
/// the op tree, the S1 surface and the side tables, and allocate nothing; the
/// window's whole exact `allocs` budget (7) is `vue/permitted-contents`
/// building its owned content-model skeleton (P4-11a), measured by skipping
/// that one rule, which leaves 0.
fn davinci_s2_markup(criterion: &mut Criterion) {
    let registry = RuleRegistry::default();
    let id = cstr!("patina_s2_markup_one_root");
    bench_stage_with_metrics(criterion, &id, "synthetic:s2-one-root-gallery", |window| {
        let allocator = Allocator::new();
        let lowered = S2Template::lower(&allocator, ONE_ROOT_TEMPLATE);
        assert!(
            lowered.lowered().diagnostics.is_empty(),
            "fixture must lower cleanly"
        );
        let markup = lowered.markup();
        let mut lint = LintContext::new(&allocator, ONE_ROOT_TEMPLATE, "bench.vue");
        let document = MarkupDocument::from_s2(&markup, TemplateSyntax::Vue);
        let visited = {
            let mut markup_ctx = MarkupContext::new(&mut lint, &document);
            window.measure(|| {
                let mut visited = 0u32;
                for rule in registry.rules() {
                    if let Some(markup_rule) = rule.as_markup_rule() {
                        document.visit_with(markup_rule, &mut markup_ctx);
                        visited += 1;
                    }
                }
                visited
            })
        };
        assert!(visited > 0, "the S2 pass must drive markup rules");
        assert_eq!(
            lint.warning_count() + lint.error_count(),
            0,
            "the one-root fixture must stay diagnostic-free so the alloc \
                 budget witnesses traversal, not reporting"
        );
        visited
    });
}

criterion_group!(davinci_markup_group, davinci_markup, davinci_s2_markup);
davinci_harness::main!(davinci_markup_group);
