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
