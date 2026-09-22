//! Component hosting under wasmtime (`extension-host` feature only).
//!
//! The `input-dialect` world imports no host function (its only import is
//! the types-only `vize:contracts/types` instance), so the linker is empty:
//! a guest that asks for any host capability — WASI included — fails to
//! instantiate. Guests are pure functions of the block they are given.
//!
//! Every call runs under [`GuestLimits`]: a fresh fuel budget (a runaway
//! guest is stopped with [`GuestError::OutOfFuel`]) and a ceiling on linear
//! memory (a hoarding guest is stopped with [`GuestError::MemoryLimit`]).

use std::path::Path;

use vize_s0::{String, cstr};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, ResourceLimiter, Store, Trap};

use crate::contract::{
    Capability, Diagnostic, DiagnosticPart, GuestError, GuestLimits, InputDialectGuest,
    LoweredBlock, Page, PartKind, Severity, SourceBlock, Span, Stage, Witness,
};

mod expression;

pub use expression::WasmExpressionGuest;

pub(crate) mod bindings {
    wasmtime::component::bindgen!({
        path: "../../contracts/wit",
        world: "input-dialect",
    });
}

use bindings::exports::vize::contracts::{
    handshake as wit_handshake, input_lowering as wit_lowering,
};
use bindings::vize::contracts::types as wit;

/// A component guest instantiated in this process.
pub struct WasmGuest {
    sandbox: Sandbox,
    instance: bindings::InputDialect,
}

/// One guest's store under its limits, shared by every world's host.
pub(crate) struct Sandbox {
    pub(crate) store: Store<MemoryCeiling>,
    limits: GuestLimits,
}

/// Denies memory growth past the limit and remembers that it did, so the
/// trap that follows is reported as the limit, not as the guest's own fault.
pub(crate) struct MemoryCeiling {
    max_bytes: usize,
    denied: bool,
}

impl ResourceLimiter for MemoryCeiling {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let allowed = desired <= self.max_bytes;
        self.denied |= !allowed;
        Ok(allowed)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(true)
    }
}

impl core::fmt::Debug for WasmGuest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WasmGuest")
    }
}

pub(crate) fn instantiate_error(error: &wasmtime::Error) -> GuestError {
    GuestError::Instantiate(cstr!("{error:#}"))
}

fn trap(error: &wasmtime::Error) -> GuestError {
    GuestError::Trap(cstr!("{error:#}"))
}

impl WasmGuest {
    /// Compile and instantiate the component at `path` under the default
    /// [`GuestLimits`].
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load(path: &Path) -> Result<Self, GuestError> {
        Self::load_with(path, GuestLimits::default())
    }

    /// Compile and instantiate the component at `path` under `limits`.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load_with(path: &Path, limits: GuestLimits) -> Result<Self, GuestError> {
        let (mut sandbox, component, linker) = Sandbox::load(path, limits)?;
        let instance = bindings::InputDialect::instantiate(&mut sandbox.store, &component, &linker)
            .map_err(|error| instantiate_error(&error))?;
        Ok(Self { sandbox, instance })
    }
}

impl Sandbox {
    /// Compile the component at `path` into a fresh store under `limits`.
    pub(crate) fn load(
        path: &Path,
        limits: GuestLimits,
    ) -> Result<(Self, Component, Linker<MemoryCeiling>), GuestError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        // Trap messages cross the contract verbatim, so they carry no wasm
        // backtrace: its code offsets change with every guest build.
        config.wasm_backtrace_max_frames(None);
        let engine = Engine::new(&config).map_err(|error| instantiate_error(&error))?;
        let component =
            Component::from_file(&engine, path).map_err(|error| instantiate_error(&error))?;
        let linker = Linker::new(&engine);
        let ceiling = MemoryCeiling {
            max_bytes: usize::try_from(limits.max_memory_bytes).unwrap_or(usize::MAX),
            denied: false,
        };
        let mut store = Store::new(&engine, ceiling);
        store.limiter(|ceiling| ceiling);
        store
            .set_fuel(limits.fuel_per_call)
            .map_err(|error| instantiate_error(&error))?;
        Ok((Self { store, limits }, component, linker))
    }

    /// Grant a fresh budget before a call.
    pub(crate) fn arm(&mut self) -> Result<(), GuestError> {
        self.store.data_mut().denied = false;
        self.store
            .set_fuel(self.limits.fuel_per_call)
            .map_err(|error| trap(&error))
    }

    /// Classify a failed call: a limit the host enforced, or a guest trap.
    pub(crate) fn failure(&self, error: &wasmtime::Error) -> GuestError {
        if error.downcast_ref::<Trap>() == Some(&Trap::OutOfFuel) {
            GuestError::OutOfFuel {
                budget: self.limits.fuel_per_call,
            }
        } else if self.store.data().denied {
            GuestError::MemoryLimit {
                limit: self.limits.max_memory_bytes,
            }
        } else {
            trap(error)
        }
    }
}

impl InputDialectGuest for WasmGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        self.sandbox.arm()?;
        let offer = self
            .instance
            .vize_contracts_handshake()
            .call_get_capability(&mut self.sandbox.store);
        let offer = offer.map_err(|error| self.sandbox.failure(&error))?;
        let wit_handshake::Capability {
            protocol_version,
            features,
        } = offer;
        Ok(Capability {
            protocol_version,
            features: features.iter().map(|f| String::from(f.as_str())).collect(),
        })
    }

    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError> {
        let request = wit_lowering::SourceBlock {
            source: block.source.as_str().into(),
            base: block.base,
            lang: block.lang.as_ref().map(|lang| lang.as_str().into()),
        };
        self.sandbox.arm()?;
        let answer = self
            .instance
            .vize_contracts_input_lowering()
            .call_lower_block(&mut self.sandbox.store, &request);
        let wit_lowering::LoweredBlock {
            surface,
            semantic,
            diagnostics,
        } = answer.map_err(|error| self.sandbox.failure(&error))?;
        Ok(LoweredBlock {
            surface: page(surface),
            semantic: page(semantic),
            diagnostics: diagnostics.into_iter().map(diagnostic).collect(),
        })
    }
}

pub(crate) fn page(page: wit::Page) -> Page {
    let wit::Page {
        schema_version,
        text,
    } = page;
    Page {
        schema_version,
        text: String::from(text.as_str()),
    }
}

fn span(span: wit::Span) -> Span {
    let wit::Span { start, end } = span;
    Span { start, end }
}

pub(crate) fn diagnostic(diagnostic: wit::Diagnostic) -> Diagnostic {
    let wit::Diagnostic {
        severity,
        stage,
        span: primary,
        message,
        parts,
        witness,
    } = diagnostic;
    Diagnostic {
        severity: match severity {
            wit::Severity::Error => Severity::Error,
            wit::Severity::Warning => Severity::Warning,
            wit::Severity::Info => Severity::Info,
            wit::Severity::Hint => Severity::Hint,
        },
        stage: match stage {
            wit::Stage::Source => Stage::Source,
            wit::Stage::Surface => Stage::Surface,
            wit::Stage::Semantic => Stage::Semantic,
            wit::Stage::Lowered => Stage::Lowered,
            wit::Stage::Emit => Stage::Emit,
        },
        span: span(primary),
        message: String::from(message.as_str()),
        parts: parts.into_iter().map(part).collect(),
        witness: witness.map(|witness| match witness {
            wit::Witness::LegacyExempt(producer) => {
                Witness::LegacyExempt(String::from(producer.as_str()))
            }
        }),
    }
}

fn part(part: wit::DiagnosticPart) -> DiagnosticPart {
    let wit::DiagnosticPart {
        kind,
        span: at,
        message,
    } = part;
    DiagnosticPart {
        kind: match kind {
            wit::PartKind::Primary => PartKind::Primary,
            wit::PartKind::Secondary => PartKind::Secondary,
            wit::PartKind::Help => PartKind::Help,
            wit::PartKind::Suggestion => PartKind::Suggestion,
        },
        span: span(at),
        message: String::from(message.as_str()),
    }
}
