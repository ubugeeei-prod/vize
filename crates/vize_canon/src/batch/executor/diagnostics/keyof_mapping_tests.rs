//! The keyof-indexed-assignment rule through `vize check`'s session path.

use crate::{
    batch::{
        Diagnostic, SfcBlockType, VirtualProject,
        executor::diagnostics::{LineIndex, map_batch_diagnostics},
    },
    corsa_client::{LspDiagnostic, LspPosition, LspRange},
};
use vize_carton::cstr;

fn ts2322_at(content: &str, offset: usize) -> LspDiagnostic {
    let (line, character) = LineIndex::new(content)
        .offset_to_line_col(content, offset as u32)
        .unwrap();
    LspDiagnostic {
        range: LspRange {
            start: LspPosition { line, character },
            end: LspPosition {
                line,
                character: character + 6,
            },
        },
        severity: Some(1),
        code: Some(serde_json::json!(2322)),
        source: Some("ts".into()),
        message: "Type 'boolean' is not assignable to type 'never'.".into(),
    }
}

#[test]
fn session_mapping_drops_the_keyof_shape_and_keeps_shadowed_errors() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("App.vue");
    let source = r#"<script setup lang="ts">
type A = { text: string; count: number };
declare const target: A;
declare const key: string;
const value = null as unknown as A[keyof A];
target[key as keyof A] = value;
function assign(value: boolean) {
  target[key as keyof A] = value;
}
</script><template><div /></template>"#;
    std::fs::write(&path, source).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_path(&path).unwrap();
    let file = project.find_by_original(&path).unwrap();
    let diagnostics: Vec<_> = file
        .content
        .match_indices("target[key")
        .map(|(offset, _)| ts2322_at(&file.content, offset))
        .collect();
    assert_eq!(diagnostics.len(), 2);
    let uri = cstr!("file://{}", file.virtual_path.display());

    let mapped: Vec<_> = map_batch_diagnostics(vec![(uri, diagnostics)], &project)
        .into_iter()
        .map(row)
        .collect();
    assert_eq!(
        mapped,
        [(
            path,
            7,
            2,
            "Type 'boolean' is not assignable to type 'never'.".into(),
            Some(2322),
            1,
            Some(SfcBlockType::ScriptSetup),
        )]
    );
}

type Row = (
    std::path::PathBuf,
    u32,
    u32,
    vize_carton::String,
    Option<u32>,
    u8,
    Option<SfcBlockType>,
);

fn row(diagnostic: Diagnostic) -> Row {
    (
        diagnostic.file,
        diagnostic.line,
        diagnostic.column,
        diagnostic.message,
        diagnostic.code,
        diagnostic.severity,
        diagnostic.block_type,
    )
}

#[test]
fn the_rule_reads_the_registered_project_snapshot() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("App.vue");
    for (declaration, dropped) in [
        ("const value = null as unknown as A[keyof A];", true),
        ("const value = true;", false),
    ] {
        let source = cstr!(
            "<script setup lang=\"ts\">\ntype A = {{ text: string; count: number }};\n\
             declare const target: A;\ndeclare const key: string;\n{declaration}\n\
             target[key as keyof A] = value;\n</script><template><div /></template>"
        );
        std::fs::write(&path, source.as_str()).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&path).unwrap();
        let file = project.find_by_original(&path).unwrap();
        let offset = file.content.find("target[key").unwrap();
        let uri = cstr!("file://{}", file.virtual_path.display());
        let mapped = map_batch_diagnostics(
            vec![(uri, vec![ts2322_at(&file.content, offset)])],
            &project,
        );
        assert_eq!(mapped.len(), usize::from(!dropped), "{declaration}");
    }
}
