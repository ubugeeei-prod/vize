use super::{CapabilityDecision, ColorSupport, TerminalCapabilities};

impl TerminalCapabilities {
    /// Return the viewport width in cells.
    pub const fn width(self) -> u16 {
        self.width
    }

    /// Return the viewport height in cells.
    pub const fn height(self) -> u16 {
        self.height
    }

    /// Return the color decision and its reason.
    pub const fn color(self) -> CapabilityDecision<ColorSupport> {
        self.color
    }

    /// Return the Unicode decision and its reason.
    pub const fn unicode(self) -> CapabilityDecision<bool> {
        self.unicode
    }

    /// Return the interactive-mode decision and its reason.
    pub const fn interactive(self) -> CapabilityDecision<bool> {
        self.interactive
    }

    /// Return whether standard output is redirected.
    pub const fn is_redirected(self) -> bool {
        self.redirected
    }

    /// Return whether the viewport is below the configured narrow threshold.
    pub const fn is_narrow(self) -> bool {
        self.narrow
    }

    /// Return the configured narrow-layout threshold.
    pub const fn narrow_width(self) -> u16 {
        self.narrow_width
    }
}
