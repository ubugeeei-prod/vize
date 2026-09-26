//! P4-7b allocation gates run in one process because the counters are global.

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure, measure_returning};
use vize_l0::Allocator;
use vize_patina::ir::TemplateSyntax;
use vize_patina::markup::{L2Template, MarkupContext, MarkupDocument};
use vize_patina::{LintContext, RuleRegistry};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::mimalloc();

const GALLERY: &str = r#"<section class="gallery">
  <img src="/photo.jpg" alt="A photo" />
  <button type="button" @click="open()">Open</button>
</section>
"#;

#[test]
fn authored_projection_reuses_ordinary_storage_and_holds_the_facade_budget() {
    mark_installed();
    for source in [
        GALLERY,
        "<table><tr><td>row</td></tr></table>",
        "<div v-pre><span>{{ raw }}</span></div>",
        "<svg><a><a>foreign</a></a></svg>",
    ] {
        let normal_arena = Allocator::new();
        let authored_arena = Allocator::new();
        let (_normal, normal) = measure_returning(|| vize_l1::parse(&normal_arena, source));
        let ((_tree, extra, _errors), projected) =
            measure_returning(|| vize_l1::parse_with_authored(&authored_arena, source));
        assert!(
            extra.is_none(),
            "ordinary source must reuse its existing tree"
        );
        assert_eq!(
            normal, projected,
            "ordinary source adds no heap allocation: {source}"
        );
        assert_eq!(
            normal_arena.allocated_bytes(),
            authored_arena.allocated_bytes(),
            "ordinary source adds no arena storage: {source}"
        );
    }

    let allocator = Allocator::new();
    let lowered = L2Template::lower(&allocator, GALLERY);
    assert!(lowered.lowered().diagnostics.is_empty());
    let markup = lowered.markup();
    let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
    let registry = RuleRegistry::default();
    let mut lint = LintContext::new(&allocator, GALLERY, "budget.vue");
    let mut context = MarkupContext::new(&mut lint, &document);
    // Prime the lazy WHATWG table and rule catalog outside the stage window,
    // as the benchmark does before recording a repeated traversal.
    for rule in registry.rules() {
        if let Some(rule) = rule.as_markup_rule() {
            document.visit_with(rule, &mut context);
        }
    }
    let mut count = 0;
    for rule in registry.rules() {
        if let Some(rule) = rule.as_markup_rule() {
            let metrics = measure(|| document.visit_with(rule, &mut context))
                .expect("counting allocator installed");
            let expected = if rule.name() == "vue/permitted-contents" {
                7
            } else {
                0
            };
            assert_eq!(metrics.calls, expected, "{} allocation budget", rule.name());
            count += 1;
        }
    }
    assert!(count > 0, "the default registry must drive markup rules");
    assert_eq!(lint.error_count() + lint.warning_count(), 0);
}
