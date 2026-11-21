use super::*;
use crate::config::try_load_formatter_snapshot_from_serialized;

#[test]
#[ignore = "NUL-delimited configuration and project-relative file manifests"]
fn original_project_configurations() {
    let configs =
        std::fs::read_to_string(std::env::var_os("VIZE_PROJECT_POLICY_CONFIGS").unwrap()).unwrap();
    let paths =
        std::fs::read_to_string(std::env::var_os("VIZE_PROJECT_POLICY_FILES").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut accepted = 0;
    let mut refused = 0;
    let mut compared = 0;
    let mut scoped = 0;
    let mut ignored = 0;
    let mut preserved = 0;
    for config in configs.split_terminator('\0') {
        let path = root.join(config);
        let bytes = std::fs::read(&path).unwrap();
        // The formatter history feeds archived native JSON through its original
        // serialized-config transport while retaining the real config directory.
        let transport = path.with_file_name("serialized-config.json");
        let Ok(loaded) = try_load_formatter_snapshot_from_serialized(&bytes, transport) else {
            refused += 1;
            continue;
        };
        accepted += 1;
        let dir = path.parent().unwrap();
        let lint = FormatterLinterPlan::new(loaded.linter_plan.clone(), dir);
        let whitespace =
            CompilerWhitespacePlan::new(loaded.compiler_whitespace, &loaded.compiler_entries, dir);
        let shared = ProjectFilePlan::new(
            loaded.linter_plan,
            loaded.compiler_whitespace,
            &loaded.compiler_entries,
            dir,
        );
        scoped += usize::from(!shared.scopes.is_empty());
        for name in paths.split_terminator('\0') {
            let file = dir.join(name);
            let observed = shared.resolve(&file);
            assert_eq!(observed.lint, lint.resolve(&file), "{config} {name}");
            assert_eq!(
                observed.preserves_text,
                whitespace.preserves_text_for(&file),
                "{config} {name}"
            );
            compared += 1;
            ignored += usize::from(observed.lint.is_none());
            preserved += usize::from(observed.preserves_text);
        }
    }
    assert!(accepted > 0 && scoped > 0 && compared > 0);
    eprintln!(
        "accepted={accepted} refused={refused} scoped={scoped} compared={compared} ignored={ignored} preserved={preserved}"
    );
}
