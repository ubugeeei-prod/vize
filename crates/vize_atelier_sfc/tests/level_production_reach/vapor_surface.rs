//! Attribution for Vapor templates kept on the legacy lane for
//! `surface_semantics`.
//!
//! The Vapor bridge makes that call from the template's L1 -> L2 lowering:
//! any L2 diagnostic, or a provenance rule outside the lowered, condensed and
//! harmless-drop set, keeps the retained lane. Re-lowering the same template
//! here names the first such cause; a template with neither was refused by
//! the bridge's own markup admission.

use vize_l0::Allocator;
use vize_l1::markup::SurfaceParseOptions;

/// The bridge refused the template after lowering, in markup admission.
pub const MARKUP_ADMISSION: &str = "markup_admission";

/// The first lowering cause that keeps `template` off native L3, as a bucket.
pub fn first_cause(template: &str) -> String {
    let allocator = Allocator::new();
    let (tree, errors) =
        vize_l1::markup::parse_with_options(&allocator, template, SurfaceParseOptions::default());
    let lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    if let Some(diagnostic) = lowered.diagnostics.first() {
        return format!("diagnostic:{}", message_shape(&diagnostic.message));
    }
    lowered
        .provenance
        .iter()
        .find(|record| keeps_retained_lane(&record.rule, template, record.span.start))
        .map_or_else(
            || MARKUP_ADMISSION.to_owned(),
            |record| format!("provenance:{}", record.rule),
        )
}

/// Mirrors the bridge's provenance predicate in `vize_atelier_vapor`'s L3
/// admission; keep the two in step.
fn keeps_retained_lane(rule: &str, template: &str, start: u32) -> bool {
    let harmless = rule.starts_with("lower.")
        || rule.starts_with("condense.")
        || matches!(rule, "drop.vue-is" | "drop.comment" | "drop.branch-gap");
    !harmless
        || rule == "lower.cdata-text"
        || rule == "drop.branch-gap"
            && template
                .as_bytes()
                .get(start as usize..)
                .is_none_or(|rest| rest.starts_with(b"<!--"))
}

/// A diagnostic message with quoted and backticked names elided, so one
/// cause is one bucket.
pub fn message_shape(message: &str) -> String {
    let mut shape = String::new();
    let mut quote = None;
    for character in message.chars() {
        match quote {
            Some(open) if character == open => {
                shape.push(open);
                quote = None;
            }
            Some(_) => {}
            None if matches!(character, '`' | '\'' | '"') => {
                shape.push(character);
                shape.push('_');
                quote = Some(character);
            }
            None => shape.push(character),
        }
    }
    shape.chars().take(72).collect()
}
