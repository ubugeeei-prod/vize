use crate::{
    batch::{
        VirtualProject,
        executor::diagnostics::{DiagnosticMapper, LineIndex},
    },
    corsa_client::{LspDiagnostic, LspPosition, LspRange},
};

#[test]
fn lsp_mapping_keeps_shadowed_errors_and_reuses_the_file_index() {
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
    let index = LineIndex::new(&file.content);
    let positions: Vec<_> = file
        .content
        .match_indices("target[key")
        .map(|(offset, _)| {
            index
                .offset_to_line_col(&file.content, offset as u32)
                .unwrap()
        })
        .collect();
    assert_eq!(positions.len(), 2);
    let mut mapper = DiagnosticMapper::new(&project);
    assert!(mapper.keyof_assignments.is_empty());
    for _ in 0..20 {
        for (i, &(line, character)) in positions.iter().enumerate() {
            let diagnostic = LspDiagnostic {
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
            };
            let mapped = mapper.map_lsp_diagnostic(&file.virtual_path, diagnostic);
            if i == 0 {
                assert!(mapped.is_none());
            } else {
                let mapped = mapped.expect("the shadowing parameter error must survive mapping");
                assert_eq!(mapped.file, path);
                assert_eq!(mapped.line, 7);
                assert_eq!(mapped.column, 2);
                assert_eq!(mapped.code, Some(2322));
            }
        }
        assert_eq!(mapper.keyof_assignments.len(), 1);
    }
    assert_eq!(
        mapper.keyof_assignments[&file.virtual_path].offsets.len(),
        1
    );
}

#[test]
fn cached_indexes_are_scoped_to_the_immutable_project_snapshot() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("App.vue");
    for (declaration, expected) in [
        ("const value = null as unknown as A[keyof A];", true),
        ("const value = true;", false),
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\ntype A = {{ text: string; count: number }};\n\
             declare const target: A;\ndeclare const key: string;\n{declaration}\n\
             target[key as keyof A] = value;\n</script><template><div /></template>"
        );
        std::fs::write(&path, source).unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.register_path(&path).unwrap();
        let file = project.find_by_original(&path).unwrap();
        let offset = file.content.find("target[key").unwrap() as u32;
        let (line, column) = LineIndex::new(&file.content)
            .offset_to_line_col(&file.content, offset)
            .unwrap();
        let mut mapper = DiagnosticMapper::new(&project);
        assert_eq!(
            mapper.is_keyof_indexed_assignment(&file.virtual_path, line, column),
            expected
        );
    }
}
