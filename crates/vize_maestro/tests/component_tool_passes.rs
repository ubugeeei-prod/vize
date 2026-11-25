//! Compare complete editor requests on one descriptor and on separate descriptors.
#![cfg(test)]
use davinci_harness::{
    alloc::{CountingAllocator, mark_installed, measure_returning},
    paired::assert_paired_time,
};
use std::{alloc::System, hint::black_box};
use tower_lsp::lsp_types::{GotoDefinitionResponse, OneOf, Position, Range, TextDocumentEdit, Url};
use vize_atelier_sfc::{NativeStyleOptions, NativeStyleOutput, compile_native_styles};
use vize_glyph::{FormatOptions, GlyphFormatter};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
use vize_l1::{
    container::vue::{
        DescriptorOptions, L1Ir, ScriptRole,
        boundary::{split_blocks, tag_name_eq},
    },
    markup::render,
};
use vize_l1_to_l2::pass::template::OriginalTemplate;
use vize_maestro::{
    document::DocumentStore,
    runtime::block_on,
    source_project::{
        SourceSnapshot, SourceSnapshotCache, literal_products::LiteralEditorProducts,
    },
    utils::offset_to_position_str,
};
use vize_patina::{LintDiagnostic, Linter, RuleRegistry, rules::facts::MaxTemplateComplexity};

#[global_allocator]
static MEMORY: CountingAllocator<System> = CountingAllocator::system();

const INPUT: &str = include_str!("../../../tests/fixtures/shared-toolchain/editor-session.vue.txt");

struct Editor {
    docs: DocumentStore,
    cache: SourceSnapshotCache,
    uri: Url,
    binding: Position,
    variable: Position,
}
impl Editor {
    fn open(width: usize) -> Self {
        let uri = Url::parse("file:///Setup雪🌸.vue").unwrap();
        let docs = DocumentStore::new();
        let body = "<p v-if='count &amp;&amp; other'>雪</p>".repeat(width);
        let source = INPUT
            .trim_end()
            .replace('\n', "\r\n")
            .replace("<p>{{count}} {{other}} {{final}}</p>", &body);
        let pos = |needle| offset_to_position_str(&source, source.find(needle).unwrap());
        let (binding, variable) = (pos("other:"), pos("$ink;"));
        docs.open(uri.clone(), source, 7, "vue".into());
        let cache = SourceSnapshotCache::default();
        cache.capture(&docs, &uri).unwrap();
        Self {
            docs,
            cache,
            uri,
            binding,
            variable,
        }
    }

    fn request<R>(
        &self,
        run: impl for<'a> FnOnce(&'a SourceSnapshot, &'a Allocator, &L1Ir<'a>) -> R,
    ) -> R {
        let (request, _) = self.cache.begin_query(&self.docs, &self.uri).unwrap();
        let options = DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        };
        block_on(request.run_vue_passes(&self.docs, options, run))
            .unwrap()
            .publish(&self.docs, |out| out)
            .unwrap()
    }

    fn run(&self, shared: bool) -> Products {
        let script = |snapshot: &SourceSnapshot, l1ir: &L1Ir<'_>| {
            snapshot
                .prepare_literal_products(&self.docs, l1ir, ScriptRole::Setup)
                .unwrap()
                .publish(&self.docs, Some(self.binding))
                .unwrap()
        };
        let style = |snapshot: &SourceSnapshot, l1ir: &L1Ir<'_>| {
            let (selected, ()) = snapshot
                .prepare_scss_with(&self.docs, l1ir, 0, |_| ())
                .unwrap();
            let options = NativeStyleOptions {
                filename: "Setup雪🌸.vue",
                scope_id: None,
                trim: false,
                source_map: true,
            };
            let compiled = compile_native_styles(l1ir, options, [selected.products()]).unwrap();
            let definition = snapshot
                .scss_definition(&self.docs, selected.products(), self.variable)
                .unwrap();
            let scratch = Allocator::new();
            let settings = FormatOptions::default();
            let edit = selected
                .verified_format(&self.docs, &GlyphFormatter::new(&settings, &scratch))
                .unwrap()
                .publish(&self.docs)
                .unwrap();
            (compiled, edit, definition)
        };
        if shared {
            self.request(|snapshot, arena, owner| Products {
                script: script(snapshot, owner),
                style: style(snapshot, owner),
                template: template(arena, owner),
            })
        } else {
            Products {
                script: self.request(|snapshot, _, owner| script(snapshot, owner)),
                style: self.request(|snapshot, _, owner| style(snapshot, owner)),
                template: self.request(|_, arena, owner| template(arena, owner)),
            }
        }
    }
}

struct Template {
    body: vize_l0::String,
    owner_count: usize,
    warning: Option<LintDiagnostic>,
}

fn template<'a>(arena: &'a Allocator, l1ir: &L1Ir<'a>) -> Template {
    let selected = l1ir.admitted_template().unwrap().unwrap();
    let pass = OriginalTemplate::analyze(arena, selected, &mut NoObserver).unwrap();
    let mut body = vize_l0::String::default();
    render(pass.l1ir(), &mut |part| body.push_str(part));
    Template {
        body,
        owner_count: pass.analysis().static_facts.len(),
        warning: MaxTemplateComplexity::original_diagnostic(&pass),
    }
}

struct Products {
    script: LiteralEditorProducts,
    style: (
        NativeStyleOutput,
        Option<TextDocumentEdit>,
        Option<GotoDefinitionResponse>,
    ),
    template: Template,
}

fn diagnostic(row: Option<&LintDiagnostic>) -> serde_json::Value {
    row.map(|r| serde_json::json!({
        "rule": r.rule_name, "severity": r.severity, "message": r.message, "help": r.help,
        "start":r.start, "end":r.end, "fix":r.fix,
        "labels":r.labels.iter().map(|l| (l.start,l.end,l.message.as_str())).collect::<Vec<_>>(),
    })).unwrap_or(serde_json::Value::Null)
}

fn style_extent(source: &str) -> std::ops::Range<usize> {
    let mut extent = None;
    split_blocks(source, |block| {
        if tag_name_eq(block.tag_name, b"style") {
            extent = Some(block.location.tag_start..block.location.tag_end);
            return false;
        }
        true
    })
    .unwrap();
    extent.expect("the fixture's style block")
}

fn check(editor: &Editor, width: usize, prior: &Products, current: &Products) {
    let snapshot = editor.cache.capture(&editor.docs, &editor.uri).unwrap();
    let source = snapshot.source();
    let original_style = style_extent(source);
    let expected_range = Range::new(
        offset_to_position_str(source, original_style.start),
        offset_to_position_str(source, original_style.end),
    );
    let arena = Allocator::new();
    let settings = FormatOptions::default();
    let formatted = GlyphFormatter::new(&settings, &arena)
        .format(source)
        .unwrap();
    let expected_style = formatted
        .code
        .as_str()
        .get(style_extent(&formatted.code))
        .expect("the formatted style block has UTF-8 byte boundaries");

    let mut registry = RuleRegistry::new();
    registry.register(Box::new(MaxTemplateComplexity));
    let stock = Linter::with_registry(registry).lint_sfc(snapshot.source(), "Setup雪🌸.vue");
    let reference = stock
        .diagnostics
        .iter()
        .find(|d| d.rule_name == "vue/max-template-complexity");
    assert_eq!(reference.is_some(), width >= 20);
    let expected_warning = diagnostic(reference);
    for output in [prior, current] {
        assert_eq!(
            diagnostic(output.template.warning.as_ref()),
            expected_warning
        );
        assert_eq!(
            output.template.body,
            "<p v-if='count &amp;&amp; other'>雪</p>".repeat(width)
        );
        assert!(output.template.owner_count >= width);
        let [error] = output.script.type_errors.as_slice() else {
            panic!("one original type error")
        };
        assert_eq!(error.range.start, editor.binding);
        assert_eq!(
            error.message,
            "Type 'boolean' is not assignable to type 'number'."
        );
        assert_eq!(
            output.script.binding_hover.as_ref().unwrap().range.unwrap(),
            error.range
        );
        assert_eq!(output.script.version, 7);
        let (compiled, edit, definition) = &output.style;
        assert_eq!(
            compiled.css(),
            Some(".card {\n  color: red;\n  display: none;\n  margin-left: 4px;\n}")
        );
        let map: serde_json::Value =
            serde_json::from_str(compiled.css_source_map().unwrap()).unwrap();
        assert_eq!(
            map["sourcesContent"],
            serde_json::json!([snapshot.source()])
        );
        let edit = edit.as_ref().unwrap();
        assert_eq!(edit.text_document.version, Some(7));
        assert_eq!(edit.text_document.uri, editor.uri);
        let [OneOf::Left(replacement)] = edit.edits.as_slice() else {
            panic!("one verified formatter edit")
        };
        assert_eq!(replacement.range, expected_range);
        assert_eq!(replacement.new_text.as_str(), expected_style);
        let Some(GotoDefinitionResponse::Scalar(location)) = definition else {
            panic!("one original variable")
        };
        assert_eq!(location.uri, editor.uri);
        assert_eq!(
            location.range.start,
            offset_to_position_str(snapshot.source(), snapshot.source().find("$ink:").unwrap())
        );
    }
    assert_eq!(prior.style, current.style);
    assert_eq!(prior.template.owner_count, current.template.owner_count);
    assert_eq!(prior.script.type_errors, current.script.type_errors);
    assert_eq!(prior.script.binding_hover, current.script.binding_hover);
    assert_eq!(prior.script.prefer_const, current.script.prefer_const);
    assert_eq!(prior.script.outside, current.script.outside);
}

#[test]
fn five_tool_outputs_match_separately_published_queries() {
    for width in [1, 20] {
        let editor = Editor::open(width);
        check(&editor, width, &editor.run(false), &editor.run(true));
    }
}

#[test]
#[ignore = "paired optimized requests including compiler, lint, format, type and editor publication"]
fn measure_five_tools_on_one_revision() {
    mark_installed();
    for width in [1, 20, 64] {
        let editor = Editor::open(width);
        let (prior, before) = measure_returning(|| editor.run(false));
        let (current, after) = measure_returning(|| editor.run(true));
        check(&editor, width, &prior, &current);
        let (before, after) = (before.unwrap(), after.unwrap());
        let label = vize_l0::cstr!(
            "five tools width={width} calls={}/{} peak={}/{}",
            before.calls,
            after.calls,
            before.peak_bytes_over_start,
            after.peak_bytes_over_start
        );
        assert!(
            after.calls <= before.calls
                && after.peak_bytes_over_start <= before.peak_bytes_over_start,
            "{label}"
        );
        assert_paired_time(&label, 16, |shared| {
            black_box(editor.run(shared));
        });
    }
}
