//! Allocation regression probe for the direct-IR JSX markup pass.
//!
//! Setup (parse, contexts, registry) stays outside the stage window; the
//! measured section is exactly the per-rule `visit_with` loop `lint_jsx`
//! drives over the OXC program. Root discovery streams each outermost JSX
//! element straight into the walker, so the committed one-root fixture must
//! perform **zero** root-container allocations no matter how many markup
//! rules run — the exact `allocs` budget makes any reintroduced per-rule
//! root vector (or child spill) fail closed.

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_patina::ir::TemplateSyntax;
use vize_patina::markup::{MarkupContext, MarkupDocument};
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
        let oxc_allocator = oxc_allocator::Allocator::default();
        let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, ONE_ROOT, JsxLang::Jsx);
        assert!(parsed.diagnostics.is_empty(), "fixture must parse cleanly");
        let allocator = Allocator::new();
        let mut lint = LintContext::new(&allocator, ONE_ROOT, "bench.jsx");
        let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);
        let mut markup_ctx = MarkupContext::new(&mut lint, &document);
        let visited = window.measure(|| {
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
        });
        drop(markup_ctx);
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

criterion_group!(davinci_markup_group, davinci_markup);
davinci_harness::main!(davinci_markup_group);
