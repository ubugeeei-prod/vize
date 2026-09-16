use super::super::parse_cli_diagnostics;
use crate::batch::VirtualProject;
use crate::batch::executor::diagnostics::DiagnosticMapper;

#[test]
fn cli_mapping_preserves_shadowed_assignment_errors() {
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
    let mut output = String::new();
    for (offset, _) in file.content.match_indices("target[key") {
        use std::fmt::Write;
        let before = &file.content[..offset];
        let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
        writeln!(
            output,
            "{}({line},{column}): error TS2322: Type 'boolean' is not assignable to type 'never'.",
            file.virtual_path.display()
        )
        .unwrap();
    }
    let mut diagnostics = Vec::new();
    let mut mapper = DiagnosticMapper::new(&project);
    parse_cli_diagnostics(&output, &project, &mut mapper, &mut diagnostics);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].file, path);
    assert_eq!(diagnostics[0].line, 7);
    assert_eq!(diagnostics[0].column, 2);
    assert_eq!(diagnostics[0].code, Some(2322));
}
