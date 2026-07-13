use std::fs;

use vize_atelier_sfc::SfcScriptSyntaxProduct;
use vize_atlas::ProductStatus;
use vize_carton::String as CompactString;
use vize_croquis::CroquisDocumentProduct;
use vize_flow::FlowProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::{ReliefProduct, TransformedReliefProduct};

use super::*;
use crate::batch::ImportRewriter;
use crate::virtual_ts::VirtualTsOptions;

#[path = "tests/fallback.rs"]
mod fallback;

fn case(name: &str) -> (PathBuf, PathBuf) {
    static NEXT_CASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/vize-tests/tests")
        .join(format!(
            "{name}-{}-{}",
            std::process::id(),
            NEXT_CASE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    (root, src)
}

fn source(path: PathBuf, content: &str) -> RegisteredSource {
    RegisteredSource {
        path,
        content: content.into(),
        source_type: None,
    }
}

fn direct_vue(
    path: &Path,
    content: &str,
    options: super::super::VueDocumentVirtualTsOptions,
) -> CompactString {
    super::super::generate_vue_document_virtual_ts_with_options(
        path,
        content,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        true,
        options,
    )
    .unwrap()
    .code
}

#[test]
fn vue_recipe_executes_one_shared_descriptor_relief_and_croquis() {
    crate::virtual_ts::reset_authored_script_fallback_parse_invocations();
    let (root, src) = case("atlas-canon-counters");
    let path = src.join("Counter.vue");
    let content = "<script setup>const value = 1</script><template>{{ value }}</template>";
    let project = VirtualProject::new(&root).unwrap();
    let (compilation, sources) = prepare_compilation(&project, &[source(path, content)]).unwrap();
    let snapshot = compilation.snapshot();
    let mut session = snapshot.query_session();
    let outcome = session
        .query::<CanonTypedDocumentProduct>(sources[0])
        .unwrap();

    assert_eq!(outcome.status(), ProductStatus::Executed);
    assert!(outcome.plan().contains::<SfcDescriptorProduct>());
    assert!(outcome.plan().contains::<ReliefProduct>());
    assert!(outcome.plan().contains::<CroquisDocumentProduct>());
    assert!(outcome.plan().contains::<SfcScriptSyntaxProduct>());
    assert!(outcome.plan().contains::<ModuleSyntaxProduct>());
    assert!(!outcome.plan().contains::<TransformedReliefProduct>());
    assert!(!outcome.plan().contains::<FlowProduct>());
    assert_eq!(
        session
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        crate::virtual_ts::authored_script_fallback_parse_invocations(),
        0
    );
    assert_eq!(
        session
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<SfcScriptSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        session
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
    session.query::<SfcDescriptorProduct>(sources[0]).unwrap();
    assert_eq!(
        session
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .cache_hits(),
        1
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vue_recipe_shapes_keep_script_only_relief_free_and_template_only_module_free() {
    let cases = [
        (
            "script-only",
            "<script lang=\"ts\">export { ref } from 'vue'</script>",
            false,
            true,
        ),
        (
            "template-only",
            "<template><p>hello</p></template>",
            true,
            false,
        ),
    ];
    for (name, content, has_relief, has_module) in cases {
        let (root, src) = case(name);
        let path = src.join("Shape.vue");
        let project = VirtualProject::new(&root).unwrap();
        let (compilation, sources) =
            prepare_compilation(&project, &[source(path, content)]).unwrap();
        let snapshot = compilation.snapshot();
        let mut session = snapshot.query_session();
        let outcome = session
            .query::<CanonTypedDocumentProduct>(sources[0])
            .unwrap();

        outcome.value().to_corsa_result().unwrap();
        assert_eq!(
            outcome.plan().contains::<ReliefProduct>(),
            has_relief,
            "{name}"
        );
        assert_eq!(
            outcome.plan().contains::<ModuleSyntaxProduct>(),
            has_module,
            "{name}"
        );
        assert_eq!(
            session
                .counters()
                .for_product::<ReliefProduct>()
                .executions(),
            u64::from(has_relief),
            "{name}"
        );
        assert_eq!(
            session
                .counters()
                .for_product::<ModuleSyntaxProduct>()
                .executions(),
            u64::from(has_module),
            "{name}"
        );
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn editor_and_socket_vue_document_path_has_no_shadow_sfc_or_template_parse() {
    let document = include_str!("../document.rs");
    let artifact_codegen = include_str!("../vue_artifact_codegen.rs");
    let diagnostics = include_str!("../diagnostics.rs");
    let server = include_str!("../../../corsa_server.rs");
    let legacy_validator = concat!("validate_script_setup_", "semantics_located");

    assert!(!document.contains("parse_sfc("));
    assert!(!document.contains("vize_armature::parse"));
    assert!(!document.contains("analyze_sfc_descriptor"));
    assert!(!artifact_codegen.contains("collect_script_parse_diagnostics"));
    assert!(!artifact_codegen.contains("oxc_parser"));
    assert!(!diagnostics.contains(legacy_validator));
    assert!(diagnostics.contains("script_syntax.validate_script_setup_semantics(source)"));
    assert!(!server.contains("parse_sfc("));
    assert!(!server.contains(legacy_validator));
}

#[test]
fn vue_scripts_and_declarations_share_one_project_source_store() {
    let (root, src) = case("atlas-canon-project-sources");
    let vue = src.join("App.vue");
    let script = src.join("main.ts");
    let declaration = src.join("env.d.ts");
    let project = VirtualProject::new(&root).unwrap();
    let inputs = [
        source(
            vue.clone(),
            "<script setup>const value = 1</script><template>{{ value }}</template>",
        ),
        source(script.clone(), "import App from './App.vue'; void App;"),
        source(declaration.clone(), "declare const ambient: string;"),
    ];
    let (compilation, identities) = prepare_compilation(&project, &inputs).unwrap();
    assert_eq!(compilation.sources().len(), 3);
    let snapshot = compilation.snapshot();
    for (identity, path) in identities.into_iter().zip([vue, script, declaration]) {
        let mut session = snapshot.query_session();
        let outcome = session
            .query::<CanonTypedDocumentProduct>(identity)
            .unwrap();
        let registered = outcome.value().to_corsa_result().unwrap();
        assert_eq!(registered.file.original_path, path);
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn options_api_and_legacy_modes_are_byte_identical_to_direct_codegen() {
    let cases = [
        (
            "atlas-canon-options-api",
            false,
            r#"<script lang="ts">
export default { data() { return { count: 1 } }, methods: { inc() { this.count++ } } }
</script><template><button @click="inc">{{ count }}</button></template>"#,
        ),
        (
            "atlas-canon-legacy",
            true,
            r#"<script lang="ts">
export default { props: { title: String }, data() { return { count: 1 } } }
</script><template><p>{{ title }} {{ count }} {{ $route.path }}</p></template>"#,
        ),
    ];
    for (name, legacy, content) in cases {
        let (root, src) = case(name);
        let path = src.join("Mode.vue");
        let mut project = VirtualProject::new(&root).unwrap();
        project.set_options_api(!legacy);
        project.set_legacy_vue2(legacy);
        project.register_vue_file(&path, content).unwrap();
        let actual = &project.find_by_original(&path).unwrap().content;
        let expected = direct_vue(
            &path,
            content,
            super::super::VueDocumentVirtualTsOptions {
                options_api: !legacy,
                legacy_vue2: legacy,
            },
        );
        assert_eq!(actual.as_str(), expected.as_str(), "{name}");
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn malformed_container_and_template_preserve_error_and_fallback_bytes() {
    let (root, src) = case("atlas-canon-malformed");
    let path = src.join("Malformed.vue");
    let container = "<template/><template/>";
    let direct_error = match super::super::generate_vue_document_virtual_ts(
        &path,
        container,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        true,
    ) {
        Ok(_) => panic!("duplicate template must fail"),
        Err(error) => error.to_string(),
    };
    let mut project = VirtualProject::new(&root).unwrap();
    let artifact_error = project
        .register_vue_file(&path, container)
        .unwrap_err()
        .to_string();
    assert_eq!(artifact_error, direct_error);

    let template = "<template><div>{{ value </div></template>";
    let expected = direct_vue(
        &path,
        template,
        super::super::VueDocumentVirtualTsOptions::default(),
    );
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_vue_file(&path, template).unwrap();
    assert_eq!(
        project.find_by_original(&path).unwrap().content.as_str(),
        expected.as_str()
    );
    assert!(!project.diagnostics().is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn imported_heritage_props_use_the_shared_resolved_croquis_without_byte_drift() {
    let (root, src) = case("atlas-canon-imported-props");
    fs::write(
        src.join("types.ts"),
        "export interface RootProps { side?: 'left' | 'right'; resizable?: boolean }",
    )
    .unwrap();
    let path = src.join("Resolved.vue");
    let content = r#"<script setup lang="ts">
import type { RootProps } from './types'
interface Props extends Pick<RootProps, 'side' | 'resizable'> { label?: string }
defineProps<Props>()
</script><template>{{ side }} {{ resizable }} {{ label }}</template>"#;
    let expected = direct_vue(
        &path,
        content,
        super::super::VueDocumentVirtualTsOptions::default(),
    );
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_vue_file(&path, content).unwrap();
    assert_eq!(
        project.find_by_original(&path).unwrap().content.as_str(),
        expected.as_str()
    );
    let _ = fs::remove_dir_all(root);
}
