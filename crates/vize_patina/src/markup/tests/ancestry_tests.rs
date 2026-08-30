use std::cell::RefCell;

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{
    MarkupBinding, MarkupContext, MarkupDirective, MarkupDocument, MarkupElement, MarkupRule,
    MarkupText,
};
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

#[derive(Default)]
struct AncestryProbe {
    entries: RefCell<Vec<String>>,
    exits: RefCell<Vec<String>>,
    callbacks: RefCell<Vec<String>>,
}

impl MarkupRule for AncestryProbe {
    fn name(&self) -> &'static str {
        "test/markup-ancestry"
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.entries
            .borrow_mut()
            .push(snapshot("enter", ctx, element));
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.exits.borrow_mut().push(snapshot("exit", ctx, element));
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if element.is_tag("section") {
            self.callbacks.borrow_mut().push(format!(
                "binding:{} {}",
                binding.arg_name().unwrap_or("-"),
                stack_snapshot(ctx)
            ));
        }
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        if element.is_tag("section") {
            self.callbacks.borrow_mut().push(format!(
                "directive:{}:{} {}",
                directive.name(),
                directive.arg_name().unwrap_or("-"),
                stack_snapshot(ctx)
            ));
        }
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        if !text.content().trim().is_empty() {
            self.callbacks
                .borrow_mut()
                .push(format!("text {}", stack_snapshot(ctx)));
        }
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, _range: crate::ir::ByteRange) {
        self.callbacks
            .borrow_mut()
            .push(format!("interpolation {}", stack_snapshot(ctx)));
    }
}

fn snapshot(label: &str, ctx: &MarkupContext<'_, '_>, element: &MarkupElement<'_>) -> String {
    let parent = ctx
        .parent_element()
        .map(tag_name)
        .unwrap_or_else(|| "-".to_owned());
    let current = ctx
        .current_element()
        .map(tag_name)
        .unwrap_or_else(|| "-".to_owned());
    let ancestors = ctx.ancestor_elements().map(tag_name).collect::<Vec<_>>();
    let has_section = ctx.has_ancestor(|ancestor| ancestor.is_tag("section"));

    format!(
        "{label}:{} current={current} parent={parent} ancestors=[{}] has_section={has_section}",
        tag_name(*element),
        ancestors.join("/")
    )
}

fn stack_snapshot(ctx: &MarkupContext<'_, '_>) -> String {
    let parent = ctx
        .parent_element()
        .map(tag_name)
        .unwrap_or_else(|| "-".to_owned());
    let current = ctx
        .current_element()
        .map(tag_name)
        .unwrap_or_else(|| "-".to_owned());
    let ancestors = ctx.ancestor_elements().map(tag_name).collect::<Vec<_>>();

    format!(
        "current={current} parent={parent} ancestors=[{}]",
        ancestors.join("/")
    )
}

fn tag_name(element: MarkupElement<'_>) -> String {
    let tag = element.tag();
    if tag.is_empty() {
        "#fragment".to_owned()
    } else {
        tag.to_owned()
    }
}

fn run_over_template(source: &str) -> AncestryProbe {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, errors) = parser.parse();
    assert!(errors.is_empty(), "template parse errors: {errors:?}");
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let probe = AncestryProbe::default();
    let mut lint = LintContext::new(&allocator, source, "test.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(&probe, &mut ctx);
    probe
}

fn run_over_jsx_oxc(source: &str) -> AncestryProbe {
    let oxc_allocator = oxc_allocator::Allocator::default();
    let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
    let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

    let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let probe = AncestryProbe::default();
    let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(&probe, &mut ctx);
    probe
}

fn run_over_jsx_lowered(source: &str) -> Vec<String> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

    let mut entries = Vec::new();
    for lowered_root in &lowered.roots {
        let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
        let probe = AncestryProbe::default();
        let mut lint = LintContext::new(&allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(&probe, &mut ctx);
        entries.extend(probe.entries.into_inner());
    }
    entries
}

#[test]
fn template_callbacks_observe_the_current_element_stack() {
    let probe = run_over_template(
        r#"<section id="root" @click="go">hello {{ name }}<span>leaf</span></section>"#,
    );

    assert_eq!(
        probe.callbacks.into_inner(),
        vec![
            "binding:id current=section parent=- ancestors=[]",
            "binding:click current=section parent=- ancestors=[]",
            "directive:on:click current=section parent=- ancestors=[]",
            "text current=section parent=- ancestors=[]",
            "interpolation current=section parent=- ancestors=[]",
            "text current=span parent=section ancestors=[section]",
        ]
    );
}

#[test]
fn jsx_callbacks_observe_the_current_element_stack() {
    let probe = run_over_jsx_oxc(
        r#"const C = () => <section id="root" onClick={go}>hello {name}<span>leaf</span></section>;"#,
    );

    assert_eq!(
        probe.callbacks.into_inner(),
        vec![
            "binding:id current=section parent=- ancestors=[]",
            "binding:Click current=section parent=- ancestors=[]",
            "directive:on:Click current=section parent=- ancestors=[]",
            "text current=section parent=- ancestors=[]",
            "interpolation current=section parent=- ancestors=[]",
            "text current=span parent=section ancestors=[section]",
        ]
    );
}

#[test]
fn template_ancestry_tracks_current_parent_and_root_first_ancestors() {
    let probe = run_over_template("<section><p><span>Text</span></p></section><aside></aside>");

    assert_eq!(
        probe.entries.into_inner(),
        vec![
            "enter:section current=section parent=- ancestors=[] has_section=false",
            "enter:p current=p parent=section ancestors=[section] has_section=true",
            "enter:span current=span parent=p ancestors=[section/p] has_section=true",
            "enter:aside current=aside parent=- ancestors=[] has_section=false",
        ]
    );
    assert_eq!(
        probe.exits.into_inner(),
        vec![
            "exit:span current=span parent=p ancestors=[section/p] has_section=true",
            "exit:p current=p parent=section ancestors=[section] has_section=true",
            "exit:section current=section parent=- ancestors=[] has_section=false",
            "exit:aside current=aside parent=- ancestors=[] has_section=false",
        ],
        "exit hooks must see the element before it is popped"
    );
}

#[test]
fn template_directive_scopes_do_not_reset_element_ancestry() {
    let probe = run_over_template(
        r#"<section><p v-if="ok"></p><ul><li v-for="item in items"><span></span></li></ul><footer></footer></section>"#,
    );

    assert_eq!(
        probe.entries.into_inner(),
        vec![
            "enter:section current=section parent=- ancestors=[] has_section=false",
            "enter:p current=p parent=section ancestors=[section] has_section=true",
            "enter:ul current=ul parent=section ancestors=[section] has_section=true",
            "enter:li current=li parent=ul ancestors=[section/ul] has_section=true",
            "enter:span current=span parent=li ancestors=[section/ul/li] has_section=true",
            "enter:footer current=footer parent=section ancestors=[section] has_section=true",
        ]
    );
}

#[test]
fn jsx_ancestry_tracks_elements_and_fragments() {
    let probe =
        run_over_jsx_oxc("const C = () => <section><><p><span /></p></><aside /></section>;");

    assert_eq!(
        probe.entries.into_inner(),
        vec![
            "enter:section current=section parent=- ancestors=[] has_section=false",
            "enter:#fragment current=#fragment parent=section ancestors=[section] has_section=true",
            "enter:p current=p parent=#fragment ancestors=[section/#fragment] has_section=true",
            "enter:span current=span parent=p ancestors=[section/#fragment/p] has_section=true",
            "enter:aside current=aside parent=section ancestors=[section] has_section=true",
        ]
    );
}

#[test]
fn jsx_attribute_value_roots_keep_the_host_traversal_parent() {
    let probe = run_over_jsx_oxc("const C = () => <Panel header={<ul><li /></ul>}><p /></Panel>;");

    assert_eq!(
        probe.entries.into_inner(),
        vec![
            "enter:Panel current=Panel parent=- ancestors=[] has_section=false",
            "enter:ul current=ul parent=Panel ancestors=[Panel] has_section=false",
            "enter:li current=li parent=ul ancestors=[Panel/ul] has_section=false",
            "enter:p current=p parent=Panel ancestors=[Panel] has_section=false",
        ]
    );
}

#[test]
fn lowered_jsx_roots_get_the_same_template_stack_contract() {
    assert_eq!(
        run_over_jsx_lowered(
            "const C = () => <section>{items.map((item) => <p key={item.id}><span /></p>)}</section>;",
        ),
        vec![
            "enter:section current=section parent=- ancestors=[] has_section=false",
            "enter:p current=p parent=section ancestors=[section] has_section=true",
            "enter:span current=span parent=p ancestors=[section/p] has_section=true",
        ]
    );
}
