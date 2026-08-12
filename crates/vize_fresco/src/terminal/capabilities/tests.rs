use super::*;
use crate::terminal::{Color, Style};

fn resolve(probe: TerminalCapabilityProbe) -> TerminalCapabilities {
    TerminalCapabilities::resolve(&probe, TerminalProfileOptions::default())
}

#[test]
fn color_precedence_is_explicit_force_disable_safety_then_detection() {
    let redirected = TerminalCapabilityProbe::new(120, 40, false)
        .with_no_color(true)
        .with_force_color(Some("3"));
    let forced = resolve(redirected.clone());
    assert_eq!(forced.color().value(), ColorSupport::TrueColor);
    assert_eq!(
        forced.color().reason(),
        CapabilityReason::ForceColorEnvironment
    );
    assert!(forced.is_redirected());

    let explicit_never = TerminalCapabilities::resolve(
        &redirected,
        TerminalProfileOptions {
            color: ColorPreference::Never,
            ..TerminalProfileOptions::default()
        },
    );
    assert_eq!(explicit_never.color().value(), ColorSupport::Monochrome);
    assert_eq!(
        explicit_never.color().reason(),
        CapabilityReason::ExplicitPreference
    );

    let no_color = resolve(TerminalCapabilityProbe::new(120, 40, true).with_no_color(true));
    assert_eq!(
        no_color.color().reason(),
        CapabilityReason::NoColorEnvironment
    );
    let redirected = resolve(TerminalCapabilityProbe::new(120, 40, false));
    assert_eq!(
        redirected.color().reason(),
        CapabilityReason::RedirectedOutput
    );
}

#[test]
fn environment_color_depths_and_zero_values_are_exact() {
    let true_color = resolve(
        TerminalCapabilityProbe::new(80, 24, true)
            .with_term("xterm-256color")
            .with_colorterm("truecolor"),
    );
    assert_eq!(true_color.color().value(), ColorSupport::TrueColor);
    assert_eq!(
        true_color.color().reason(),
        CapabilityReason::DetectedTrueColor
    );

    let ansi256 = resolve(TerminalCapabilityProbe::new(80, 24, true).with_term("screen-256color"));
    assert_eq!(ansi256.color().value(), ColorSupport::Ansi256);
    assert_eq!(ansi256.color().reason(), CapabilityReason::DetectedAnsi256);

    let force_zero =
        resolve(TerminalCapabilityProbe::new(80, 24, true).with_force_color(Some("0")));
    assert_eq!(force_zero.color().value(), ColorSupport::Monochrome);
    assert_eq!(
        force_zero.color().reason(),
        CapabilityReason::ColorDisabledEnvironment
    );
}

#[test]
fn unicode_resolution_handles_overrides_locales_dumb_and_redirects() {
    let utf8 = resolve(TerminalCapabilityProbe::new(80, 24, false).with_locale("ja_JP.UTF-8"));
    assert!(utf8.unicode().value());
    assert_eq!(utf8.unicode().reason(), CapabilityReason::Utf8Locale);
    assert_eq!(utf8.select_symbol("→", "->"), "→");

    let ascii = resolve(TerminalCapabilityProbe::new(80, 24, true).with_locale("C"));
    assert!(!ascii.unicode().value());
    assert_eq!(ascii.unicode().reason(), CapabilityReason::NonUtf8Locale);
    assert_eq!(ascii.select_symbol("→", "->"), "->");

    let dumb = resolve(
        TerminalCapabilityProbe::new(80, 24, true)
            .with_term("dumb")
            .with_locale("en_US.UTF-8"),
    );
    assert_eq!(dumb.unicode().reason(), CapabilityReason::DumbTerminal);

    let invalid =
        resolve(TerminalCapabilityProbe::new(80, 24, true).with_unicode_override(Some("maybe")));
    assert!(!invalid.unicode().value());
    assert_eq!(
        invalid.unicode().reason(),
        CapabilityReason::InvalidEnvironmentOverride
    );
}

#[test]
fn interactive_resolution_never_upgrades_redirected_or_dumb_output() {
    for probe in [
        TerminalCapabilityProbe::new(80, 24, false),
        TerminalCapabilityProbe::new(80, 24, true).with_term("dumb"),
    ] {
        let profile = TerminalCapabilities::resolve(
            &probe,
            TerminalProfileOptions {
                interactive: FeaturePreference::Always,
                ..TerminalProfileOptions::default()
            },
        );
        assert!(!profile.interactive().value());
    }

    let ci = resolve(TerminalCapabilityProbe::new(80, 24, true).with_ci(true));
    assert!(!ci.interactive().value());
    assert_eq!(ci.interactive().reason(), CapabilityReason::CiEnvironment);

    let explicit = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(80, 24, true).with_ci(true),
        TerminalProfileOptions {
            interactive: FeaturePreference::Always,
            ..TerminalProfileOptions::default()
        },
    );
    assert!(explicit.interactive().value());
    assert_eq!(
        explicit.interactive().reason(),
        CapabilityReason::ExplicitPreference
    );
}

#[test]
fn narrow_layout_threshold_is_configurable_and_zero_disables_it() {
    let default_narrow = resolve(TerminalCapabilityProbe::new(59, 20, true));
    assert!(default_narrow.is_narrow());
    assert_eq!(default_narrow.narrow_width(), 60);
    assert!(!resolve(TerminalCapabilityProbe::new(60, 20, true)).is_narrow());

    let disabled = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(1, 1, true),
        TerminalProfileOptions {
            narrow_width: 0,
            ..TerminalProfileOptions::default()
        },
    );
    assert!(!disabled.is_narrow());
    assert_eq!((disabled.width(), disabled.height()), (1, 1));
}

#[test]
fn style_adaptation_preserves_attributes_and_clamps_color_depth() {
    let style = Style::new()
        .fg(Color::Rgb(255, 10, 10))
        .bg(Color::Indexed(231))
        .bold()
        .underline();
    let monochrome = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(80, 24, true),
        TerminalProfileOptions {
            color: ColorPreference::Never,
            ..TerminalProfileOptions::default()
        },
    )
    .adapt_style(style);
    assert_eq!(monochrome.fg, None);
    assert_eq!(monochrome.bg, None);
    assert!(monochrome.bold && monochrome.underline);

    let ansi16 = resolve(TerminalCapabilityProbe::new(80, 24, true)).adapt_style(style);
    assert_eq!(ansi16.fg, Some(Color::LightRed));
    assert_eq!(ansi16.bg, Some(Color::LightWhite));

    let ansi256 = resolve(TerminalCapabilityProbe::new(80, 24, true).with_term("xterm-256color"))
        .adapt_style(style);
    assert!(matches!(ansi256.fg, Some(Color::Indexed(_))));
    assert_eq!(ansi256.bg, Some(Color::Indexed(231)));
}

#[test]
fn capability_profile_serialization_preserves_values_and_reasons() {
    let profile = resolve(
        TerminalCapabilityProbe::new(42, 18, false)
            .with_locale("C")
            .with_no_color(true),
    );
    let value = serde_json::to_value(profile).unwrap();
    assert_eq!(value["width"], 42);
    assert_eq!(value["color"]["value"], "monochrome");
    assert_eq!(value["color"]["reason"], "no-color-environment");
    assert_eq!(value["unicode"]["reason"], "non-utf8-locale");
    assert_eq!(value["interactive"]["reason"], "redirected-output");
    assert_eq!(value["narrow"], true);
}

#[test]
fn napi_labels_and_serde_wire_values_cannot_drift() {
    for color in [
        ColorSupport::Monochrome,
        ColorSupport::Ansi16,
        ColorSupport::Ansi256,
        ColorSupport::TrueColor,
    ] {
        assert_eq!(serde_json::to_value(color).unwrap(), color.as_str());
    }
    for reason in [
        CapabilityReason::ExplicitPreference,
        CapabilityReason::ForceColorEnvironment,
        CapabilityReason::NoColorEnvironment,
        CapabilityReason::ColorDisabledEnvironment,
        CapabilityReason::RedirectedOutput,
        CapabilityReason::DumbTerminal,
        CapabilityReason::DetectedTerminal,
        CapabilityReason::DetectedAnsi256,
        CapabilityReason::DetectedTrueColor,
        CapabilityReason::Utf8Locale,
        CapabilityReason::NonUtf8Locale,
        CapabilityReason::UnicodeEnvironmentOverride,
        CapabilityReason::InteractiveEnvironmentOverride,
        CapabilityReason::InvalidEnvironmentOverride,
        CapabilityReason::CiEnvironment,
    ] {
        assert_eq!(serde_json::to_value(reason).unwrap(), reason.as_str());
    }
}
