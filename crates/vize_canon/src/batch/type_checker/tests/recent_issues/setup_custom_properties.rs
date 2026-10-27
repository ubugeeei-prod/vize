//! A member a global mixin or plugin declares through
//! `ComponentCustomProperties` resolves in a `<script setup>` template with its
//! declared type, as it does on `vue-tsc`'s template context; a name nothing
//! declares keeps the instance diagnostic, once per authored occurrence.

use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use vize_l0::{String, cstr};

/// A global mixin's members, declared the way a plugin declares them.
const AUGMENTATION: &str = "import 'vue';\n\
declare module 'vue' {\n\
  interface ComponentCustomProperties {\n\
    isSP: boolean;\n\
    appName: string;\n\
  }\n\
}\n";

const BOX: &str = r#"<template>
  <div><slot name="text" /></div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ name: 'Box' })
</script>
"#;

const SETUP: &str = r#"<script setup lang="ts">
import Box from './Box.vue'
const label = 'x'
</script>

<template>
  <span :title="isSP ? 'a' : 'b'">{{ appName.toUpperCase() }} {{ label }}</span>
  <Box><template #text><i>{{ isSP }}</i></template></Box>
  <b>{{ isSP.toFixed() }}</b>
  <u>{{ missing }} {{ missing }}</u>
</template>
"#;

const OPTIONS: &str = r#"<template>
  <div>
    <span>{{ appName.toUpperCase() }}</span>
    <Box><template #text><i v-for="i in (isSP ? 5 : 4)" :key="i">x</i></template></Box>
    <b>{{ missing }}</b>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import Box from './Box.vue'

export default defineComponent({ components: { Box } })
</script>
"#;

// Separate inputs preserve the original three-diagnostic regression above.
const MEMBER_AUGMENTATION: &str = r#"import 'vue';
declare module 'vue' {
  interface ComponentCustomProperties {
    isSP: boolean;
    appName: string;
    opaque: unknown;
    dynamic: any;
    $service: { ping(value: string): number };
  }
}
"#;

const MEMBER_SETUP: &str = r#"<script setup lang="ts">
const label = 'x'
</script>

<template>
  <span :title="isSP ? 'a' : 'b'">{{ appName.toUpperCase() }} {{ label }}</span>
  <b>{{ isSP.toFixed() }}</b>
  <b>{{ appName.missingStringMember() }}</b>
  <i>{{ opaque.member }}</i>
  <u>{{ dynamic.deep.call() }}</u>
  <p>{{ $service.ping(123) }}</p>
  <u>{{ missing }} {{ missing }}</u>
</template>
"#;

#[test]
fn custom_property_member_types_and_native_type_queries_remain_exact() {
    let Some(native) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required public-instance type queries cannot skip the native runtime"
        );
        return;
    };
    let project_root = create_project_case(
        "setup-custom-property-member-types",
        &[
            ("src/mixin.d.ts", MEMBER_AUGMENTATION),
            ("src/Members.vue", MEMBER_SETUP),
        ],
    );
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &project_root,
        BatchTypeCheckerOptions::default(),
        Some(&native),
    )
    .expect("member checker should initialize");
    checker.scan_project().expect("member project should scan");
    let checked = checker
        .check_project()
        .expect("member project should check");
    let mut diagnostics: Vec<_> = checked
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    diagnostics.sort();
    let projected = checker.virtual_files();
    let projected = projected
        .iter()
        .find(|file| file.original_path.ends_with("Members.vue"))
        .expect("actual Members.vue projection");
    let generated = std::fs::read_to_string(&projected.virtual_path)
        .expect("materialized production projection");
    assert_eq!(generated, projected.content.as_str());
    let (materialized_root, config) = checker.native_project_paths_for_test();
    let whole_diagnostics: Vec<_> = checked
        .diagnostics
        .iter()
        .map(|diagnostic| {
            serde_json::json!({
                "file": diagnostic.file,
                "line": diagnostic.line,
                "column": diagnostic.column,
                "message": diagnostic.message,
                "code": diagnostic.code,
                "severity": diagnostic.severity,
                "blockType": diagnostic.block_type.as_ref().map(|block| cstr!("{block:?}")),
            })
        })
        .collect();
    let packet = serde_json::json!({
        "native": native,
        "wholeAuthoredSource": MEMBER_SETUP,
        "wholeAugmentation": MEMBER_AUGMENTATION,
        "virtualPath": projected.virtual_path,
        "wholeGeneratedSource": generated,
        "configPath": config,
        "wholeConfig": std::fs::read_to_string(&config).expect("materialized config"),
        "wholeDiagnostics": whole_diagnostics,
    });
    retain_member_packet("before-type-queries", &packet);
    let file_wire = projected.virtual_path.display().to_string();
    let config_wire = config.display().to_string();
    let types = block_on(async {
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(&native)
                .with_mode(ApiMode::AsyncJsonRpcStdio)
                .with_cwd(&materialized_root),
            &config_wire,
            None,
        )
        .await
        .expect("actual materialized member project session should initialize");
        let mut types = Vec::new();
        for (name, expected) in [
            ("isSP", "boolean"),
            ("appName", "string"),
            ("opaque", "unknown"),
            ("dynamic", "any"),
        ] {
            let declaration_marker = cstr!("var {name}!:");
            let instance_marker = cstr!("__vize_public_instance.{name}");
            let declaration = generated
                .find(declaration_marker.as_str())
                .map(|start| start + "var ".len())
                .expect("actual typed lexical declaration");
            let instance_read = generated
                .find(instance_marker.as_str())
                .map(|start| start + "__vize_public_instance.".len())
                .expect("actual instance member read");
            for (kind, offset) in [("lexical", declaration), ("instance", instance_read)] {
                let position = u32::try_from(offset).expect("native query position");
                let record = session
                    .get_type_at_position(file_wire.as_str(), position)
                    .await
                    .expect("native member type query should complete")
                    .expect("native member type should exist");
                let whole_record = cstr!("{record:#?}");
                let rendered = session
                    .type_to_string(record.id, None, None)
                    .await
                    .expect("native member type should render");
                types.push(serde_json::json!({
                    "name": name,
                    "kind": kind,
                    "position": position,
                    "wholeRecord": whole_record,
                    "type": rendered,
                    "expected": expected,
                }));
            }
        }
        session
            .close()
            .await
            .expect("member type session should close");
        types
    });
    retain_member_packet("type-queries", &serde_json::json!({"queries": types}));
    for query in &types {
        assert_eq!(query.get("type"), query.get("expected"), "{query}");
    }
    let mut expected: Vec<(String, Option<u32>, String)> = vec![
        (
            "src/Members.vue".into(),
            Some(2339),
            "7:14: Property 'toFixed' does not exist on type 'boolean'.".into(),
        ),
        (
            "src/Members.vue".into(),
            Some(2339),
            "8:17: Property 'missingStringMember' does not exist on type 'string'.".into(),
        ),
        (
            "src/Members.vue".into(),
            Some(18046),
            "9:9: 'opaque' is of type 'unknown'.".into(),
        ),
        (
            "src/Members.vue".into(),
            Some(2345),
            "11:23: Argument of type 'number' is not assignable to parameter of type 'string'."
                .into(),
        ),
        (
            "src/Members.vue".into(),
            Some(2339),
            "12:9: Property 'missing' does not exist on the component instance.".into(),
        ),
        (
            "src/Members.vue".into(),
            Some(2339),
            "12:23: Property 'missing' does not exist on the component instance.".into(),
        ),
    ];
    expected.sort();
    assert_eq!(diagnostics, expected);
    drop(checker);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_member_mutation_matrix();
}

fn assert_member_mutation_matrix() {
    const CALLS: &str = r#"<script setup lang="ts">
function booleanOnly(value: boolean) { return value }
function stringOnly(value: string) { return value }
</script>
<template>
<p>{{ booleanOnly(isSP) }}</p>
<p>{{ stringOnly(isSP) }}</p>
<p>{{ missing }}</p>
</template>
"#;
    for (member_type, errors) in [
        ("boolean", vec![(7, 18, "string")]),
        ("string", vec![(6, 19, "boolean")]),
        ("unknown", vec![(6, 19, "boolean"), (7, 18, "string")]),
        ("any", vec![]),
    ] {
        let augmentation =
            MEMBER_AUGMENTATION.replace("isSP: boolean;", &cstr!("isSP: {member_type};"));
        let project_root = create_project_case(
            &cstr!("setup-custom-property-mutation-{member_type}"),
            &[
                ("src/mixin.d.ts", &augmentation),
                ("src/Mutation.vue", CALLS),
            ],
        );
        let actual = project_diagnostics(&project_root, false);
        retain_member_packet(
            &cstr!("mutation-{member_type}"),
            &serde_json::json!({
                "wholeAuthoredSource": CALLS,
                "wholeAugmentation": augmentation,
                "exactDiagnosticSnapshot": actual,
            }),
        );
        let mut expected: Vec<(String, Option<u32>, String)> = vec![(
            "src/Mutation.vue".into(),
            Some(2339),
            "8:7: Property 'missing' does not exist on the component instance.".into(),
        )];
        for (line, column, parameter_type) in errors {
            expected.push((
                "src/Mutation.vue".into(),
                Some(2345),
                cstr!("{line}:{column}: Argument of type '{member_type}' is not assignable to parameter of type '{parameter_type}'."),
            ));
        }
        expected.sort();
        assert_eq!(
            actual, expected,
            "member declaration mutation: {member_type}"
        );
        let _ = std::fs::remove_dir_all(&project_root);
    }
}

fn retain_member_packet(name: &str, packet: &serde_json::Value) {
    eprintln!("public-instance member packet {name}={packet}");
    if let Some(directory) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(directory).join("public-instance-member-types");
        std::fs::create_dir_all(&directory).expect("member packet directory");
        std::fs::write(
            directory.join(cstr!("{name}.json").as_str()),
            serde_json::to_vec_pretty(packet).expect("complete member packet"),
        )
        .expect("retain complete member packet");
    }
}

#[test]
fn custom_properties_resolve_in_script_setup_templates() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "setup-custom-properties",
        &[
            ("src/mixin.d.ts", AUGMENTATION),
            ("src/Box.vue", BOX),
            ("src/Setup.vue", SETUP),
        ],
    );
    let diagnostics = project_diagnostics(&project_root, false);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        diagnostics,
        vec![
            (
                "src/Setup.vue".into(),
                Some(2339),
                "10:23: Property 'missing' does not exist on the component instance.".into()
            ),
            (
                "src/Setup.vue".into(),
                Some(2339),
                "10:9: Property 'missing' does not exist on the component instance.".into()
            ),
            (
                "src/Setup.vue".into(),
                Some(2339),
                "9:14: Property 'toFixed' does not exist on type 'boolean'.".into()
            ),
        ]
    );
}

#[test]
fn custom_properties_resolve_in_options_api_templates() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "options-custom-properties",
        &[
            ("src/mixin.d.ts", AUGMENTATION),
            ("src/Box.vue", BOX),
            ("src/Options.vue", OPTIONS),
        ],
    );
    let diagnostics = project_diagnostics(&project_root, true);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        diagnostics,
        vec![(
            "src/Options.vue".into(),
            Some(2339),
            "5:11: Property 'missing' does not exist on type 'ComponentPublicInstance<{}>'.".into()
        )]
    );
}

/// The caller already gated on a resolvable checker binary, so every failure
/// past that point is a real regression and panics instead of skipping.
fn project_diagnostics(
    project_root: &Path,
    options_api: bool,
) -> Vec<(String, Option<u32>, String)> {
    let mut checker =
        BatchTypeChecker::with_options(project_root, BatchTypeCheckerOptions::default())
            .expect("batch checker should initialize");
    if options_api {
        checker.enable_options_api();
    }
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}

/// Full native diagnostics prove both repeated missing reads remain errors,
/// while the current loop alias cannot hide an augmented iterable property.
#[test]
fn loop_sources_keep_authored_reads_and_parent_scope() {
    if resolve_test_tsgo_binary().is_none() {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "native loop-source regression requires the checker"
        );
        return;
    }
    let augmentation = r#"import 'vue';
declare module 'vue' { interface ComponentCustomProperties { item: number[] } }
"#;
    let source = r#"<script setup lang="ts">
const label = 'x'
</script>
<template>
  <i v-for="item in item" :key="item">{{ item.toFixed() }}</i>
  <i v-for="entry in (missing || missing)" :key="entry">{{ entry }}</i>
</template>
"#;
    let root = create_project_case(
        "loop-source-authored-reads",
        &[("src/mixin.d.ts", augmentation), ("src/Loop.vue", source)],
    );
    let diagnostics = project_diagnostics(&root, false);
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/Loop.vue".into(),
                Some(2339),
                "6:23: Property 'missing' does not exist on the component instance.".into()
            ),
            (
                "src/Loop.vue".into(),
                Some(2339),
                "6:34: Property 'missing' does not exist on the component instance.".into()
            ),
        ]
    );
    let _ = std::fs::remove_dir_all(&root);
}
