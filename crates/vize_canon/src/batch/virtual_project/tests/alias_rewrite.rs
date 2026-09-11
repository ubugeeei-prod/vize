use std::fs;

use super::{VirtualProject, unique_case_dir};

fn write_vue_import_case(case: &std::path::Path, tsconfig: Option<&str>) -> std::path::PathBuf {
    write_vue_import_case_with_specifier(case, tsconfig, "@/components/api/DirectiveTable.vue")
}

fn write_vue_import_case_with_specifier(
    case: &std::path::Path,
    tsconfig: Option<&str>,
    specifier: &str,
) -> std::path::PathBuf {
    let _ = fs::remove_dir_all(case);
    fs::create_dir_all(case.join("src/components/api")).unwrap();
    if let Some(tsconfig) = tsconfig {
        fs::write(case.join("tsconfig.json"), tsconfig).unwrap();
    }
    fs::write(
        case.join("src/components/api/DirectiveTable.vue"),
        "<script setup lang=\"ts\">export interface Props { id: string }</script>",
    )
    .unwrap();
    let app = case.join("src/App.vue");
    fs::write(
        &app,
        format!(
            r#"<script setup lang="ts">
import DirectiveTable from "{specifier}";
void DirectiveTable;
</script>
"#
        ),
    )
    .unwrap();
    app
}

fn alias_tsconfig() -> &'static str {
    r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  }
}"#
}

#[test]
fn batch_sfc_keeps_unconfigured_alias_vue_import_authored() {
    let case = unique_case_dir("unconfigured-alias-vue-import");
    let app = write_vue_import_case(&case, None);

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/components/api/DirectiveTable.vue\""));
    assert!(!generated.contains("\"@/components/api/DirectiveTable.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_rewrites_configured_alias_vue_import_to_the_mirror() {
    let case = unique_case_dir("configured-alias-vue-import");
    let app = write_vue_import_case(&case, Some(alias_tsconfig()));

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/components/api/DirectiveTable.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_rewrites_configured_extensionless_alias_sfc_import_to_the_mirror() {
    let case = unique_case_dir("configured-extensionless-alias-sfc-import");
    let app = write_vue_import_case_with_specifier(
        &case,
        Some(alias_tsconfig()),
        "@/components/api/DirectiveTable",
    );

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/components/api/DirectiveTable.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_keeps_configured_extensionless_alias_ts_import_authored() {
    let case = unique_case_dir("configured-extensionless-alias-ts-import");
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(case.join("src/lib")).unwrap();
    fs::write(case.join("tsconfig.json"), alias_tsconfig()).unwrap();
    fs::write(
        case.join("src/lib/format.ts"),
        "export const format = () => '';\n",
    )
    .unwrap();
    let app = case.join("src/App.vue");
    fs::write(
        &app,
        r#"<script setup lang="ts">
import { format } from "@/lib/format";
void format;
</script>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/lib/format\""));
    assert!(!generated.contains("\"@/lib/format.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_rewrites_configured_extensionless_alias_index_sfc_import_to_the_mirror() {
    let case = unique_case_dir("configured-extensionless-alias-index-sfc-import");
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(case.join("src/panels/Card")).unwrap();
    fs::write(case.join("tsconfig.json"), alias_tsconfig()).unwrap();
    fs::write(case.join("src/panels/Card/index.vue"), "<template />").unwrap();
    let app = case.join("src/App.vue");
    fs::write(
        &app,
        r#"<script setup lang="ts">
import Card from "@/panels/Card";
void Card;
</script>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/panels/Card/index.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_keeps_extensionless_exact_alias_to_vue_authored() {
    let case = unique_case_dir("extensionless-exact-alias-to-vue");
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(case.join("src/local")).unwrap();
    fs::write(
        case.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@scope/ui": ["src/local/LocalWidget.vue"]
    }
  }
}"#,
    )
    .unwrap();
    fs::write(
        case.join("src/local/LocalWidget.vue"),
        "<script setup lang=\"ts\">defineProps<{ localOnly: string }>()</script>",
    )
    .unwrap();
    let app = case.join("src/App.vue");
    fs::write(
        &app,
        r#"<script setup lang="ts">
import Widget from "@scope/ui";
void Widget;
</script>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@scope/ui\""));
    assert!(!generated.contains("\"@scope/ui.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_keeps_extensionless_alias_with_vue_target_authored() {
    let case = unique_case_dir("extensionless-alias-with-vue-target");
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(case.join("src/local")).unwrap();
    fs::write(
        case.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@widgets/*": ["src/local/*.vue"]
    }
  }
}"#,
    )
    .unwrap();
    fs::write(
        case.join("src/local/LocalWidget.vue"),
        "<script setup lang=\"ts\">defineProps<{ localOnly: string }>()</script>",
    )
    .unwrap();
    let app = case.join("src/App.vue");
    fs::write(
        &app,
        r#"<script setup lang="ts">
import Widget from "@widgets/LocalWidget";
void Widget;
</script>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@widgets/LocalWidget\""));
    assert!(!generated.contains("\"@widgets/LocalWidget.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}

#[test]
fn batch_sfc_keeps_vue_suffixed_alias_key_import_authored() {
    let case = unique_case_dir("vue-suffixed-alias-key-import");
    let app = write_vue_import_case(
        &case,
        Some(
            r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*.vue": ["src/*.vue"]
    }
  }
}"#,
        ),
    );

    let mut project = VirtualProject::new(&case).unwrap();
    project.register_path(&app).unwrap();
    let generated = project.find_by_original(&app).unwrap().content.as_str();

    assert!(generated.contains("\"@/components/api/DirectiveTable.vue\""));
    assert!(!generated.contains("\"@/components/api/DirectiveTable.vue.ts\""));

    let _ = fs::remove_dir_all(&case);
}
