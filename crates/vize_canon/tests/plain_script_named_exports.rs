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
        parsed.diagnostics.is_empty(),
        "virtual TS should parse without errors: {:?}",
        parsed.diagnostics
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

#[test]
fn normal_script_value_and_type_declarations_are_exported_in_both_spaces() {
    let case_dir = unique_case_dir("plain-script-value-and-type-exports");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("DiffViewer.vue");
    let vue_content = r#"<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "DiffViewer",
});

export enum DiffDisplayMode {
  Unified = 'unified',
  Split = 'split',
}

export const enum DiffMarker {
  Added = 'added',
}

export class DiffCursor {
  line = 0;
}

export const pageSize = 20;

export function toPageCount(total: number) {
  return total;
}
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

    // An `enum`/`class` declares a value *and* a type. Bridging only the value
    // out of `__setup()` is what made consumers hit TS2749.
    for (value, type_side) in [
        (
            "export const DiffDisplayMode = __vize_plain_script_exports.DiffDisplayMode;",
            "export type DiffDisplayMode = (typeof DiffDisplayMode)[keyof typeof DiffDisplayMode];",
        ),
        (
            "export const DiffMarker = __vize_plain_script_exports.DiffMarker;",
            "export type DiffMarker = (typeof DiffMarker)[keyof typeof DiffMarker];",
        ),
        (
            "export const DiffCursor = __vize_plain_script_exports.DiffCursor;",
            "export type DiffCursor = InstanceType<typeof DiffCursor>;",
        ),
    ] {
        assert!(
            content.contains(value),
            "value side must stay available: {value}\n{content}"
        );
        assert!(
            content.contains(type_side),
            "type side must stay available: {type_side}\n{content}"
        );
    }

    // A value-only declaration must not be handed a type meaning it never had.
    for (value, absent_type_side) in [
        (
            "export const pageSize = __vize_plain_script_exports.pageSize;",
            "export type pageSize ",
        ),
        (
            "export const toPageCount = __vize_plain_script_exports.toPageCount;",
            "export type toPageCount ",
        ),
    ] {
        assert!(
            content.contains(value),
            "value-only export must stay available: {value}\n{content}"
        );
        assert!(
            !content.contains(absent_type_side),
            "value-only export must not gain a type export: {absent_type_side}\n{content}"
        );
    }

    assert_ts_parses(content);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn normal_script_exported_type_body_stays_intact_with_exported_const_typeof() {
    let case_dir = unique_case_dir("plain-script-exported-type-body");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("PreviewMoshiQuestionCard.vue");
    let vue_content = r#"<script lang="ts">
import { defineComponent, PropType } from "@nuxtjs/composition-api";

export const DATA_TYPE = {
  QUESTION: "question",
  ANSWER: "answer",
} as const;

export type RenderedBody = {
  dataType: (typeof DATA_TYPE)[keyof typeof DATA_TYPE];
  data: string;
};

export default defineComponent({
  props: {
    renderedBodies: {
      type: Array as PropType<RenderedBody[]>,
      default: "",
    },
  },
});
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
            "export type RenderedBody = {\n  dataType: (typeof DATA_TYPE)[keyof typeof DATA_TYPE];\n  data: string;\n};"
        ),
        "normal <script> exported type body must remain syntactically intact:\n{content}"
    );
    assert!(
        content.contains("export const DATA_TYPE = __vize_plain_script_exports.DATA_TYPE;"),
        "normal <script> exported const must remain visible to module exports:\n{content}"
    );
    assert_ts_parses(content);

    let _ = fs::remove_dir_all(&case_dir);
}
