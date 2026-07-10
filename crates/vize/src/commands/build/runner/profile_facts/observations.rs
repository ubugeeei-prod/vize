//! Compatibility profile observations around per-file compilation.
//!
//! Graph-native execution metrics come from Atlas traces. These counters keep
//! the existing CLI report stable while the legacy compiler entrypoint remains.

use vize_carton::{config::VueVersion, profiler::global_profiler};

use super::super::cache::StatsCompileCacheDecision;
use super::super::settings::CompileFileSettings;

pub(crate) fn record_atelier_profile_facts(
    settings: CompileFileSettings,
    template_size: usize,
    script_size: usize,
    style_count: usize,
    has_scoped: bool,
    is_ts: bool,
) {
    if !settings.record_profile_totals {
        return;
    }

    let profiler = global_profiler();
    profiler.record_counter("atelier.profile.lane.compiler", 1);
    profiler.record_counter("atelier.profile.input.sfc", 1);
    profiler.record_counter("atelier.profile.source.sfc", 1);
    profiler.record_counter(
        "atelier.profile.template_bytes",
        usize_to_counter(template_size),
    );
    profiler.record_counter(
        "atelier.profile.script_bytes",
        usize_to_counter(script_size),
    );
    profiler.record_counter(
        "atelier.profile.style_blocks",
        usize_to_counter(style_count),
    );
    profiler.record_counter("atelier.profile.has_scoped_style", u64::from(has_scoped));
    profiler.record_counter("atelier.profile.is_ts", u64::from(is_ts));
    profiler.record_counter(dialect_counter(settings.dialect), 1);

    record_optional(
        "atelier.profile.input.vue_template",
        "atelier.profile.source.template",
        template_size > 0,
    );
    record_optional(
        if is_ts {
            "atelier.profile.input.ts"
        } else {
            "atelier.profile.input.js"
        },
        "atelier.profile.source.script",
        script_size > 0,
    );
    record_optional(
        "atelier.profile.input.style",
        "atelier.profile.source.style",
        style_count > 0,
    );
    record_target("atelier.profile.target.ssr", settings.ssr);
    record_target("atelier.profile.target.vapor", settings.vapor);
    record_target(
        "atelier.profile.target.vdom",
        !settings.ssr && !settings.vapor,
    );
    record_target(
        "atelier.profile.target.dom",
        !settings.ssr && !settings.vapor,
    );

    if !matches!(settings.dialect, VueVersion::V3) {
        profiler.record_counter("atelier.profile.fallback.legacy_compatibility", 1);
    }
}

pub(crate) fn record_atelier_cache_decision(
    settings: CompileFileSettings,
    decision: StatsCompileCacheDecision,
) {
    if !settings.record_profile_totals {
        return;
    }
    let (counter, bypass) = match decision {
        StatsCompileCacheDecision::Cacheable => ("atelier.cache.stats_compile.eligible", false),
        StatsCompileCacheDecision::BypassSelfComponentExact => {
            ("atelier.cache.stats_compile.bypass.self_exact", true)
        }
        StatsCompileCacheDecision::BypassSelfComponentKebab => {
            ("atelier.cache.stats_compile.bypass.self_kebab", true)
        }
    };
    global_profiler().record_counter(counter, 1);
    if bypass {
        global_profiler().record_counter("atelier.profile.fallback.cache_bypass", 1);
    }
}

fn record_optional(input: &'static str, product: &'static str, active: bool) {
    if active {
        let profiler = global_profiler();
        profiler.record_counter(input, 1);
        profiler.record_counter(product, 1);
    }
}

fn record_target(counter: &'static str, active: bool) {
    global_profiler().record_counter(counter, u64::from(active));
}

const fn dialect_counter(version: VueVersion) -> &'static str {
    match version {
        VueVersion::V3 => "atelier.profile.dialect.vue3",
        VueVersion::V2_7 => "atelier.profile.dialect.vue2_7",
        VueVersion::V2 => "atelier.profile.dialect.vue2",
        VueVersion::V1 => "atelier.profile.dialect.vue1",
        VueVersion::V0_11 => "atelier.profile.dialect.vue0_11",
        VueVersion::V0_10 => "atelier.profile.dialect.vue0_10",
    }
}

fn usize_to_counter(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialect_counter_names_cover_all_config_versions() {
        assert_eq!(
            VueVersion::ALL.map(dialect_counter),
            [
                "atelier.profile.dialect.vue3",
                "atelier.profile.dialect.vue2_7",
                "atelier.profile.dialect.vue2",
                "atelier.profile.dialect.vue1",
                "atelier.profile.dialect.vue0_11",
                "atelier.profile.dialect.vue0_10",
            ]
        );
    }
}
