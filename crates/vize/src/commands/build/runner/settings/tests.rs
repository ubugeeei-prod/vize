//! Stats cache must distinguish actual output-affecting whitespace modes.

use super::super::{cache::StatsCompileCache, compile_stats::compile_file_stats_with_cache};
use super::{CompileFileSettings, load_build_config};
use crate::commands::build::{BuildArgs, config::CompileStats};
use vize_atelier_core::{WhitespaceStrategy, parser::with_whitespace_mode};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

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
