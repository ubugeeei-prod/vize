//! Interactive Fresco presentation for normalized Doctor reports.

#[cfg(feature = "profiling")]
mod benchmark;
mod model;
mod render;

#[cfg(test)]
mod tests;

use std::{
    env, fmt,
    io::{self, IsTerminal, Write},
    path::Path,
    process::Command,
};

use vize_doctor::DoctorReport;
use vize_fresco::{
    Backend, Event, FrameActivityTelemetry, FrameRenderer, TerminalCapabilities,
    TerminalCapabilityProbe, TerminalPanicHookError, TerminalPanicHookInstallation,
    TerminalProfileOptions, TerminalSignalHookError, TerminalSignalHookInstallation,
    input::read_event, install_terminal_panic_hook, install_terminal_signal_hook,
    terminal::TerminalOptions,
};
use vize_s0::{String, ToCompactString, cstr};

use super::{DoctorFormat, DoctorSource};
use model::{DoctorTuiModel, InteractionOutcome};
use render::build_frame;

const TERMINAL_OPTIONS: TerminalOptions = TerminalOptions {
    raw_mode: true,
    alternate_screen: true,
    mouse_capture: false,
    bracketed_paste: true,
    hide_cursor: true,
};

#[cfg(feature = "profiling")]
pub use benchmark::DoctorTuiBenchmark;

/// Failure to enter, drive, or restore the interactive Doctor workspace.
#[derive(Debug)]
pub(super) enum DoctorTuiError {
    InvalidFormat,
    NonInteractive(&'static str),
    Io(io::Error),
    Presentation(vize_fresco::DiagnosticPresentationError),
    Frame(vize_fresco::FrameRenderError),
    /// Fresco could not install process-wide emergency terminal restoration.
    PanicSupervision(TerminalPanicHookError),
    /// Fresco could not install process-wide termination-signal restoration.
    SignalSupervision(TerminalSignalHookError),
    /// Application work and the mandatory terminal restoration both failed.
    ///
    /// The application error remains the primary source while the display text
    /// retains the independent restoration failure for an actionable report.
    SessionAndRestoration {
        session: Box<DoctorTuiError>,
        restoration: io::Error,
    },
}

impl fmt::Display for DoctorTuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => {
                formatter.write_str("--tui cannot be combined with --format json or sarif")
            }
            Self::NonInteractive(reason) => write!(
                formatter,
                "--tui requires an interactive stdin and stdout ({reason})"
            ),
            Self::Io(error) => write!(formatter, "terminal operation failed: {error}"),
            Self::Presentation(error) => write!(formatter, "invalid diagnostic view: {error}"),
            Self::Frame(error) => write!(formatter, "Doctor frame failed: {error}"),
            Self::PanicSupervision(error) => {
                write!(formatter, "terminal panic supervision failed: {error}")
            }
            Self::SignalSupervision(error) => {
                write!(formatter, "terminal signal supervision failed: {error}")
            }
            Self::SessionAndRestoration {
                session,
                restoration,
            } => write!(
                formatter,
                "Doctor TUI failed: {session}; terminal restoration also failed: {restoration}"
            ),
        }
    }
}

impl std::error::Error for DoctorTuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Presentation(error) => Some(error),
            Self::Frame(error) => Some(error),
            Self::PanicSupervision(error) => Some(error),
            Self::SignalSupervision(error) => Some(error),
            Self::SessionAndRestoration { session, .. } => Some(session.as_ref()),
            Self::InvalidFormat | Self::NonInteractive(_) => None,
        }
    }
}

impl From<io::Error> for DoctorTuiError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<vize_fresco::DiagnosticPresentationError> for DoctorTuiError {
    fn from(error: vize_fresco::DiagnosticPresentationError) -> Self {
        Self::Presentation(error)
    }
}

impl From<vize_fresco::FrameRenderError> for DoctorTuiError {
    fn from(error: vize_fresco::FrameRenderError) -> Self {
        Self::Frame(error)
    }
}

pub(super) fn validate_request(
    format: DoctorFormat,
) -> Result<TerminalCapabilities, DoctorTuiError> {
    if format != DoctorFormat::Text {
        return Err(DoctorTuiError::InvalidFormat);
    }
    let capabilities = TerminalCapabilities::detect_stdout();
    if !io::stdin().is_terminal() {
        return Err(DoctorTuiError::NonInteractive("stdin is redirected"));
    }
    if !capabilities.interactive().value() {
        return Err(DoctorTuiError::NonInteractive(
            capabilities.interactive().reason().as_str(),
        ));
    }
    Ok(capabilities)
}

pub(super) fn run(
    report: &DoctorReport,
    sources: &[DoctorSource],
    root: &Path,
    mut capabilities: TerminalCapabilities,
) -> Result<(), DoctorTuiError> {
    prepare_panic_supervision(install_terminal_panic_hook)?;
    prepare_signal_supervision(install_terminal_signal_hook)?;
    let mut backend = Backend::new()?;
    backend.init_with_options(TERMINAL_OPTIONS)?;
    backend.clear()?;
    backend.cursor_mut().hide();
    let mut model = DoctorTuiModel::new(report, backend.width(), backend.height());

    let session = run_loop(
        &mut backend,
        &mut model,
        sources,
        root,
        &mut capabilities,
        read_event,
    );
    let restoration = backend.restore();
    finish_session(session, restoration)
}

/// Prepare emergency terminal restoration before Doctor acquires any modes.
///
/// Fresco restores native terminal state before aborting panics on Unix and
/// Windows. An unsupported platform retains Doctor's ordinary transactional and
/// unwind restoration. Failures on a supported platform are surfaced before
/// terminal state changes, rather than silently weakening the promised
/// supervision.
fn prepare_panic_supervision(
    install: impl FnOnce() -> Result<TerminalPanicHookInstallation, TerminalPanicHookError>,
) -> Result<(), DoctorTuiError> {
    match install() {
        Ok(_) => Ok(()),
        Err(TerminalPanicHookError::UnsupportedPlatform) => Ok(()),
        Err(error) => Err(DoctorTuiError::PanicSupervision(error)),
    }
}

/// Prepare termination-signal restoration before Doctor acquires any modes.
///
/// Unix installations restore terminal presentation and native raw attributes
/// before delegating `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT`. Windows
/// installations do the same before forwarding console control events.
/// Unsupported platforms retain Doctor's transactional normal and unwind
/// cleanup. Every other failure is reported before the backend can change
/// terminal state.
fn prepare_signal_supervision(
    install: impl FnOnce() -> Result<TerminalSignalHookInstallation, TerminalSignalHookError>,
) -> Result<(), DoctorTuiError> {
    match install() {
        Ok(_) => Ok(()),
        Err(TerminalSignalHookError::UnsupportedPlatform) => Ok(()),
        Err(error) => Err(DoctorTuiError::SignalSupervision(error)),
    }
}

fn finish_session(
    session: Result<(), DoctorTuiError>,
    restoration: io::Result<()>,
) -> Result<(), DoctorTuiError> {
    match (session, restoration) {
        (Err(session), Err(restoration)) => Err(DoctorTuiError::SessionAndRestoration {
            session: Box::new(session),
            restoration,
        }),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn run_loop<W: Write>(
    backend: &mut Backend<W>,
    model: &mut DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    root: &Path,
    capabilities: &mut TerminalCapabilities,
    mut next_event: impl FnMut() -> io::Result<Event>,
) -> Result<(), DoctorTuiError> {
    let mut renderer = FrameRenderer::new();
    loop {
        let mut frame = build_frame(model, sources, *capabilities)?;
        model.place_cursor(backend.cursor_mut());
        renderer.render(frame.tree_mut(), backend, FrameActivityTelemetry::default())?;

        let outcome = match next_event()? {
            Event::Resize(width, height) => {
                backend.resize(width, height);
                model.resize(width, height);
                *capabilities = capabilities_for(width, height);
                InteractionOutcome::Changed
            }
            Event::Paste(text) => model.handle_paste(&text),
            Event::Key(event) => model.handle_key(&event),
            _ => InteractionOutcome::Boundary,
        };
        match outcome {
            InteractionOutcome::Exit => return Ok(()),
            InteractionOutcome::OpenSource => {
                open_selected_source(backend, model, sources, root)?;
            }
            InteractionOutcome::Changed | InteractionOutcome::Boundary => {}
        }
    }
}

fn capabilities_for(width: u16, height: u16) -> TerminalCapabilities {
    TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::from_process(width, height, io::stdout().is_terminal()),
        TerminalProfileOptions::default(),
    )
}

fn open_selected_source<W: Write>(
    backend: &mut Backend<W>,
    model: &mut DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    root: &Path,
) -> Result<(), DoctorTuiError> {
    backend.restore()?;
    let result = launch_editor(model, sources, root);
    backend.init_with_options(TERMINAL_OPTIONS)?;
    backend.clear()?;
    match result {
        Ok(status) | Err(status) => model.set_status(status),
    }
    Ok(())
}

fn launch_editor(
    model: &DoctorTuiModel<'_>,
    sources: &[DoctorSource],
    root: &Path,
) -> Result<&'static str, &'static str> {
    let Some(finding) = model.selected_finding() else {
        return Err("No finding is selected");
    };
    let editor = env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("EDITOR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .ok_or("Set VISUAL or EDITOR to open source")?;
    let (line, column) = model.source_position(sources);
    let path = root.join(finding.primary.path.as_str());
    let mut command = editor_command(&editor, &path, line, column)?;
    let status = command
        .status()
        .map_err(|_| "Editor could not be started")?;
    if status.success() {
        Ok("Returned from editor")
    } else {
        Err("Editor exited unsuccessfully")
    }
}

fn editor_command(
    editor: &str,
    path: &Path,
    line: u64,
    column: u64,
) -> Result<Command, &'static str> {
    let mut parts = editor.split_whitespace();
    let executable = parts.next().ok_or("VISUAL or EDITOR is empty")?;
    let program = Path::new(executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable);
    let mut command = Command::new(executable);
    command.args(parts);
    match program {
        "code" | "code-insiders" | "codium" => {
            let target = path_with_position(path, line, column);
            command.arg("--goto").arg(target.as_str());
        }
        "idea" | "idea64" => {
            let line = line.to_compact_string();
            command.arg("--line").arg(line.as_str()).arg(path);
        }
        _ => {
            let line = line_argument(line);
            command.arg(line.as_str()).arg(path);
        }
    }
    Ok(command)
}

fn line_argument(line: u64) -> String {
    cstr!("+{line}")
}

fn path_with_position(path: &Path, line: u64, column: u64) -> String {
    cstr!("{}:{line}:{column}", path.display())
}
