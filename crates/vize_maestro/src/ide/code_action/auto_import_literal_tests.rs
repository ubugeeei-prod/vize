//! Authored POSIX auto-imports through the real shallow index and caller.

use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range,
    TextDocumentContentChangeEvent, TextEdit, Url, WorkspaceEdit,
};

use super::{CodeActionService, push_import_specifier};
use crate::ide::IdeContext;
use crate::ide::auto_import::{AutoImportEntry, AutoImportIndex, AutoImportKind};
use crate::server::ServerState;

const SOURCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/auto-import-quoted-symlink-3952/Page.vue.txt"
);
const COMPONENT: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/auto-import-quoted-symlink-3952/MyButton.vue.txt"
);

fn assert_module(source: &str, expected_specifier: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    assert!(!parsed.panicked);
    let Statement::ImportDeclaration(import) = &parsed.program.body[0] else {
        panic!(
            "expected a complete module import, got {:#?}",
            parsed.program
        );
    };
    assert_eq!(import.source.value.as_str(), expected_specifier);
}

fn expected_action(uri: &Url, specifier: &str, statement: &str) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Auto-import `MyButton` from `{specifier}`"),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range::new(Position::new(1, 0), Position::new(1, 0)),
                    new_text: statement.into(),
                }],
            )])),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

fn actions(state: &ServerState, uri: &Url, source: &str) -> Vec<CodeActionOrCommand> {
    let offset = source.find("MyButton v-if").unwrap() + 4;
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let position = Position::new(line, character);
    let ctx = IdeContext::new(state, uri, offset).unwrap();
    CodeActionService::code_actions(&ctx, Range::new(position, position))
}

fn exercise(quoted_target: bool, specifier: &str, statement: &str) {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project's-root");
    fs::create_dir(&project).unwrap();
    let directory = project.join("app");
    fs::create_dir(&directory).unwrap();
    let indexed = directory.join("MyButton.vue");
    if quoted_target {
        let target_directory = project.join("owner's-components");
        fs::create_dir(&target_directory).unwrap();
        let target = target_directory.join("MyButton.vue");
        fs::write(&target, COMPONENT).unwrap();
        std::os::unix::fs::symlink(&target, &indexed).unwrap();
        assert_eq!(
            fs::canonicalize(&indexed).unwrap(),
            fs::canonicalize(&target).unwrap()
        );
    } else {
        fs::write(&indexed, COMPONENT).unwrap();
    }
    let component_source = fs::read_to_string(&indexed).unwrap();
    assert_eq!(component_source, COMPONENT);
    let state = ServerState::new();
    let component = state
        .component_descriptor(&indexed, &component_source)
        .unwrap();
    assert_eq!(
        component.template.as_ref().unwrap().content,
        "<button>{{ caption }}</button>"
    );
    let allocator = Allocator::default();
    let component_script = component.script_setup.as_ref().unwrap();
    let parsed = Parser::new(&allocator, &component_script.content, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    assert_eq!(
        AutoImportIndex::find_in_directory(&directory, "MyButton"),
        Some(AutoImportEntry {
            name: "MyButton".into(),
            source: indexed,
            kind: AutoImportKind::VueComponent,
        }),
    );
    let file = directory.join("Page.vue");
    fs::write(&file, SOURCE).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, SOURCE);
    let original = state.documents.get(&uri).unwrap().text_snapshot();
    let actual = actions(&state, &uri, SOURCE);
    assert_eq!(actual, vec![expected_action(&uri, specifier, statement)]);
    assert!(Arc::ptr_eq(
        &original,
        &state.documents.get(&uri).unwrap().text_snapshot()
    ));
    assert_eq!(fs::read_to_string(&file).unwrap(), SOURCE);

    let CodeActionOrCommand::CodeAction(action) = &actual[0] else {
        unreachable!();
    };
    let changes = action.edit.as_ref().unwrap().changes.as_ref().unwrap();
    let edit = &changes.get(&uri).unwrap()[0];
    let start =
        crate::ide::position_to_offset(SOURCE, edit.range.start.line, edit.range.start.character)
            .unwrap();
    let mut applied = SOURCE.to_owned();
    applied.insert_str(start, &edit.new_text);
    assert_eq!(applied, SOURCE.replacen('\n', &format!("\n{statement}"), 1));
    assert!(state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: applied.clone(),
        }],
        2,
    ));
    state.update_virtual_docs(&uri, &applied);
    let ctx = IdeContext::new(&state, &uri, 0).unwrap();
    let descriptor = ctx.descriptor().unwrap();
    assert_eq!(
        descriptor.template.as_ref().unwrap().content,
        "\n  <MyButton v-if=\"enabled\" />\n"
    );
    assert_module(
        &descriptor.script_setup.as_ref().unwrap().content,
        specifier,
    );
    assert_eq!(actions(&state, &uri, &applied), Vec::new());
    assert_eq!(fs::read_to_string(&file).unwrap(), SOURCE);
    assert_eq!(
        fs::read_to_string(directory.join("MyButton.vue")).unwrap(),
        COMPONENT
    );
}

#[test]
fn quoted_common_parent_keeps_the_complete_ordinary_single_quoted_action() {
    exercise(
        false,
        "./MyButton.vue",
        "import MyButton from './MyButton.vue'\n",
    );
}

#[test]
fn indexed_symlink_to_quoted_parent_emits_a_valid_complete_import_and_requeries_cleanly() {
    // The common parent cancels from a relative path. A real indexed symlink
    // exposes the distinct quoted target through the existing canonicalization.
    exercise(
        true,
        "../owner's-components/MyButton.vue",
        "import MyButton from \"../owner's-components/MyButton.vue\"\n",
    );
}

#[test]
fn source_derived_unescaped_interpolation_is_not_a_valid_javascript_module() {
    // Syntax counterfactual of the original interpolation, not an old binary run.
    let source = "import MyButton from '../owner's-components/MyButton.vue'\n";
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(!parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
}

#[test]
fn existing_encoder_preserves_each_complete_module_literal_value() {
    for (value, expected) in [
        ("./MyButton.vue", "'./MyButton.vue'"),
        (
            "../owner's-components/MyButton.vue",
            "\"../owner's-components/MyButton.vue\"",
        ),
        ("../say\"hi/MyButton.vue", "'../say\"hi/MyButton.vue'"),
        (
            "..\\owner's\\MyButton.vue",
            "\"..\\\\owner's\\\\MyButton.vue\"",
        ),
        (
            "../line\nbreak/MyButton.vue",
            "\"../line\\nbreak/MyButton.vue\"",
        ),
        (
            "../line\rbreak/MyButton.vue",
            "\"../line\\rbreak/MyButton.vue\"",
        ),
        ("../tab\t/MyButton.vue", "\"../tab\\t/MyButton.vue\""),
        (
            "../control\u{01}/MyButton.vue",
            "\"../control\\u0001/MyButton.vue\"",
        ),
    ] {
        let mut literal = String::new();
        push_import_specifier(&mut literal, value);
        assert_eq!(literal, expected);
        assert_module(&format!("import MyButton from {literal}\n"), value);
    }
}
