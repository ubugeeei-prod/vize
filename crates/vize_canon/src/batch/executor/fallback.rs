//! The Corsa fallback ladder: how a degradation from a faster/stronger check
//! path to a slower/weaker one is classified and signalled.

use vize_carton::{String, cstr};

use super::super::error::CorsaError;

/// A degradation step on the Corsa fallback ladder. Each variant names the
/// faster/stronger path that failed and the slower/weaker path taken instead,
/// so an operator can tell from a single signal which capability was lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FallbackStep {
    /// Parallel sharded CLI check failed; degraded to a single CLI program
    /// (doubles work on wide machines).
    ShardedCliToSingleCli,
    /// The CLI fast path failed; degraded to the much slower project-session
    /// API.
    CliToSession,
    /// The project-session API could not be spawned/handshaken; degraded back
    /// to a single CLI program.
    SessionToCli,
}

impl FallbackStep {
    pub(super) const fn description(self) -> &'static str {
        match self {
            FallbackStep::ShardedCliToSingleCli => {
                "sharded Corsa CLI check failed; degraded to a single CLI program"
            }
            FallbackStep::CliToSession => {
                "Corsa CLI fast path unavailable; degraded to the slower project-session API"
            }
            FallbackStep::SessionToCli => {
                "Corsa project-session API unavailable; degraded to a single CLI program"
            }
        }
    }
}

/// Coarse cause of a ladder degradation, so the signal distinguishes a dead
/// process from an unparseable runtime from a real type-check failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FallbackCause {
    /// The Corsa process could not be spawned or died (IO error, broken pipe,
    /// closed reader, panic).
    Spawn,
    /// The Corsa process ran but its output could not be decoded/parsed.
    Parse,
    /// The Corsa process produced a usable error that is neither a spawn nor a
    /// parse problem.
    Check,
}

impl FallbackCause {
    pub(super) const fn label(self) -> &'static str {
        match self {
            FallbackCause::Spawn => "spawn",
            FallbackCause::Parse => "parse",
            FallbackCause::Check => "check",
        }
    }
}

/// Classify a ladder error as a spawn, parse, or check failure from its kind
/// and message, without depending on any single error variant.
pub(super) fn classify_fallback_cause(error: &CorsaError) -> FallbackCause {
    match error {
        CorsaError::Io(_) => FallbackCause::Spawn,
        CorsaError::JsonParse(_) => FallbackCause::Parse,
        _ => {
            let message = cstr!("{error}");
            if message.contains("Broken pipe")
                || message.contains("broken pipe")
                || message.contains("process is closed")
                || message.contains("jsonrpc reader")
                || message.contains("worker panicked")
                || message.contains("No such file")
                || message.contains("not found")
            {
                FallbackCause::Spawn
            } else if message.contains("marker")
                || message.contains("decode")
                || message.contains("parse")
                || message.contains("unexpected")
            {
                FallbackCause::Parse
            } else {
                FallbackCause::Check
            }
        }
    }
}

/// Whether the once-per-run fallback signal has already been emitted to stderr.
/// The structured `tracing` event fires on every degradation (subscribers can
/// dedup/aggregate), but the human-facing stderr line is emitted at most once
/// per process so a large project's repeated fallbacks stay quiet.
pub(super) static FALLBACK_NOTICE_EMITTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Emit an observable signal that the Corsa integration degraded to a
/// slower/weaker path. Always records a structured `tracing::warn!` event
/// (consumed by the LSP and CI which install a subscriber); additionally prints
/// a single human-readable stderr warning per process for the `vize check` CLI,
/// which installs no subscriber. The happy path never reaches this function, so
/// a fully fast-path run stays silent.
pub(super) fn warn_fallback(step: FallbackStep, error: &CorsaError) {
    let cause = classify_fallback_cause(error);
    tracing::warn!(
        target: "vize_canon::corsa::fallback",
        fallback = ?step,
        cause = cause.label(),
        error = %error,
        "{}",
        step.description(),
    );

    if let Some(notice) = fallback_stderr_notice(step, cause) {
        eprintln!("{notice}");
    }
}

/// Build the human-facing stderr warning for a ladder degradation, claiming the
/// once-per-process slot as a side effect. Returns `None` when the notice is
/// suppressed (`VIZE_SILENCE_CORSA_FALLBACK`) or already emitted this run, so a
/// large project's repeated fallbacks stay quiet. Separated from `eprintln!` so
/// the once/suppression policy is unit-testable without capturing real stderr.
pub(super) fn fallback_stderr_notice(step: FallbackStep, cause: FallbackCause) -> Option<String> {
    // Noise-sensitive embedders can suppress only the stderr line; the
    // structured `tracing` event still fires for observability.
    if std::env::var_os("VIZE_SILENCE_CORSA_FALLBACK").is_some() {
        return None;
    }
    if FALLBACK_NOTICE_EMITTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    Some(cstr!(
        "\x1b[33mwarning:\x1b[0m corsa: {} ({} failure). Type checking continues on a slower path.",
        step.description(),
        cause.label(),
    ))
}
