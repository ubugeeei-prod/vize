use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, PartialResultParams, Position,
    SymbolKind, TextDocumentIdentifier, Url, WorkDoneProgressParams,
};

use super::document_symbols;
use crate::server::ServerState;

fn outline(source: &str) -> Vec<DocumentSymbol> {
    let uri = Url::parse("file:///Outline.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let result = document_symbols(
        &state,
        &DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        },
    );
    let Some(DocumentSymbolResponse::Nested(symbols)) = result else {
        panic!("outline unavailable");
    };
    symbols
}

fn names(symbols: &[DocumentSymbol]) -> Vec<&str> {
    symbols.iter().map(|symbol| symbol.name.as_str()).collect()
}

fn contained(symbols: &[DocumentSymbol]) {
    for parent in symbols {
        if let Some(children) = &parent.children {
            for child in children {
                assert!(parent.range.start <= child.range.start);
                assert!(child.range.end <= parent.range.end);
            }
            contained(children);
        }
    }
}

#[test]
fn original_reported_projects_preserve_the_complete_authored_hierarchies() {
    for (source, expected) in [
        (
            include_str!(
                "../../../../../../tests/_fixtures/differential/lsp/document-symbol-outline-original/Parent.vue.txt"
            ),
            include_str!(
                "../../../../../../tests/_fixtures/differential/lsp/document-symbol-outline-original/Parent.response.expected.json"
            ),
        ),
        (
            include_str!(
                "../../../../../../tests/_fixtures/differential/lsp/document-symbol-outline-original/MySwitch.vue.txt"
            ),
            include_str!(
                "../../../../../../tests/_fixtures/differential/lsp/document-symbol-outline-original/MySwitch.response.expected.json"
            ),
        ),
    ] {
        let symbols = outline(source);
        contained(&symbols);
        assert_eq!(
            serde_json::to_value(symbols).unwrap(),
            serde_json::from_str::<serde_json::Value>(expected).unwrap()
        );
    }
}

#[test]
fn full_element_ends_use_original_opaque_children_and_never_consume_parent_closures() {
    let adjacent = outline("<template><Widget><Widget/></Widget></template>");
    let outer = &adjacent[0].children.as_ref().unwrap()[0];
    assert_eq!(outer.range.end, Position::new(0, 36));
    assert_eq!(
        outer.children.as_ref().unwrap()[0].range.end,
        Position::new(0, 27)
    );
    let source = "<template><Widget><Widget/><input><textarea>&lt;Widget&gt;</textarea><!-- </Widget> --><p>{{ '</Widget>' }}</p> \n</Widget></template>";
    let symbols = outline(source);
    contained(&symbols);
    let outer = &symbols[0].children.as_ref().unwrap()[0];
    let children = outer.children.as_ref().unwrap();
    assert_eq!(names(children), ["Widget", "input", "textarea", "p"]);
    let index = vize_l0::line_index::LineIndex::new(source);
    for (symbol, fragment) in [
        (
            outer,
            "<Widget><Widget/><input><textarea>&lt;Widget&gt;</textarea><!-- </Widget> --><p>{{ '</Widget>' }}</p> \n</Widget>",
        ),
        (&children[0], "<Widget/>"),
        (&children[1], "<input>"),
        (&children[2], "<textarea>&lt;Widget&gt;</textarea>"),
        (&children[3], "<p>{{ '</Widget>' }}</p>"),
    ] {
        let start = source.find(fragment).unwrap();
        let (line, character) = index.line_col(start + fragment.len());
        assert_eq!(symbol.range.end, Position::new(line, character));
    }
}

#[test]
fn closing_witness_refuses_later_or_prefix_matches() {
    for (source, expected) in [
        (" \n</p >later", 7),
        ("</paragraph>", 0),
        ("text</p>", 0),
        ("<!-- </p> -->", 0),
    ] {
        assert_eq!(super::template::closing_end(source, "p", 0), expected);
    }
}

#[test]
fn mixed_case_closing_tags_follow_the_original_parser_without_normalizing_names() {
    let symbols = outline("<template><DIV><span></SPAN></div></template>");
    contained(&symbols);
    let parent = &symbols[0].children.as_ref().unwrap()[0];
    let child = &parent.children.as_ref().unwrap()[0];
    assert_eq!(parent.name, "DIV");
    assert_eq!(parent.range.end, Position::new(0, 34));
    assert_eq!(parent.selection_range.start, Position::new(0, 11));
    assert_eq!(parent.selection_range.end, Position::new(0, 14));
    assert_eq!(child.name, "span");
    assert_eq!(child.range.end, Position::new(0, 28));
    assert_eq!(child.selection_range.start, Position::new(0, 16));
    assert_eq!(child.selection_range.end, Position::new(0, 20));
}

#[test]
fn declarations_follow_ast_order_without_local_variables_references_or_comments() {
    let source = "<script setup>\n// const fake = 1;\nconst {items: local, checked = true, ...rest} = props;\nlet [first,, ...tail] = list;\nfunction next() { const hidden = 1; }\nnext();\n</script>";
    let symbols = outline(source);
    let children = symbols[0].children.as_ref().unwrap();
    assert_eq!(
        names(children),
        ["local", "checked", "rest", "first", "tail", "next"]
    );
    assert_eq!(children[0].kind, SymbolKind::CONSTANT);
    assert_eq!(children[3].kind, SymbolKind::VARIABLE);
    assert_eq!(children[5].kind, SymbolKind::FUNCTION);
    assert!(children[5].children.is_none());
}

#[test]
fn tsx_jsx_and_typed_exports_keep_original_identifier_spans() {
    for lang in ["tsx", "jsx"] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"{lang}\">\nexport const Render = () => <Widget />;\n</script>"
        );
        let symbols = outline(&source);
        let child = &symbols[0].children.as_ref().unwrap()[0];
        assert_eq!(child.name, "Render");
        assert_eq!(child.kind, SymbolKind::FUNCTION);
        assert_eq!(child.selection_range.start, Position::new(1, 13));
        assert_eq!(child.selection_range.end, Position::new(1, 19));
        assert!(child.children.is_none());
    }
    let symbols = outline(
        "<script lang=\"ts\">\nexport interface Shape { name: string }\nexport type Id = string;\nexport class Box { value = 1; get() { return this.value; } }\n</script>",
    );
    let children = symbols[0].children.as_ref().unwrap();
    assert_eq!(names(children), ["Shape", "Id", "Box"]);
    assert_eq!(children[0].kind, SymbolKind::INTERFACE);
    assert_eq!(
        names(children[2].children.as_ref().unwrap()),
        ["value", "get"]
    );
}

#[test]
fn static_members_preserve_nested_objects_and_omit_unresolved_keys_and_spreads() {
    let symbols = outline(
        "<script setup lang=\"ts\">const store = ({ get: () => 1, set(v: number) {}, nested: { value: 2 }, [dynamic]: 3, ...spread } satisfies object);</script>",
    );
    let store = &symbols[0].children.as_ref().unwrap()[0];
    let members = store.children.as_ref().unwrap();
    assert_eq!(names(members), ["get", "set", "nested"]);
    assert_eq!(members[0].kind, SymbolKind::METHOD);
    assert_eq!(members[1].kind, SymbolKind::METHOD);
    assert_eq!(names(members[2].children.as_ref().unwrap()), ["value"]);
}

#[test]
fn untransformed_template_elements_keep_nested_directives_and_components() {
    let symbols = outline(
        "<template><section><template v-for=\"item in list\"><Widget v-if=\"item\" ref=\"widget\"/><p>{{ item }}</p></template></section></template>",
    );
    let section = &symbols[0].children.as_ref().unwrap()[0];
    assert_eq!(section.name, "section");
    let nested_template = &section.children.as_ref().unwrap()[0];
    assert_eq!(nested_template.name, "template");
    let children = nested_template.children.as_ref().unwrap();
    assert_eq!(names(children), ["Widget", "p"]);
    assert_eq!(children[0].kind, SymbolKind::CLASS);
    assert_eq!(children[1].kind, SymbolKind::OBJECT);
}

#[test]
fn implicit_html_table_repairs_never_invent_an_authored_selection() {
    let symbols = outline("<template><table><tr><td>value</td></tr></table></template>");
    let table = &symbols[0].children.as_ref().unwrap()[0];
    assert_eq!(table.name, "table");
    let row = &table.children.as_ref().unwrap()[0];
    assert_eq!(row.name, "tr");
    assert_eq!(row.children.as_ref().unwrap()[0].name, "td");
    assert_eq!(row.selection_range.start, Position::new(0, 18));
    assert_eq!(row.selection_range.end, Position::new(0, 20));
}

#[test]
fn utf16_and_crlf_positions_retain_original_document_coordinates() {
    let symbols = outline(
        "<script setup>const emoji = '😀'; const 名 = 1;</script>\r\n<template>😀<Comp>é<span></span></Comp></template>",
    );
    let scripts = symbols
        .iter()
        .find(|symbol| symbol.name == "script setup")
        .unwrap();
    let binding = &scripts.children.as_ref().unwrap()[1];
    assert_eq!(binding.selection_range.start, Position::new(0, 40));
    assert_eq!(binding.selection_range.end, Position::new(0, 41));
    let component = &symbols[0].children.as_ref().unwrap()[0];
    assert_eq!(component.selection_range.start, Position::new(1, 13));
    assert_eq!(component.selection_range.end, Position::new(1, 17));
}

#[test]
fn failed_and_empty_content_keep_the_block_without_inventing_children() {
    for source in [
        "<script setup>const = ;</script>",
        "<template><div><span></div></template>",
        "<script setup></script>",
    ] {
        assert!(outline(source).iter().all(|block| block.children.is_none()));
    }
    assert!(outline("").is_empty());
}

#[test]
fn art_markup_uses_original_html_spans_and_pug_remains_explicitly_block_only() {
    let art =
        outline("<template lang=\"art\"><section><Widget>{{value}}</Widget></section></template>");
    assert_eq!(art[0].children.as_ref().unwrap()[0].name, "section");
    assert_eq!(art[0].detail.as_deref(), Some("art"));
    let pug = outline(
        "<template lang=\"pug\">\nsection\n  Widget(ref='item')\n</template><script setup>const item = 1;</script>",
    );
    assert_eq!(pug[0].detail.as_deref(), Some("pug"));
    assert!(pug[0].children.is_none());
    assert_eq!(pug[1].children.as_ref().unwrap()[0].name, "item");
}
