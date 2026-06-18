use std::fs;
use std::path::{Path, PathBuf};

use vize_canon::VirtualProject;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

fn assert_ts_parses(source: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
    assert!(
        parsed.errors.is_empty(),
        "virtual TS should parse without errors: {:?}",
        parsed.errors
    );
}

#[test]
fn normal_script_named_value_exports_are_module_exports() {
    let case_dir = unique_case_dir("plain-script-named-exports");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("ParseMdFileDialog.vue");
    let vue_content = r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "ParseMdFileDialog",
});

export const setupParseMdFileDialogCtx = () => ({ ready: true });
</script>
"#;
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();
    let content = project
        .find_by_original(&vue_path)
        .unwrap()
        .content
        .as_str();

    assert!(
        content.contains(
            "export const setupParseMdFileDialogCtx = __vize_plain_script_exports.setupParseMdFileDialogCtx;"
        ),
        "normal <script> named exports must stay available from the virtual module:\n{content}"
    );
    assert_ts_parses(content);

    let _ = fs::remove_dir_all(&case_dir);
}
