use super::*;

#[test]
fn as_str_round_trips_all_variants_through_config_parsing() {
    assert_eq!(LegacyVueVersion::ALL.len(), 4);
    for version in LegacyVueVersion::ALL {
        let dialect = VueVersion::from_config_str(version.as_str())
            .unwrap_or_else(|error| panic!("{}: {error}", version.as_str()));
        assert_eq!(LegacyVueVersion::from_dialect(dialect), Some(version));
    }
}

#[test]
fn config_string_resolves_to_version_and_capabilities() {
    // The full plumbing a config consumer runs once per file:
    // raw string -> dialect -> legacy line -> capability set.
    let dialect = VueVersion::from_config_str("0.10").unwrap();
    let version = LegacyVueVersion::from_dialect(dialect).unwrap();
    assert_eq!(version, LegacyVueVersion::V0_10);
    let caps = version.capabilities();
    assert!(caps.computed_dollar_get_set);
    assert_eq!(caps, LegacyDialectCapabilities::for_dialect(dialect));
}

#[test]
fn v0_10_differs_from_v0_11_only_in_documented_surfaces() {
    let v0_10 = LegacyVueVersion::V0_10.capabilities();
    let v0_11 = LegacyVueVersion::V0_11.capabilities();
    // 0.10 keeps the pre-rewrite computed `$get`/`$set` form.
    assert!(v0_10.computed_dollar_get_set);
    assert!(!v0_11.computed_dollar_get_set);
    // Both 0.x lines share the clause-style directive surface.
    assert_eq!(v0_10.directive_arg_style, DirectiveArgStyle::Clause);
    assert_eq!(v0_11.directive_arg_style, DirectiveArgStyle::Clause);
    assert!(v0_10.v_repeat_syntax && v0_11.v_repeat_syntax);
    assert!(v0_10.v_with_directive && v0_11.v_with_directive);
    assert!(v0_10.v_component_directive && v0_11.v_component_directive);
    // Both 0.x lines support `{{{ html }}}` raw-HTML interpolation.
    assert!(v0_10.raw_html_interpolation && v0_11.raw_html_interpolation);
}

#[test]
fn v1_modernizes_directive_surface_but_keeps_old_filters() {
    let v0_11 = LegacyVueVersion::V0_11.capabilities();
    let v1 = LegacyVueVersion::V1.capabilities();
    assert!(v0_11.v_repeat_syntax && !v1.v_repeat_syntax);
    assert_eq!(v0_11.directive_arg_style, DirectiveArgStyle::Clause);
    assert_eq!(v1.directive_arg_style, DirectiveArgStyle::Colon);
    assert!(!v1.v_with_directive && !v1.v_component_directive);
    assert!(v1.supports_filters && v1.space_separated_filter_args);
    assert!(v1.attr_value_interpolation);
    // Vue 1.x keeps the `{{{ html }}}` raw-HTML interpolation.
    assert!(v1.raw_html_interpolation);
}

#[test]
fn v2_keeps_filters_but_drops_v1_interpolation_surfaces() {
    let v1 = LegacyVueVersion::V1.capabilities();
    let v2 = LegacyVueVersion::V2.capabilities();
    assert!(v2.supports_filters);
    assert!(v1.space_separated_filter_args && !v2.space_separated_filter_args);
    assert!(v1.attr_value_interpolation && !v2.attr_value_interpolation);
    assert!(!v1.scoped_slot_attrs && v2.scoped_slot_attrs);
    // Vue 2 dropped triple-mustache raw-HTML interpolation in favor of `v-html`.
    assert!(v1.raw_html_interpolation && !v2.raw_html_interpolation);
}

#[test]
fn v2_and_v2_7_share_the_template_dialect() {
    let v2 = LegacyVueVersion::from_dialect(VueVersion::V2).unwrap();
    let v2_7 = LegacyVueVersion::from_dialect(VueVersion::V2_7).unwrap();
    assert_eq!(v2, v2_7);
    assert_eq!(
        LegacyDialectCapabilities::for_dialect(VueVersion::V2),
        LegacyDialectCapabilities::for_dialect(VueVersion::V2_7),
    );
}

#[test]
fn default_dialect_resolves_to_the_all_off_vue3_set() {
    let caps = LegacyDialectCapabilities::for_dialect(VueVersion::V3);
    assert_eq!(caps, LegacyDialectCapabilities::VUE3);
    assert_eq!(caps, LegacyDialectCapabilities::default());
    assert!(!caps.supports_filters);
    assert!(!caps.v_repeat_syntax);
    assert!(!caps.raw_html_interpolation);
    assert_eq!(caps.directive_arg_style, DirectiveArgStyle::Colon);
    // Every legacy line differs from the default set, so a capability
    // check can never confuse a legacy document with a Vue 3 one.
    for version in LegacyVueVersion::ALL {
        assert_ne!(version.capabilities(), LegacyDialectCapabilities::VUE3);
    }
}
