use std::io::IsTerminal;

use super::{
    CapabilityDecision, CapabilityReason, ColorPreference, ColorSupport, FeaturePreference,
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};

impl TerminalCapabilities {
    /// Detect standard-output capabilities using the default profile options.
    ///
    /// If terminal size discovery fails, positive `COLUMNS` and `LINES` values
    /// are used before the deterministic 80x24 fallback.
    pub fn detect_stdout() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap_or_else(|_| {
            (
                positive_environment_u16("COLUMNS").unwrap_or(80),
                positive_environment_u16("LINES").unwrap_or(24),
            )
        });
        let probe =
            TerminalCapabilityProbe::from_process(width, height, std::io::stdout().is_terminal());
        Self::resolve(&probe, TerminalProfileOptions::default())
    }

    /// Resolve an explicit probe and profile without reading process state.
    pub fn resolve(probe: &TerminalCapabilityProbe, options: TerminalProfileOptions) -> Self {
        Self {
            width: probe.width,
            height: probe.height,
            color: resolve_color(probe, options.color),
            unicode: resolve_unicode(probe, options.unicode),
            interactive: resolve_interactive(probe, options.interactive),
            redirected: !probe.is_terminal,
            narrow: options.narrow_width > 0 && probe.width < options.narrow_width,
            narrow_width: options.narrow_width,
        }
    }
}

fn resolve_color(
    probe: &TerminalCapabilityProbe,
    preference: ColorPreference,
) -> CapabilityDecision<ColorSupport> {
    match preference {
        ColorPreference::Never => {
            return decision(
                ColorSupport::Monochrome,
                CapabilityReason::ExplicitPreference,
            );
        }
        ColorPreference::Always(depth) => {
            return decision(depth, CapabilityReason::ExplicitPreference);
        }
        ColorPreference::Auto => {}
    }

    if let Some(value) = probe.force_color.as_deref() {
        return match forced_color_depth(value) {
            Some(depth) => decision(depth, CapabilityReason::ForceColorEnvironment),
            None => decision(
                ColorSupport::Monochrome,
                CapabilityReason::ColorDisabledEnvironment,
            ),
        };
    }
    if probe
        .clicolor_force
        .as_deref()
        .is_some_and(|value| !is_false(value))
    {
        return decision(
            detected_color_depth(probe).max(ColorSupport::Ansi16),
            CapabilityReason::ForceColorEnvironment,
        );
    }
    if probe.no_color {
        return decision(
            ColorSupport::Monochrome,
            CapabilityReason::NoColorEnvironment,
        );
    }
    if probe.clicolor.as_deref().is_some_and(is_false) {
        return decision(
            ColorSupport::Monochrome,
            CapabilityReason::ColorDisabledEnvironment,
        );
    }
    if !probe.is_terminal {
        return decision(ColorSupport::Monochrome, CapabilityReason::RedirectedOutput);
    }
    if is_dumb(probe) {
        return decision(ColorSupport::Monochrome, CapabilityReason::DumbTerminal);
    }

    let depth = detected_color_depth(probe);
    let reason = match depth {
        ColorSupport::TrueColor => CapabilityReason::DetectedTrueColor,
        ColorSupport::Ansi256 => CapabilityReason::DetectedAnsi256,
        ColorSupport::Ansi16 | ColorSupport::Monochrome => CapabilityReason::DetectedTerminal,
    };
    decision(depth.max(ColorSupport::Ansi16), reason)
}

fn resolve_unicode(
    probe: &TerminalCapabilityProbe,
    preference: FeaturePreference,
) -> CapabilityDecision<bool> {
    match preference {
        FeaturePreference::Never => {
            return decision(false, CapabilityReason::ExplicitPreference);
        }
        FeaturePreference::Always => {
            return decision(true, CapabilityReason::ExplicitPreference);
        }
        FeaturePreference::Auto => {}
    }
    if let Some(value) = probe.unicode_override.as_deref() {
        return parse_unicode(value).map_or_else(
            || decision(false, CapabilityReason::InvalidEnvironmentOverride),
            |enabled| decision(enabled, CapabilityReason::UnicodeEnvironmentOverride),
        );
    }
    if is_dumb(probe) {
        return decision(false, CapabilityReason::DumbTerminal);
    }
    if let Some(locale) = probe.locale.as_deref() {
        return if is_utf8_locale(locale) {
            decision(true, CapabilityReason::Utf8Locale)
        } else {
            decision(false, CapabilityReason::NonUtf8Locale)
        };
    }
    if probe.is_terminal {
        decision(true, CapabilityReason::DetectedTerminal)
    } else {
        decision(false, CapabilityReason::RedirectedOutput)
    }
}

fn resolve_interactive(
    probe: &TerminalCapabilityProbe,
    preference: FeaturePreference,
) -> CapabilityDecision<bool> {
    if !probe.is_terminal {
        return decision(false, CapabilityReason::RedirectedOutput);
    }
    if is_dumb(probe) {
        return decision(false, CapabilityReason::DumbTerminal);
    }
    match preference {
        FeaturePreference::Never => {
            return decision(false, CapabilityReason::ExplicitPreference);
        }
        FeaturePreference::Always => {
            return decision(true, CapabilityReason::ExplicitPreference);
        }
        FeaturePreference::Auto => {}
    }
    if let Some(value) = probe.interactive_override.as_deref() {
        return parse_boolean(value).map_or_else(
            || decision(false, CapabilityReason::InvalidEnvironmentOverride),
            |enabled| decision(enabled, CapabilityReason::InteractiveEnvironmentOverride),
        );
    }
    if probe.ci {
        return decision(false, CapabilityReason::CiEnvironment);
    }
    decision(true, CapabilityReason::DetectedTerminal)
}

fn decision<T: Copy>(value: T, reason: CapabilityReason) -> CapabilityDecision<T> {
    CapabilityDecision::new(value, reason)
}

fn detected_color_depth(probe: &TerminalCapabilityProbe) -> ColorSupport {
    let colorterm = probe
        .colorterm
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let term = probe
        .term
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(colorterm.as_str(), "truecolor" | "24bit")
        || term.contains("truecolor")
        || term.contains("24bit")
        || term.ends_with("-direct")
    {
        ColorSupport::TrueColor
    } else if term.contains("256color") {
        ColorSupport::Ansi256
    } else {
        ColorSupport::Ansi16
    }
}

fn forced_color_depth(value: &str) -> Option<ColorSupport> {
    match value.trim().to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "off" => None,
        "2" => Some(ColorSupport::Ansi256),
        "3" | "truecolor" | "24bit" => Some(ColorSupport::TrueColor),
        _ => Some(ColorSupport::Ansi16),
    }
}

fn parse_unicode(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "unicode" | "utf8" | "utf-8" => Some(true),
        "0" | "false" | "no" | "off" | "ascii" => Some(false),
        _ => None,
    }
}

fn parse_boolean(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn is_false(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "no" | "off"
    )
}

fn is_utf8_locale(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace('-', "");
    normalized.contains("utf8")
}

fn is_dumb(probe: &TerminalCapabilityProbe) -> bool {
    probe
        .term
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("dumb"))
}

fn positive_environment_u16(name: &str) -> Option<u16> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
}
