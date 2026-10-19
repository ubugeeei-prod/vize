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
mod tests {
    use super::*;

    fn profile() -> TerminalCapabilityProbe {
        TerminalCapabilityProbe::new(80, 24, true)
            .with_term("xterm-256color")
            .with_locale("en_US.UTF-8")
    }

    fn transcript(probe: &TerminalCapabilityProbe) -> String {
        let mut progress = ReadyProgress::with_profile(Vec::new(), probe);
        for (command, label) in [
            ("fmt", "Format"),
            ("lint", "Lint"),
            ("check", "Type check"),
            ("build", "Build"),
        ] {
            progress.stage(command, label, || {});
        }
        progress.finish();
        std::str::from_utf8(&progress.writer).unwrap().into()
    }

    #[test]
    fn redirected_ci_and_dumb_profiles_keep_the_existing_log_bytes() {
        for probe in [
            TerminalCapabilityProbe::new(80, 24, false).with_force_color(Some("1")),
            profile().with_ci(true),
            profile().with_term("dumb"),
        ] {
            assert_eq!(
                transcript(&probe),
                "vize ready: fmt\nvize ready: lint\nvize ready: check\nvize ready: build\n"
            );
        }
    }

    #[test]
    fn no_color_retains_stage_counts_and_completion_meaning() {
        let output = transcript(&profile().with_no_color(true));
        assert!(!output.contains('\x1b'));
        for expected in [
            "vize ready",
            "[1/4] Format",
            "[2/4] Lint",
            "[3/4] Type check",
            "[4/4] Build",
            "✓ Ready  4 stages completed in",
        ] {
            assert!(output.contains(expected), "missing {expected} in {output}");
        }
    }

    #[test]
    fn ascii_profile_retains_every_completion_without_unicode() {
        let output = transcript(&profile().with_locale("C").with_no_color(true));
        assert!(output.is_ascii());
        assert!(output.contains("done Format"));
        assert!(output.contains("done Ready  4 stages completed in"));
    }

    #[test]
    fn terminal_color_styles_the_existing_stage_meaning() {
        let output = transcript(&profile());
        assert!(output.contains("\x1b[1;96mvize ready\x1b[0m"));
        assert!(output.contains("\x1b[1;32mReady\x1b[0m"));
        assert!(!output.contains('\r'));
        assert!(!output.contains("\x1b[?"));
    }

    #[test]
    fn an_interrupted_stage_is_never_completed_or_followed_by_ready() {
        let mut progress = ReadyProgress::with_profile(Vec::new(), &profile().with_no_color(true));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            progress.stage("fmt", "Format", || panic!("stage failed"));
        }));
        assert!(result.is_err());
        progress.finish();
        let output = std::str::from_utf8(&progress.writer).unwrap();
        assert!(output.contains("[1/4] Format"));
        assert!(!output.contains("✓ Format"));
        assert!(!output.contains("stages completed"));
    }

    #[test]
    fn durations_use_readable_units_without_a_zero_second_label() {
        assert_eq!(duration(Duration::from_millis(123)), "123 ms");
        assert_eq!(duration(Duration::from_millis(1250)), "1.25 s");
    }
}
