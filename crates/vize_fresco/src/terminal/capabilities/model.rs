use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Color depth a presentation may emit without violating its output profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSupport {
    /// Emit no foreground or background color sequences.
    Monochrome,
    /// Emit the 16 named ANSI colors.
    #[serde(rename = "ansi-16")]
    Ansi16,
    /// Emit the 256-color indexed palette.
    #[serde(rename = "ansi-256")]
    Ansi256,
    /// Emit 24-bit RGB colors.
    TrueColor,
}

impl ColorSupport {
    /// Return whether any terminal colors may be emitted.
    pub const fn is_color(self) -> bool {
        !matches!(self, Self::Monochrome)
    }

    /// Return whether 24-bit colors may be emitted unchanged.
    pub const fn is_true_color(self) -> bool {
        matches!(self, Self::TrueColor)
    }

    /// Return the stable kebab-case wire value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Monochrome => "monochrome",
            Self::Ansi16 => "ansi-16",
            Self::Ansi256 => "ansi-256",
            Self::TrueColor => "true-color",
        }
    }
}

/// Why one resolved capability has its current value.
///
/// Reasons are intentionally part of the public contract. They accompany the
/// [`CapabilityDecision`] values exposed for color, Unicode, and interactivity;
/// redirected output and narrow layout remain plain boolean flags without
/// reasons. Diagnostic presentations should report these values when color,
/// Unicode, or interactivity is unavailable or forced, instead of deriving
/// unsupported capability reasons from escape sequences or rendered glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityReason {
    /// The caller selected an explicit API preference.
    ExplicitPreference,
    /// A force-color environment variable enabled color output.
    ForceColorEnvironment,
    /// `NO_COLOR` disabled color output.
    NoColorEnvironment,
    /// `FORCE_COLOR=0` or `CLICOLOR=0` disabled color output.
    ColorDisabledEnvironment,
    /// Standard output is not a terminal.
    RedirectedOutput,
    /// `TERM=dumb` requires the most conservative presentation.
    DumbTerminal,
    /// Terminal metadata selected the ordinary ANSI palette.
    DetectedTerminal,
    /// Terminal metadata advertised a 256-color palette.
    #[serde(rename = "detected-ansi-256")]
    DetectedAnsi256,
    /// Terminal metadata advertised 24-bit RGB color.
    DetectedTrueColor,
    /// A UTF-8 locale enabled Unicode presentation.
    Utf8Locale,
    /// A known non-UTF-8 locale required ASCII presentation.
    NonUtf8Locale,
    /// `FRESCO_UNICODE` selected Unicode or ASCII presentation.
    UnicodeEnvironmentOverride,
    /// `FRESCO_INTERACTIVE` selected interactive behavior.
    InteractiveEnvironmentOverride,
    /// An invalid Fresco boolean override failed closed.
    InvalidEnvironmentOverride,
    /// A CI environment disabled interactive terminal behavior.
    CiEnvironment,
}

impl CapabilityReason {
    /// Return the stable kebab-case wire value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPreference => "explicit-preference",
            Self::ForceColorEnvironment => "force-color-environment",
            Self::NoColorEnvironment => "no-color-environment",
            Self::ColorDisabledEnvironment => "color-disabled-environment",
            Self::RedirectedOutput => "redirected-output",
            Self::DumbTerminal => "dumb-terminal",
            Self::DetectedTerminal => "detected-terminal",
            Self::DetectedAnsi256 => "detected-ansi-256",
            Self::DetectedTrueColor => "detected-true-color",
            Self::Utf8Locale => "utf8-locale",
            Self::NonUtf8Locale => "non-utf8-locale",
            Self::UnicodeEnvironmentOverride => "unicode-environment-override",
            Self::InteractiveEnvironmentOverride => "interactive-environment-override",
            Self::InvalidEnvironmentOverride => "invalid-environment-override",
            Self::CiEnvironment => "ci-environment",
        }
    }
}

/// A resolved value together with its stable explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDecision<T> {
    value: T,
    reason: CapabilityReason,
}

impl<T: Copy> CapabilityDecision<T> {
    pub(super) const fn new(value: T, reason: CapabilityReason) -> Self {
        Self { value, reason }
    }

    /// Return the resolved capability value.
    pub const fn value(self) -> T {
        self.value
    }

    /// Return the reason the value was selected.
    pub const fn reason(self) -> CapabilityReason {
        self.reason
    }
}

/// Caller-level color policy. Explicit preferences precede environment values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "mode", content = "depth")]
pub enum ColorPreference {
    /// Resolve color from the probe and supported environment conventions.
    #[default]
    Auto,
    /// Force a monochrome presentation.
    Never,
    /// Force the supplied maximum color depth, including redirected output.
    Always(ColorSupport),
}

/// Caller-level policy for a boolean capability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeaturePreference {
    /// Resolve from the probe and supported environment conventions.
    #[default]
    Auto,
    /// Disable the feature.
    Never,
    /// Enable the feature when its safety preconditions permit it.
    Always,
}

/// Stable inputs controlling capability resolution.
///
/// Defaults resolve color, Unicode, and interactivity automatically and mark
/// viewports narrower than 60 columns as narrow. Automatic resolution never
/// upgrades redirected output or `TERM=dumb` to interactive mode, even when
/// callers request [`FeaturePreference::Always`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalProfileOptions {
    /// Color preference. Defaults to [`ColorPreference::Auto`].
    pub color: ColorPreference,
    /// Unicode preference. Defaults to [`FeaturePreference::Auto`].
    pub unicode: FeaturePreference,
    /// Interactive-mode preference. Defaults to [`FeaturePreference::Auto`].
    ///
    /// Redirected output and `TERM=dumb` remain non-interactive even when this
    /// is [`FeaturePreference::Always`], preventing unsafe terminal mode entry.
    pub interactive: FeaturePreference,
    /// Widths below this value use the narrow layout. Defaults to 60 cells.
    /// A value of zero disables automatic narrow-layout selection.
    pub narrow_width: u16,
}

impl Default for TerminalProfileOptions {
    fn default() -> Self {
        Self {
            color: ColorPreference::Auto,
            unicode: FeaturePreference::Auto,
            interactive: FeaturePreference::Auto,
            narrow_width: 60,
        }
    }
}

/// Explicit terminal and environment observations used by the resolver.
///
/// Build a probe with [`TerminalCapabilityProbe::new`] for deterministic tests
/// and embedding. [`TerminalCapabilityProbe::from_process`] reads only the
/// documented environment variables represented by this type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCapabilityProbe {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) is_terminal: bool,
    pub(super) term: Option<CompactString>,
    pub(super) colorterm: Option<CompactString>,
    pub(super) no_color: bool,
    pub(super) force_color: Option<CompactString>,
    pub(super) clicolor: Option<CompactString>,
    pub(super) clicolor_force: Option<CompactString>,
    pub(super) locale: Option<CompactString>,
    pub(super) unicode_override: Option<CompactString>,
    pub(super) interactive_override: Option<CompactString>,
    pub(super) ci: bool,
}

impl TerminalCapabilityProbe {
    /// Create a probe with no environment hints.
    pub const fn new(width: u16, height: u16, is_terminal: bool) -> Self {
        Self {
            width,
            height,
            is_terminal,
            term: None,
            colorterm: None,
            no_color: false,
            force_color: None,
            clicolor: None,
            clicolor_force: None,
            locale: None,
            unicode_override: None,
            interactive_override: None,
            ci: false,
        }
    }

    /// Read the supported environment conventions around an explicit viewport.
    pub fn from_process(width: u16, height: u16, is_terminal: bool) -> Self {
        let value = |name| std::env::var_os(name).map(|item| item.to_string_lossy().into_owned());
        let locale = value("LC_ALL")
            .filter(|item| !item.is_empty())
            .or_else(|| value("LC_CTYPE").filter(|item| !item.is_empty()))
            .or_else(|| value("LANG").filter(|item| !item.is_empty()));
        Self::new(width, height, is_terminal)
            .with_term(value("TERM").unwrap_or_default())
            .with_colorterm(value("COLORTERM").unwrap_or_default())
            .with_no_color(std::env::var_os("NO_COLOR").is_some())
            .with_force_color(value("FORCE_COLOR"))
            .with_clicolor(value("CLICOLOR"))
            .with_clicolor_force(value("CLICOLOR_FORCE"))
            .with_locale(locale.unwrap_or_default())
            .with_unicode_override(value("FRESCO_UNICODE"))
            .with_interactive_override(value("FRESCO_INTERACTIVE"))
            .with_ci(value("CI").is_some_and(|item| !is_false(&item)))
    }

    /// Set the `TERM` value. An empty value removes the hint.
    pub fn with_term(mut self, value: impl Into<CompactString>) -> Self {
        self.term = non_empty(value.into());
        self
    }

    /// Set the `COLORTERM` value. An empty value removes the hint.
    pub fn with_colorterm(mut self, value: impl Into<CompactString>) -> Self {
        self.colorterm = non_empty(value.into());
        self
    }

    /// Record whether `NO_COLOR` is present. Presence disables automatic color.
    pub const fn with_no_color(mut self, present: bool) -> Self {
        self.no_color = present;
        self
    }

    /// Set `FORCE_COLOR`, preserving an explicitly empty value.
    pub fn with_force_color(mut self, value: Option<impl Into<CompactString>>) -> Self {
        self.force_color = value.map(Into::into);
        self
    }

    /// Set `CLICOLOR`.
    pub fn with_clicolor(mut self, value: Option<impl Into<CompactString>>) -> Self {
        self.clicolor = value.map(Into::into);
        self
    }

    /// Set `CLICOLOR_FORCE`.
    pub fn with_clicolor_force(mut self, value: Option<impl Into<CompactString>>) -> Self {
        self.clicolor_force = value.map(Into::into);
        self
    }

    /// Set the effective `LC_ALL`, `LC_CTYPE`, or `LANG` locale.
    pub fn with_locale(mut self, value: impl Into<CompactString>) -> Self {
        self.locale = non_empty(value.into());
        self
    }

    /// Set `FRESCO_UNICODE` (`1`/`true`/`unicode` or `0`/`false`/`ascii`).
    pub fn with_unicode_override(mut self, value: Option<impl Into<CompactString>>) -> Self {
        self.unicode_override = value.map(Into::into);
        self
    }

    /// Set `FRESCO_INTERACTIVE` (`1`/`true` or `0`/`false`).
    pub fn with_interactive_override(mut self, value: Option<impl Into<CompactString>>) -> Self {
        self.interactive_override = value.map(Into::into);
        self
    }

    /// Record whether a truthy `CI` value is present.
    pub const fn with_ci(mut self, ci: bool) -> Self {
        self.ci = ci;
        self
    }
}

/// Complete presentation profile resolved for one terminal viewport.
///
/// Each capability includes both the resolved value and a stable
/// [`CapabilityReason`]. This lets diagnostic and inspection tools expose why a
/// terminal is monochrome, ASCII-only, non-interactive, redirected, or narrow
/// without touching process-global terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalCapabilities {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) color: CapabilityDecision<ColorSupport>,
    pub(super) unicode: CapabilityDecision<bool>,
    pub(super) interactive: CapabilityDecision<bool>,
    pub(super) redirected: bool,
    pub(super) narrow: bool,
    pub(super) narrow_width: u16,
}

fn non_empty(value: CompactString) -> Option<CompactString> {
    (!value.is_empty()).then_some(value)
}

fn is_false(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off"
    )
}
