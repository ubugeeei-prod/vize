//! Source-plate facts surfaced by `vize build --profile`.

use std::{path::Path, time::Duration};

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::config::VueVersion;
use vize_carton::profiler::global_profiler;
use vize_carton::{String, cstr};

use crate::commands::build::config::FileProfile;

use super::settings::CompileFileSettings;

#[derive(Clone, Copy)]
pub(super) enum StatsCacheStatus {
    Hit,
    Miss,
    BypassSelfComponent,
    NotRequested,
}

impl StatsCacheStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::BypassSelfComponent => "bypass:self-component",
            Self::NotRequested => "not-requested",
        }
    }
}

pub(super) struct FileProfileFacts {
    pub(super) file_size: usize,
    pub(super) parse_time: Duration,
    pub(super) compile_time: Duration,
    pub(super) total_time: Duration,
    pub(super) template_size: usize,
    pub(super) script_size: usize,
    pub(super) style_count: usize,
}

pub(super) fn file_profile(
    path: &Path,
    facts: FileProfileFacts,
    settings: CompileFileSettings,
    cache_status: StatsCacheStatus,
) -> FileProfile {
    record_source_facts(
        settings,
        facts.file_size,
        facts.template_size,
        facts.script_size,
        facts.style_count,
        cache_status,
    );

    FileProfile {
        path: path.to_path_buf(),
        file_size: facts.file_size,
        parse_time: facts.parse_time,
        compile_time: facts.compile_time,
        total_time: facts.total_time,
        template_size: facts.template_size,
        script_size: facts.script_size,
        style_count: facts.style_count,
        profile_note: file_note(
            settings,
            facts.template_size,
            facts.script_size,
            facts.style_count,
            cache_status,
        ),
    }
}

fn record_source_facts(
    settings: CompileFileSettings,
    file_size: usize,
    template_size: usize,
    script_size: usize,
    style_count: usize,
    cache_status: StatsCacheStatus,
) {
    let profiler = global_profiler();
    profiler.record_counter("source.plate.sfc.requests", 1);
    profiler.record_counter("source.bytes", file_size as u64);
    profiler.record_counter("source.block.template.bytes", template_size as u64);
    profiler.record_counter("source.block.script.bytes", script_size as u64);
    profiler.record_counter("source.block.style.count", style_count as u64);

    record_lane(settings);
    record_dialect(settings.dialect);
    record_template_syntax(settings.template_syntax);
    record_cache_status(cache_status);
}

fn file_note(
    settings: CompileFileSettings,
    template_size: usize,
    script_size: usize,
    style_count: usize,
    cache_status: StatsCacheStatus,
) -> String {
    cstr!(
        "lane {}, plate source.sfc, dialect {}, syntax {}, blocks template {} B / script {} B / styles {}, cache {}",
        lane_label(settings),
        dialect_label(settings.dialect),
        template_syntax_label(settings.template_syntax),
        template_size,
        script_size,
        style_count,
        cache_status.label()
    )
}

fn record_lane(settings: CompileFileSettings) {
    let profiler = global_profiler();
    match lane_label(settings) {
        "atelier.vapor" => profiler.record_counter("lane.atelier.vapor.requests", 1),
        "atelier.ssr" => profiler.record_counter("lane.atelier.ssr.requests", 1),
        "atelier.custom-renderer" => {
            profiler.record_counter("lane.atelier.custom_renderer.requests", 1);
        }
        _ => profiler.record_counter("lane.atelier.dom.requests", 1),
    }
}

fn record_dialect(dialect: VueVersion) {
    let profiler = global_profiler();
    match dialect {
        VueVersion::V3 => profiler.record_counter("dialect.vue3.files", 1),
        VueVersion::V2_7 => profiler.record_counter("dialect.vue2_7.files", 1),
        VueVersion::V2 => profiler.record_counter("dialect.vue2.files", 1),
        VueVersion::V1 => profiler.record_counter("dialect.vue1.files", 1),
        VueVersion::V0_11 => profiler.record_counter("dialect.vue0_11.files", 1),
        VueVersion::V0_10 => profiler.record_counter("dialect.vue0_10.files", 1),
    }
}

fn record_template_syntax(template_syntax: TemplateSyntaxMode) {
    let profiler = global_profiler();
    match template_syntax {
        TemplateSyntaxMode::Standard => {
            profiler.record_counter("template_syntax.standard.files", 1);
        }
        TemplateSyntaxMode::Strict => profiler.record_counter("template_syntax.strict.files", 1),
        TemplateSyntaxMode::Quirks => profiler.record_counter("template_syntax.quirks.files", 1),
        _ => profiler.record_counter("template_syntax.other.files", 1),
    }
}

fn record_cache_status(cache_status: StatsCacheStatus) {
    let profiler = global_profiler();
    match cache_status {
        StatsCacheStatus::Hit => profiler.record_counter("source.cache.hit.files", 1),
        StatsCacheStatus::Miss => profiler.record_counter("source.cache.miss.files", 1),
        StatsCacheStatus::BypassSelfComponent => {
            profiler.record_counter("source.cache.bypass.self_component.files", 1);
        }
        StatsCacheStatus::NotRequested => {
            profiler.record_counter("source.cache.not_requested.files", 1);
        }
    }
}

fn lane_label(settings: CompileFileSettings) -> &'static str {
    if settings.vapor {
        "atelier.vapor"
    } else if settings.ssr {
        "atelier.ssr"
    } else if settings.custom_renderer {
        "atelier.custom-renderer"
    } else {
        "atelier.dom"
    }
}

fn dialect_label(dialect: VueVersion) -> &'static str {
    match dialect {
        VueVersion::V3 => "vue3",
        VueVersion::V2_7 => "vue2.7",
        VueVersion::V2 => "vue2",
        VueVersion::V1 => "vue1",
        VueVersion::V0_11 => "vue0.11",
        VueVersion::V0_10 => "vue0.10",
    }
}

fn template_syntax_label(template_syntax: TemplateSyntaxMode) -> &'static str {
    match template_syntax {
        TemplateSyntaxMode::Standard => "standard",
        TemplateSyntaxMode::Strict => "strict",
        TemplateSyntaxMode::Quirks => "quirks",
        _ => "other",
    }
}
