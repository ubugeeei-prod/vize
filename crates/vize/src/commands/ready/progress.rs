//! Append-only progress for the existing ready stages, using stderr's profile.

use std::io::{IsTerminal, Write};
use std::time::{Duration, Instant};

use vize_fresco::{TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions};
use vize_l0::{String, cstr};

const STAGES: usize = 4;

pub(super) struct ReadyProgress<W = std::io::Stderr> {
    writer: W,
    capabilities: TerminalCapabilities,
    started: Instant,
    completed: usize,
}

impl ReadyProgress {
    pub(super) fn new() -> Self {
        let probe = TerminalCapabilityProbe::from_process(80, 24, std::io::stderr().is_terminal());
        Self::with_profile(std::io::stderr(), &probe)
    }
}

impl<W: Write> ReadyProgress<W> {
    fn with_profile(writer: W, probe: &TerminalCapabilityProbe) -> Self {
        let capabilities = TerminalCapabilities::resolve(probe, TerminalProfileOptions::default());
        let mut progress = Self {
            writer,
            capabilities,
            started: Instant::now(),
            completed: 0,
        };
        if progress.interactive() {
            let title = progress.styled("vize ready", "\x1b[1;96m");
            let _ = writeln!(progress.writer, "\n  {title}\n");
        }
        progress
    }

    pub(super) fn stage(&mut self, command: &str, label: &str, run: impl FnOnce()) {
        if self.interactive() {
            let position = self.styled(&cstr!("[{}/{}]", self.completed + 1, STAGES), "\x1b[90m");
            let label = self.styled(label, "\x1b[1m");
            let _ = writeln!(self.writer, "  {position} {label}");
        } else {
            let _ = writeln!(self.writer, "vize ready: {command}");
        }
        let started = Instant::now();
        run();
        self.completed += 1;
        self.completed_stage(label, started.elapsed());
    }

    fn completed_stage(&mut self, label: &str, elapsed: Duration) {
        if self.interactive() {
            let marker = self.styled(self.done_marker(), "\x1b[32m");
            let duration = self.styled(&duration(elapsed), "\x1b[90m");
            let _ = writeln!(self.writer, "  {marker} {label}  {duration}\n");
        }
    }

    pub(super) fn finish(&mut self) {
        if self.interactive() && self.completed == STAGES {
            let marker = self.styled(self.done_marker(), "\x1b[1;32m");
            let title = self.styled("Ready", "\x1b[1;32m");
            let elapsed = duration(self.started.elapsed());
            let _ = writeln!(
                self.writer,
                "  {marker} {title}  {STAGES} stages completed in {elapsed}\n"
            );
        }
    }

    fn interactive(&self) -> bool {
        self.capabilities.interactive().value()
    }

    fn done_marker(&self) -> &str {
        if self.capabilities.unicode().value() {
            "✓"
        } else {
            "done"
        }
    }

    fn styled(&self, text: &str, style: &str) -> String {
        if self.capabilities.color().value().is_color() {
            cstr!("{style}{text}\x1b[0m")
        } else {
            text.into()
        }
    }
}

fn duration(elapsed: Duration) -> String {
    if elapsed.as_secs() > 0 {
        cstr!("{:.2} s", elapsed.as_secs_f64())
    } else {
        cstr!("{} ms", elapsed.as_millis())
    }
}

#[cfg(test)]
mod tests;
