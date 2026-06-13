//! Profile facts recorded around per-file SFC compilation.

use vize_carton::profiler::global_profiler;

use super::cache::StatsCompileCacheDecision;
use super::settings::CompileFileSettings;

pub(super) fn record_atelier_profile_facts(
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
    profiler.record_counter("atelier.profile.target.ssr", u64::from(settings.ssr));
    profiler.record_counter("atelier.profile.target.vapor", u64::from(settings.vapor));
    profiler.record_counter(
        "atelier.profile.target.dom",
        u64::from(!settings.ssr && !settings.vapor),
    );
}

pub(super) fn record_atelier_cache_decision(
    settings: CompileFileSettings,
    decision: StatsCompileCacheDecision,
) {
    if !settings.record_profile_totals {
        return;
    }

    let name = match decision {
        StatsCompileCacheDecision::Cacheable => "atelier.cache.stats_compile.eligible",
        StatsCompileCacheDecision::BypassSelfComponentExact => {
            "atelier.cache.stats_compile.bypass.self_exact"
        }
        StatsCompileCacheDecision::BypassSelfComponentKebab => {
            "atelier.cache.stats_compile.bypass.self_kebab"
        }
    };
    global_profiler().record_counter(name, 1);
}

fn usize_to_counter(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
}
