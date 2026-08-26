//! The comparison half of the surface projection ([`super::surface`]),
//! split along the types/check boundary under the source budget. The
//! divergence rule is TS-25: investigate, never average; every skip is
//! a counted class on [`SurfaceCounters`].

use vize_s0::String;

use super::surface::{PBind, PModel, PName, PSurface, SurfaceCounters};

/// Whether any text of `surface` is entity-shaped under the S1
/// no-decoding scope (the text projection's predicate, applied to the
/// binding surface).
fn entity_bearing_surface(surface: &PSurface) -> bool {
    let text_hit = |text: &Option<Option<String>>| matches!(text, Some(Some(t)) if super::text::entity_bearing(t.as_str()));
    let name_hit = |name: &PName| match name {
        PName::Static(text) | PName::Dynamic(Some(text)) => {
            super::text::entity_bearing(text.as_str())
        }
        PName::Dynamic(None) | PName::Spread => false,
    };
    surface.attrs.iter().any(|(name, value)| {
        super::text::entity_bearing(name.as_str())
            || value.as_deref().is_some_and(super::text::entity_bearing)
    }) || surface
        .binds
        .iter()
        .chain(&surface.ons)
        .any(|bind| text_hit(&bind.value))
        || surface.directives.iter().any(|dir| text_hit(&dir.value))
        || surface.models.iter().any(|model| {
            model
                .value
                .as_deref()
                .is_some_and(super::text::entity_bearing)
                || model.prop.as_ref().is_some_and(name_hit)
        })
}

/// One divergence panic, with everything needed to investigate.
macro_rules! diverged {
    ($name:expr, $source:expr, $old:expr, $s2:expr, $($why:tt)+) => {
        panic!(
            "TS-25 surface divergence [{}]: {}\ntemplate:\n{}\nlegacy surfaces: {:#?}\ns2 surfaces: {:#?}",
            $name, format_args!($($why)+), $source, $old, $s2
        )
    };
}

/// Compare the two lanes' owner surfaces for one template.
///
/// # Panics
///
/// Panics on any divergence inside the compared domain (TS-25).
pub fn check(
    name: &str,
    source: &str,
    old: &[PSurface],
    s2: &[PSurface],
    counters: &mut SurfaceCounters,
) {
    // The entity class is a template-level predicate on the S2 surfaces
    // (where the authored `&` survives), decided before any comparison.
    if s2.iter().any(entity_bearing_surface) {
        counters.entity_templates += 1;
        return;
    }
    if old.len() != s2.len() {
        diverged!(
            name,
            source,
            old,
            s2,
            "owner count {} vs {}",
            old.len(),
            s2.len()
        );
    }
    for (index, (old_surface, s2_surface)) in old.iter().zip(s2).enumerate() {
        if old_surface.attrs != s2_surface.attrs {
            diverged!(name, source, old, s2, "owner {index} attrs");
        }
        counters.attrs += old_surface.attrs.len() as u64;
        check_binds(
            name,
            source,
            (old, s2, index),
            (&old_surface.binds, &s2_surface.binds),
            "bind",
            counters,
        );
        check_binds(
            name,
            source,
            (old, s2, index),
            (&old_surface.ons, &s2_surface.ons),
            "on",
            counters,
        );
        if old_surface.directives.len() != s2_surface.directives.len() {
            diverged!(name, source, old, s2, "owner {index} directive count");
        }
        for (old_dir, s2_dir) in old_surface.directives.iter().zip(&s2_surface.directives) {
            let mut old_dir = old_dir.clone();
            let _ = compound_folds(&mut old_dir.value, &s2_dir.value, counters);
            if old_dir == *s2_dir {
                counters.directives += 1;
            } else {
                diverged!(name, source, old, s2, "owner {index} directive");
            }
        }
        check_models(
            name,
            source,
            (old, s2, index),
            old_surface,
            s2_surface,
            counters,
        );
        counters.owners += 1;
    }
}

/// Fold a legacy compound value into the counted class; returns whether
/// the fold applied (the caller then compares everything but the value).
fn compound_folds(
    old_value: &mut Option<Option<String>>,
    s2_value: &Option<Option<String>>,
    counters: &mut SurfaceCounters,
) -> bool {
    if matches!(old_value, Some(None)) && matches!(s2_value, Some(Some(_))) {
        counters.values_compound += 1;
        *old_value = s2_value.clone();
        true
    } else {
        false
    }
}

fn check_binds(
    name: &str,
    source: &str,
    (old, s2, index): (&[PSurface], &[PSurface], usize),
    (old_binds, s2_binds): (&[PBind], &[PBind]),
    what: &str,
    counters: &mut SurfaceCounters,
) {
    if old_binds.len() != s2_binds.len() {
        diverged!(name, source, old, s2, "owner {index} {what} count");
    }
    for (old_bind, s2_bind) in old_binds.iter().zip(s2_binds) {
        let mut old_bind = old_bind.clone();
        let _ = compound_folds(&mut old_bind.value, &s2_bind.value, counters);
        if let (PName::Dynamic(None), PName::Dynamic(Some(_))) = (&old_bind.name, &s2_bind.name) {
            counters.values_compound += 1;
            old_bind.name = s2_bind.name.clone();
        }
        if old_bind != *s2_bind {
            diverged!(
                name,
                source,
                old,
                s2,
                "owner {index} {what} {old_bind:?} vs {s2_bind:?}"
            );
        }
        let counter = match (&s2_bind.name, what) {
            (PName::Static(_), "bind") => &mut counters.binds,
            (PName::Dynamic(_), "bind") => &mut counters.binds_dynamic,
            (PName::Spread, "bind") => &mut counters.binds_spread,
            (PName::Static(_), _) => &mut counters.ons,
            (PName::Dynamic(_), _) => &mut counters.ons_dynamic,
            (PName::Spread, _) => &mut counters.ons_spread,
        };
        *counter += 1;
    }
}

/// The model half: the pattern-scope class skips the owner, and the
/// remaining contracts compare pairwise.
fn check_models(
    name: &str,
    source: &str,
    (old, s2, index): (&[PSurface], &[PSurface], usize),
    old_surface: &PSurface,
    s2_surface: &PSurface,
    counters: &mut SurfaceCounters,
) {
    if old_surface.pattern_scoped || s2_surface.pattern_scoped {
        if !old_surface.models.is_empty() || !s2_surface.models.is_empty() {
            counters.models_pattern_scope += 1;
        }
        return;
    }
    if old_surface.models.len() != s2_surface.models.len() {
        diverged!(name, source, old, s2, "owner {index} model count");
    }
    for (old_model, s2_model) in old_surface.models.iter().zip(&s2_surface.models) {
        let mut old_model = old_model.clone();
        if old_model.value.is_none() && s2_model.value.is_some() {
            counters.values_compound += 1;
            old_model.value = s2_model.value.clone();
        }
        if old_model != *s2_model {
            diverged!(
                name,
                source,
                old,
                s2,
                "owner {index} model {old_model:?} vs {s2_model:?}"
            );
        }
        counters.models += 1;
    }
}
