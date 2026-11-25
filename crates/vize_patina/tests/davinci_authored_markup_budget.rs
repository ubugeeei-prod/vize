//! P4-7b allocation gates run directly on the process main thread.
//! The counters also include allocations on spawned threads, so libtest's
//! concurrent result-channel bookkeeping must stay outside these windows.

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure, measure_returning};
use vize_l0::Allocator;
use vize_patina::ir::TemplateSyntax;
use vize_patina::markup::{LintL2Template, MarkupContext, MarkupDocument};
use vize_patina::{LintContext, Linter, RuleRegistry};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::mimalloc();

const GALLERY: &str = r#"<section class="gallery">
  <img src="/photo.jpg" alt="A photo" />
  <button type="button" @click="open()">Open</button>
</section>
"#;

const CASE: &str = "authored_projection_reuses_ordinary_storage_and_holds_the_facade_budget";

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1);
    let (mut list, mut ignored, mut exact) = (false, false, false);
    let mut filter = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--ignored" => ignored = true,
            "--exact" => exact = true,
            "--nocapture" => {}
            "--format" if args.next().as_deref() == Some("terse") => {}
            value if !value.starts_with('-') && filter.is_none() => filter = Some(arg),
            _ => return Err("unsupported harness argument"),
        }
    }
    let selected = filter.as_ref().is_none_or(|filter| {
        if exact {
            filter == CASE
        } else {
            CASE.contains(filter)
        }
    });
    if ignored || !selected {
        return Ok(());
    }
    if list {
        println!("{CASE}: test");
        return Ok(());
    }
    authored_projection_reuses_ordinary_storage_and_holds_the_facade_budget();
    Ok(())
}

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
        let (_normal, normal) = measure_returning(|| vize_l1::markup::parse(&normal_arena, source));
        let ((_tree, extra, _errors), projected) =
            measure_returning(|| vize_l1::markup::parse_with_authored(&authored_arena, source));
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
    let lowered = LintL2Template::lower(&allocator, GALLERY);
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
            // `None` means the counting allocator is not installed.
            let calls =
                measure(|| document.visit_with(rule, &mut context)).map(|metrics| metrics.calls);
            let expected = if rule.name() == "vue/permitted-contents" {
                7
            } else {
                0
            };
            assert_eq!(calls, Some(expected), "{} allocation budget", rule.name());
            count += 1;
        }
    }
    assert!(count > 0, "the default registry must drive markup rules");
    assert_eq!(lint.error_count() + lint.warning_count(), 0);

    // Default class checks need no sets when no duplicate is possible. Keep
    // these complete source traversals inside the original zero-allocation gate.
    let class_rules = registry
        .rules()
        .iter()
        .filter(|rule| rule.meta().name == "html/no-duplicate-class")
        .filter_map(|rule| rule.as_markup_rule())
        .collect::<Vec<_>>();
    assert_eq!(
        class_rules.len(),
        1,
        "static class checks enabled by default as one markup rule"
    );
    for &class_rule in &class_rules {
        for source in [
            "<div class=\"\">Text</div>",
            "<div class=\" \t\r\n\u{c} \">Text</div>",
            "<div class=\"token\">Text</div>",
            "<div class=\" \t 東京 \u{c}\r\n \">Text</div>",
            "<div class=\"a\u{b}a\">Text</div>",
            "<div class=\"a\u{a0}a\">Text</div>",
        ] {
            let allocator = Allocator::new();
            let lowered = LintL2Template::lower(&allocator, source);
            assert!(lowered.lowered().diagnostics.is_empty());
            let markup = lowered.markup();
            let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
            let mut lint = LintContext::new(&allocator, source, "class-budget.vue");
            let mut context = MarkupContext::new(&mut lint, &document);
            let calls = measure(|| document.visit_with(class_rule, &mut context))
                .map(|metrics| metrics.calls);
            assert_eq!(calls, Some(0), "empty/single token allocation: {source}");
            assert_eq!(lint.error_count() + lint.warning_count(), 0, "{source}");
        }
    }
    lint_template_allocation_budget();
}

const LINT_SECTION: &str = r#"<section v-if="ready"><!-- note --><b :title="heading">Hello {{ name }}!</b><ul><li v-for="item in items" :key="item.id">{{ item }}</li></ul><button @click="save">Save</button></section>"#;

/// The public template lint entry, measured in the same single-case process.
fn lint_template_allocation_budget() {
    let linter = Linter::new();
    let source = format!("<main>{}</main>", LINT_SECTION.repeat(96));
    drop(linter.lint_template(&source, "fixture.vue"));
    let (result, metrics) = measure_returning(|| linter.lint_template(&source, "fixture.vue"));
    assert!(result.diagnostics.is_empty());
    let calls = metrics.expect("counting allocator installed").calls;
    // Recording every lowering decision took 4,417 calls on this fixture.
    // The product entry used 3,831 after switching to the unobserved lowering.
    assert!(
        calls <= 4_000,
        "{calls} allocations while linting a template"
    );
}
