use std::error::Error;

use vize_fresco::{TerminalSignalHookError, TerminalSignalHookInstallation};

use super::super::{DoctorTuiError, prepare_signal_supervision};

#[test]
fn accepts_installation_idempotency_and_platform_fallback() {
    for installation in [
        TerminalSignalHookInstallation::Installed,
        TerminalSignalHookInstallation::AlreadyInstalled,
    ] {
        assert!(prepare_signal_supervision(|| Ok(installation)).is_ok());
    }

    assert!(
        prepare_signal_supervision(|| Err(TerminalSignalHookError::UnsupportedPlatform)).is_ok()
    );
}

#[test]
fn surfaces_state_failures_before_terminal_entry() {
    for cause in [
        TerminalSignalHookError::InstallationPoisoned,
        TerminalSignalHookError::InstallationStateUncertain,
    ] {
        let expected = cause.to_string();
        let error = prepare_signal_supervision(|| Err(cause)).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("terminal signal supervision failed: {expected}")
        );
        assert_eq!(error.source().map(ToString::to_string), Some(expected));
        assert!(matches!(error, DoctorTuiError::SignalSupervision(_)));
    }
}
