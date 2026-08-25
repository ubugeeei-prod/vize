use std::{fmt, io::IsTerminal};

use vize_fresco::{
    ColorSupport, TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
use vize_s0::{String, cstr};

#[derive(Clone, Copy)]
pub(super) struct TextStyle {
    color: bool,
}

impl TextStyle {
    pub(super) fn stdout() -> Self {
        Self::from_terminal(std::io::stdout().is_terminal())
    }

    pub(super) fn stderr() -> Self {
        Self::from_terminal(std::io::stderr().is_terminal())
    }

    fn from_terminal(is_terminal: bool) -> Self {
        let probe = TerminalCapabilityProbe::from_process(80, 24, is_terminal);
        let capabilities = TerminalCapabilities::resolve(&probe, TerminalProfileOptions::default());
        Self {
            color: capabilities.color().value() != ColorSupport::Monochrome,
        }
    }

    pub(super) fn red(self, text: impl fmt::Display) -> String {
        self.paint("31", text)
    }

    pub(super) fn green(self, text: impl fmt::Display) -> String {
        self.paint("32", text)
    }

    pub(super) fn yellow(self, text: impl fmt::Display) -> String {
        self.paint("33", text)
    }

    pub(super) fn underline(self, text: impl fmt::Display) -> String {
        self.paint("4", text)
    }

    fn paint(self, code: &str, text: impl fmt::Display) -> String {
        if self.color {
            cstr!("\x1b[{code}m{text}\x1b[0m")
        } else {
            cstr!("{text}")
        }
    }
}
