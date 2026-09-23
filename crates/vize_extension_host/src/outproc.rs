//! Out-of-process hosting: the guest runs in a child process.
//!
//! The child is `vize-extension-host serve <component.wasm>` (this crate's
//! binary, built with the `extension-host` feature), which instantiates the
//! component under wasmtime and answers [`wire`] requests. The parent side
//! here links no wasm runtime at all: a guest crash, a runaway allocation or
//! a trap stays in the child's address space.
//!
//! [`wire`]: crate::wire

use std::io::{BufReader, ErrorKind};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use vize_s0::cstr;

use crate::contract::{
    Capability, GuestError, GuestLimits, InputDialectGuest, LoweredBlock, SourceBlock,
};
use crate::expression::{Analysis, ExpressionBatch, ExpressionDialectGuest};
use crate::output::{EmitRequest, Emitted, OutputTargetGuest};
use crate::wire::{Request, Response, read_message, transport, write_message};

/// A guest behind a child process.
#[derive(Debug)]
pub struct OutOfProcessGuest {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

/// The command that serves `component` from the `runner` binary under the
/// default [`GuestLimits`].
#[must_use]
pub fn serve_command(runner: &Path, component: &Path) -> Command {
    serve_command_with(runner, component, GuestLimits::default())
}

/// The command that serves `component` from the `runner` binary under
/// `limits`: the child enforces them exactly as the in-process host does.
#[must_use]
pub fn serve_command_with(runner: &Path, component: &Path, limits: GuestLimits) -> Command {
    let mut command = Command::new(runner);
    command.arg("serve").arg(component);
    with_limits(&mut command, limits);
    command
}

/// The command that serves an `expression-dialect` component.
#[must_use]
pub fn serve_expression_command(runner: &Path, component: &Path, limits: GuestLimits) -> Command {
    let mut command = Command::new(runner);
    command
        .arg("serve")
        .arg(component)
        .arg("--world")
        .arg("expression-dialect");
    with_limits(&mut command, limits);
    command
}

/// The command that serves an `output-target` component.
#[must_use]
pub fn serve_output_command(runner: &Path, component: &Path, limits: GuestLimits) -> Command {
    let mut command = Command::new(runner);
    command
        .arg("serve")
        .arg(component)
        .arg("--world")
        .arg("output-target");
    with_limits(&mut command, limits);
    command
}

fn with_limits(command: &mut Command, limits: GuestLimits) {
    command
        .arg("--fuel")
        .arg(vize_s0::cstr!("{}", limits.fuel_per_call).as_str())
        .arg("--memory")
        .arg(vize_s0::cstr!("{}", limits.max_memory_bytes).as_str());
}

impl OutOfProcessGuest {
    /// Spawn `command` and wait for the child to report the guest loaded.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with the child's exact load error, or
    /// [`GuestError::Transport`] when the child cannot be spawned or exits
    /// before it is ready.
    pub fn spawn(mut command: Command) -> Result<Self, GuestError> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| transport(&error))?;
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().expect("stdout is piped"));
        let mut guest = Self {
            child,
            stdin,
            stdout,
        };
        match guest.receive()? {
            Response::Ready => Ok(guest),
            Response::LoadError(message) => Err(GuestError::Instantiate(message)),
            other => Err(unexpected(&other)),
        }
    }

    fn receive(&mut self) -> Result<Response, GuestError> {
        match read_message(&mut self.stdout) {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Err(GuestError::Transport(cstr!(
                "the guest process closed its output"
            ))),
            Err(error) => Err(transport(&error)),
        }
    }

    fn call(&mut self, request: &Request) -> Result<Response, GuestError> {
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(GuestError::Transport(cstr!("the guest process is closed")));
        };
        write_message(stdin, request).map_err(|error| match error.kind() {
            ErrorKind::BrokenPipe => GuestError::Transport(cstr!("the guest process exited")),
            _ => transport(&error),
        })?;
        match self.receive()? {
            Response::Guest(error) => Err(error),
            response => Ok(response),
        }
    }
}

fn unexpected(response: &Response) -> GuestError {
    let name = match response {
        Response::Ready => "ready",
        Response::LoadError(_) => "load-error",
        Response::Capability(_) => "capability",
        Response::LoweredBlock(_) => "lowered-block",
        Response::Analysis(_) => "analysis",
        Response::Emitted(_) => "emitted",
        Response::Guest(_) => "guest",
    };
    GuestError::Transport(cstr!("unexpected `{name}` response"))
}

impl InputDialectGuest for OutOfProcessGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        match self.call(&Request::GetCapability)? {
            Response::Capability(capability) => Ok(capability),
            other => Err(unexpected(&other)),
        }
    }

    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError> {
        let request = Request::LowerBlock {
            block: block.clone(),
        };
        match self.call(&request)? {
            Response::LoweredBlock(lowered) => Ok(lowered),
            other => Err(unexpected(&other)),
        }
    }
}

impl ExpressionDialectGuest for OutOfProcessGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        InputDialectGuest::get_capability(self)
    }

    fn analyze(&mut self, batch: &ExpressionBatch) -> Result<Analysis, GuestError> {
        let request = Request::Analyze {
            batch: batch.clone(),
        };
        match self.call(&request)? {
            Response::Analysis(analysis) => Ok(analysis),
            other => Err(unexpected(&other)),
        }
    }
}

impl OutputTargetGuest for OutOfProcessGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        InputDialectGuest::get_capability(self)
    }

    fn emit(&mut self, request: &EmitRequest) -> Result<Emitted, GuestError> {
        let request = Request::Emit {
            request: request.clone(),
        };
        match self.call(&request)? {
            Response::Emitted(emitted) => Ok(emitted),
            other => Err(unexpected(&other)),
        }
    }
}

impl Drop for OutOfProcessGuest {
    fn drop(&mut self) {
        // The child holds no state worth a graceful exit, and a guest stuck
        // in a loop must not hang the host: close its input, kill it, and
        // reap it so no zombie outlives the host.
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
