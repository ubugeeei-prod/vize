//! Stats cache must distinguish actual output-affecting whitespace modes.

use super::super::{cache::StatsCompileCache, compile_stats::compile_file_stats_with_cache};
use super::{BuildConfigSettings, CompileFileSettings, load_build_config};
use crate::commands::build::{BuildArgs, config::CompileStats};
use vize_atelier_core::{WhitespaceStrategy, parser::with_whitespace_mode};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::config::{CompilerConfigEntry, ConfigExperimentalVueFlags, ConfigFeatureFlags};

#[test]
fn imported_type_stats_keep_distinct_project_authority() {
    let root = tempfile::tempdir().expect("projects");
    let source = "<script setup lang=\"ts\">import type { FooProps } from '@lib/FooProps'; const props = defineProps<FooProps>();</script><template><div>{{ props.bar }}</div></template>";
    let mut inputs = Vec::new();
    for (name, field_type) in [("first", "Record<string, unknown>"), ("second", "boolean")] {
        let project = root.path().join(name);
        std::fs::create_dir_all(project.join("src")).expect("source directory");
        std::fs::write(
            project.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@lib/*":["./src/*"]}}}"#,
        )
        .expect("project config");
        std::fs::write(
            project.join("src/FooProps.ts"),
            vize_l0::cstr!("export interface FooProps {{ bar?: {field_type} }}"),
        )
        .expect("type input");
        let path = project.join("Card.vue");
        std::fs::write(&path, source).expect("same source input");
        inputs.push(path);
    }
    let settings =
        CompileFileSettings::resolve(&BuildArgs::default(), load_build_config(true, None));
    let stats = CompileStats::new(2);
    let reference_first =
        compile_file_stats_with_cache(&inputs[0], &settings, &stats, &StatsCompileCache::default())
            .expect("first independent compile")
            .0;
    let reference_second =
        compile_file_stats_with_cache(&inputs[1], &settings, &stats, &StatsCompileCache::default())
            .expect("second independent compile")
            .0;
    assert_ne!(
        reference_first, reference_second,
        "different runtime constructors have different byte lengths"
    );
    let shared = StatsCompileCache::default();
    assert_eq!(
        compile_file_stats_with_cache(&inputs[0], &settings, &stats, &shared)
            .expect("first shared compile")
            .0,
        reference_first
    );
    assert_eq!(
        compile_file_stats_with_cache(&inputs[1], &settings, &stats, &shared)
            .expect("second shared compile")
            .0,
        reference_second
    );
}

#[test]
fn imported_type_stats_recover_unresolved_inputs_and_observe_type_file_edits() {
    let root = tempfile::tempdir().expect("project");
    let path = root.path().join("Card.vue");
    std::fs::create_dir_all(root.path().join("src")).expect("source directory");
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@lib/*":["./src/*"]}}}"#,
    )
    .expect("config");
    std::fs::write(&path, "<script setup lang=\"ts\">import type { FooProps } from '@lib/FooProps'; const props = defineProps<FooProps>();</script><template><div>{{ props.bar }}</div></template>").expect("SFC");
    let settings =
        || CompileFileSettings::resolve(&BuildArgs::default(), load_build_config(true, None));
    let stats = CompileStats::new(1);
    let cache = StatsCompileCache::default();
    let failure = compile_file_stats_with_cache(&path, &settings(), &stats, &cache)
        .expect_err("unresolved imported type");
    assert_eq!(
        failure.error,
        "Cannot resolve imported macro type `FooProps` from `@lib/FooProps`."
    );
    let mut lengths = Vec::new();
    for field_type in ["Record<string, unknown>", "boolean"] {
        std::fs::write(
            root.path().join("src/FooProps.ts"),
            vize_l0::cstr!("export interface FooProps {{ bar?: {field_type} }}"),
        )
        .expect("repair or edit type");
        let current = settings();
        let reference =
            compile_file_stats_with_cache(&path, &current, &stats, &StatsCompileCache::default())
                .expect("fresh independent compile")
                .0;
        let actual = compile_file_stats_with_cache(&path, &current, &stats, &cache)
            .expect("retained cache must observe a new session")
            .0;
        assert_eq!(actual, reference);
        lengths.push(actual);
    }
    assert_ne!(lengths[0], lengths[1]);
}

#[test]
fn entry_compiler_settings_apply_only_to_matching_files() {
    let project = tempfile::tempdir().unwrap();
    let legacy = project.path().join("src/legacy/Old.vue");
    let modern = project.path().join("src/New.vue");
    let config = BuildConfigSettings {
        project_root: Some(project.path().to_path_buf()),
        compiler_template_syntax: None,
        compiler_whitespace: Some("preserve"),
        features: ConfigFeatureFlags::default(),
        experimental_vue: ConfigExperimentalVueFlags::default(),
        ssr: None,
        vapor: None,
        custom_elements: Vec::new(),
        dialect: None,
        host_compiler: None,
        compiler_entries: vec![CompilerConfigEntry {
            base_path: None,
            files: Some(vec!["src/legacy/**".into()]),
            ignores: Vec::new(),
            template_syntax: Some("strict"),
            whitespace: Some("condense".into()),
            vapor: Some(true),
            custom_elements: vec!["legacy-*".into()],
        }],
        config_dir: project.path().to_path_buf(),
    };
    let settings = CompileFileSettings::resolve(&BuildArgs::default(), config);
    let legacy_settings = settings.compiler_for_path(&legacy);
    assert_eq!(legacy_settings.whitespace, WhitespaceStrategy::Condense);
    assert_eq!(
        legacy_settings.template_syntax,
        vize_atelier_core::TemplateSyntaxMode::Strict
    );
    assert!(legacy_settings.vapor);
    assert_eq!(legacy_settings.custom_elements, ["legacy-*"]);
    let modern_settings = settings.compiler_for_path(&modern);
    assert_eq!(modern_settings.whitespace, WhitespaceStrategy::Preserve);
    assert_eq!(
        modern_settings.template_syntax,
        vize_atelier_core::TemplateSyntaxMode::Standard
    );
    assert!(!modern_settings.vapor);
    assert!(modern_settings.custom_elements.is_empty());
    assert_ne!(settings.cache_bits(&legacy), settings.cache_bits(&modern));

    let source = include_str!(
        "../../../../../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/App.vue.txt"
    );
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let compile = |path| {
        settings
            .with_whitespace(path, || {
                compile_sfc(&descriptor, SfcCompileOptions::default())
            })
            .unwrap()
            .code
    };
    assert_ne!(compile(&modern), compile(&legacy));
}

#[test]
fn shared_stats_cache_keeps_whitespace_modes_and_restores_thread_local_parser_state() {
    let source = include_str!(
        "../../../../../../../tests/_fixtures/differential/compiler/cli-whitespace-7880/App.vue.txt"
    );
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("App.vue");
    std::fs::write(&path, source).unwrap();
    let cache = StatsCompileCache::default();
    let stats = CompileStats::new(7);
    let mut lengths = Vec::new();
    for (mode, hit, entries) in [
        (None, false, 1),
        (Some("preserve"), false, 2),
        (Some("condense"), true, 2),
        (Some("vue2-line-breaks"), false, 3),
        (Some("preserve"), true, 3),
        (None, true, 3),
        (Some("vue2-line-breaks"), true, 3),
    ] {
        let mut config = load_build_config(true, None);
        config.compiler_whitespace = mode;
        let settings = CompileFileSettings::resolve(&BuildArgs::default(), config);
        let (actual, profile) =
            compile_file_stats_with_cache(&path, &settings, &stats, &cache).unwrap();
        assert_eq!(cache.entries.lock().unwrap().len(), entries);
        if hit {
            assert_eq!(profile.parse_time, std::time::Duration::ZERO);
            assert_eq!(profile.compile_time, std::time::Duration::ZERO);
        }
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "App.vue".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let mut options = SfcCompileOptions::default();
        options.parse.filename = "App.vue".into();
        options.script.id = Some(path.to_string_lossy().as_ref().into());
        options.template.id = Some("App.vue".into());
        options.style.id = "App.vue".into();
        let expected =
            with_whitespace_mode(settings.whitespace, settings.legacy_line_breaks, || {
                compile_sfc(&descriptor, options)
            })
            .unwrap();
        assert!(expected.errors.is_empty());
        assert_eq!(actual, expected.code.len());
        lengths.push(actual);
        // The existing RAII scope must leave an ordinary subsequent compile condensed.
        let ordinary = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
        assert_eq!(
            ordinary.code,
            with_whitespace_mode(WhitespaceStrategy::Condense, false, || compile_sfc(
                &descriptor,
                SfcCompileOptions::default()
            ))
            .unwrap()
            .code
        );
    }
    assert_ne!(lengths[0], lengths[1]);
    assert_eq!(lengths[0], lengths[2]);
    assert_eq!(lengths[1], lengths[4]);
    assert_eq!(lengths[0], lengths[5]);
    assert_eq!(lengths[3], lengths[6]);
    assert_eq!(cache.entries.lock().unwrap().len(), 3);
}
