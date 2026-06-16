use super::{is_cli_diagnostic_line, is_global_diagnostic_line, parse_cli_diagnostics};
use crate::batch::VirtualProject;
use crate::batch::executor::diagnostics::DiagnosticMapper;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(&*cstr!(
            "cli-fallback-{name}-{}-{case_id}",
            std::process::id()
        ))
}

#[test]
fn partitions_vue_files_and_shares_program_wide_sources() {
    use super::partition_virtual_files;

    let case_dir = unique_case_dir("shard-partition");
    let _ = fs::remove_dir_all(&case_dir);
    let src = case_dir.join("src");
    fs::create_dir_all(&src).unwrap();
    for index in 0..4 {
        fs::write(
            src.join(cstr!("Comp{index}.vue").as_str()),
            "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>",
        )
        .unwrap();
    }
    // A Vue file whose script augments the program must stay shared.
    fs::write(
        src.join("Augment.vue"),
        "<script lang=\"ts\">declare global { interface Window { __x?: number } }\nexport default {}</script>",
    )
    .unwrap();
    fs::write(src.join("util.ts"), "export const util = 1;\n").unwrap();

    let mut project = crate::batch::VirtualProject::new(&case_dir).unwrap();
    let paths: Vec<_> = fs::read_dir(&src)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    project.register_paths(&paths).unwrap();

    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    // 4 Vue files and the unimported util.ts partition across both shards.
    assert_eq!(plan.owners.len(), 5);
    assert!(plan.owners.values().any(|&shard| shard == 0));
    assert!(plan.owners.values().any(|&shard| shard == 1));
    let util_owner = plan.owners.get(&src.join("util.ts")).copied();
    assert!(util_owner.is_some(), "plain sources are partitioned too");
    // The augmenting .vue is included in every shard.
    for shard in &plan.shards {
        assert!(
            shard
                .iter()
                .any(|path| path.to_string_lossy().ends_with("Augment.vue.ts")),
            "program-wide augmentations must be visible to every shard"
        );
    }

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn shards_along_import_graph_components() {
    use super::partition_virtual_files;

    let case_dir = unique_case_dir("shard-components");
    let _ = fs::remove_dir_all(&case_dir);
    let src = case_dir.join("src");
    fs::create_dir_all(&src).unwrap();
    // A imports B (one component); C and D are isolated.
    fs::write(
        src.join("A.vue"),
        "<script setup lang=\"ts\">import B from './B.vue'\nvoid B</script><template><B /></template>",
    )
    .unwrap();
    for name in ["B", "C", "D"] {
        fs::write(
            src.join(cstr!("{name}.vue").as_str()),
            "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>",
        )
        .unwrap();
    }

    let mut project = crate::batch::VirtualProject::new(&case_dir).unwrap();
    let paths: Vec<_> = fs::read_dir(&src)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    project.register_paths(&paths).unwrap();

    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    // Import-connected files must land in the same shard so no shard
    // re-checks another shard's Vue files through transitive loading.
    let owner_a = plan.owners.get(&src.join("A.vue")).copied();
    let owner_b = plan.owners.get(&src.join("B.vue")).copied();
    assert!(owner_a.is_some());
    assert_eq!(owner_a, owner_b);

    // A dominant connected component degrades to an unsharded run: link
    // C into the A/B component so only D stays separate (3 vs 1 files).
    fs::write(
        src.join("C.vue"),
        "<script setup lang=\"ts\">import A from './A.vue'\nvoid A</script><template><A /></template>",
    )
    .unwrap();
    let mut project = crate::batch::VirtualProject::new(&case_dir).unwrap();
    let paths: Vec<_> = fs::read_dir(&src)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    project.register_paths(&paths).unwrap();
    let plan = partition_virtual_files(&project, 2);
    assert!(
        plan.shards.is_empty(),
        "a dominant component must collapse to a single program"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn recognizes_global_and_positioned_diagnostic_lines() {
    assert!(is_global_diagnostic_line(
        "error TS2688: Cannot find type definition file for 'vite/client'."
    ));
    assert!(is_global_diagnostic_line("warning TS1: w"));
    assert!(!is_global_diagnostic_line(
        "  The file is in the program because:"
    ));
    assert!(!is_global_diagnostic_line("error: missing argument"));
    assert!(!is_global_diagnostic_line("error TSX: nope"));

    assert!(is_cli_diagnostic_line(
        "src/App.vue.ts(3,7): error TS2322: Type 'string' is not assignable to type 'number'."
    ));
    assert!(!is_cli_diagnostic_line(
        "error TS2688: Cannot find type definition file for 'vite/client'."
    ));
}

#[test]
fn parses_cli_diagnostics_back_to_original_files() {
    let case_dir = unique_case_dir("diagnostics");
    let _ = fs::remove_dir_all(&case_dir);
    let source = case_dir.join("src").join("main.ts");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "const value: number = 'x';\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&source).unwrap();
    project.materialize().unwrap();

    let output = cstr!(
        "{}(1,7): error TS2322: Type 'string' is not assignable to type 'number'.",
        project.virtual_root().join("src").join("main.ts").display()
    );
    let mut diagnostics = Vec::new();
    let mut mapper = DiagnosticMapper::new(&project);
    parse_cli_diagnostics(output.as_str(), &project, &mut mapper, &mut diagnostics);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].file, source);
    assert_eq!(diagnostics[0].line, 0);
    assert_eq!(diagnostics[0].column, 6);
    assert_eq!(diagnostics[0].code, Some(2322));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn parses_cli_diagnostics_with_relative_dotdot_paths() {
    let case_dir = unique_case_dir("diagnostics-dotdot");
    let _ = fs::remove_dir_all(&case_dir);
    let source = case_dir.join("src").join("main.ts");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "const value: number = 'x';\n").unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&source).unwrap();
    project.materialize().unwrap();

    let output =
        "src/../src/main.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'.";
    let mut diagnostics = Vec::new();
    let mut mapper = DiagnosticMapper::new(&project);
    parse_cli_diagnostics(output, &project, &mut mapper, &mut diagnostics);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].file, source);
    assert_eq!(diagnostics[0].line, 0);
    assert_eq!(diagnostics[0].column, 6);
    assert_eq!(diagnostics[0].code, Some(2322));

    let _ = fs::remove_dir_all(&case_dir);
}
