//! Profile facts recorded around per-file SFC compilation.

use vize_atelier_core::source_atlas::{
    SourceAtlasCoordinate, SourceAtlasPlate, SourceAtlasRegistry, SourceAtlasSource,
    SourceAtlasTarget,
};
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
    record_source_atlas_registry(
        SourceAtlasRegistry::compiler()
            .with_source(SourceAtlasSource::Sfc)
            .with_plate(SourceAtlasPlate::Sfc)
            .with_target(SourceAtlasTarget::Sfc)
            .with_coordinate(SourceAtlasCoordinate::from(settings.dialect))
            .with_source_if(template_size > 0, SourceAtlasSource::VueTemplate)
            .with_source_if(script_size > 0, source_atlas_script_source(is_ts))
            .with_source_if(style_count > 0, SourceAtlasSource::Style)
            .with_plate_if(template_size > 0, SourceAtlasPlate::Template)
            .with_plate_if(script_size > 0, SourceAtlasPlate::Script)
            .with_plate_if(style_count > 0, SourceAtlasPlate::Style)
            .with_target_if(settings.ssr, SourceAtlasTarget::Ssr)
            .with_target_if(settings.vapor, SourceAtlasTarget::Vapor)
            .with_target_if(!settings.ssr && !settings.vapor, SourceAtlasTarget::Vdom)
            .with_target_if(!settings.ssr && !settings.vapor, SourceAtlasTarget::Dom),
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

trait SourceAtlasRegistryProfileExt {
    fn with_source_if(self, enabled: bool, source: SourceAtlasSource) -> Self;
    fn with_plate_if(self, enabled: bool, plate: SourceAtlasPlate) -> Self;
    fn with_target_if(self, enabled: bool, target: SourceAtlasTarget) -> Self;
}

impl SourceAtlasRegistryProfileExt for SourceAtlasRegistry {
    fn with_source_if(self, enabled: bool, source: SourceAtlasSource) -> Self {
        if enabled {
            self.with_source(source)
        } else {
            self
        }
    }

    fn with_plate_if(self, enabled: bool, plate: SourceAtlasPlate) -> Self {
        if enabled {
            self.with_plate(plate)
        } else {
            self
        }
    }

    fn with_target_if(self, enabled: bool, target: SourceAtlasTarget) -> Self {
        if enabled {
            self.with_target(target)
        } else {
            self
        }
    }
}

fn source_atlas_script_source(is_ts: bool) -> SourceAtlasSource {
    if is_ts {
        SourceAtlasSource::Ts
    } else {
        SourceAtlasSource::Js
    }
}

fn record_source_atlas_registry(registry: SourceAtlasRegistry) {
    let route = registry.route;
    let profiler = global_profiler();
    profiler.record_counter(registry.lane.profile_counter(), 1);
    for source in route.sources.iter() {
        profiler.record_counter(source.profile_counter(), 1);
    }
    for plate in LEGACY_OBSERVED_PLATES {
        profiler.record_counter(
            plate.profile_counter(),
            u64::from(route.plates.contains(plate)),
        );
    }
    for plate in route
        .plates
        .iter()
        .filter(|plate| !is_legacy_observed_plate(*plate))
    {
        profiler.record_counter(plate.profile_counter(), 1);
    }
    for target in LEGACY_OBSERVED_TARGETS {
        profiler.record_counter(
            target.profile_counter(),
            u64::from(route.targets.contains(target)),
        );
    }
    for target in route
        .targets
        .iter()
        .filter(|target| !is_legacy_observed_target(*target))
    {
        profiler.record_counter(target.profile_counter(), 1);
    }
    if let Some(coordinate) = route.coordinate {
        profiler.record_counter(coordinate.profile_counter(), 1);
    }
    for fallback in registry.fallbacks.iter() {
        profiler.record_counter(fallback.profile_counter(), 1);
    }
}

// These counters intentionally keep 0/1 samples so existing profile reports
// preserve their per-file averages while future atlas lanes remain active-only.
const LEGACY_OBSERVED_PLATES: [SourceAtlasPlate; 4] = [
    SourceAtlasPlate::Sfc,
    SourceAtlasPlate::Template,
    SourceAtlasPlate::Script,
    SourceAtlasPlate::Style,
];

const LEGACY_OBSERVED_TARGETS: [SourceAtlasTarget; 5] = [
    SourceAtlasTarget::Sfc,
    SourceAtlasTarget::Ssr,
    SourceAtlasTarget::Vapor,
    SourceAtlasTarget::Vdom,
    SourceAtlasTarget::Dom,
];

fn is_legacy_observed_plate(plate: SourceAtlasPlate) -> bool {
    matches!(
        plate,
        SourceAtlasPlate::Sfc
            | SourceAtlasPlate::Template
            | SourceAtlasPlate::Script
            | SourceAtlasPlate::Style
    )
}

fn is_legacy_observed_target(target: SourceAtlasTarget) -> bool {
    matches!(
        target,
        SourceAtlasTarget::Sfc
            | SourceAtlasTarget::Ssr
            | SourceAtlasTarget::Vapor
            | SourceAtlasTarget::Vdom
            | SourceAtlasTarget::Dom
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_carton::config::VueVersion;

    #[test]
    fn dialect_counter_names_cover_all_config_versions() {
        let names: std::vec::Vec<_> = VueVersion::ALL
            .into_iter()
            .map(|version| SourceAtlasCoordinate::from(version).profile_counter())
            .collect();

        assert_eq!(
            names,
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

    #[test]
    fn source_and_target_counter_names_are_stable() {
        assert_eq!(
            SourceAtlasSource::Sfc.profile_counter(),
            "atelier.profile.input.sfc"
        );
        assert_eq!(
            SourceAtlasTarget::Vdom.profile_counter(),
            "atelier.profile.target.vdom"
        );
        assert_eq!(
            SourceAtlasTarget::Sfc.profile_counter(),
            "atelier.profile.target.sfc"
        );
        assert_eq!(
            SourceAtlasTarget::Tsx.profile_counter(),
            "atelier.profile.target.tsx"
        );
    }
}
