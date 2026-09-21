//! Component hosting under wasmtime (`extension-host` feature only).
//!
//! The `input-dialect` world imports no host function (its only import is
//! the types-only `vize:contracts/types` instance), so the linker is empty:
//! a guest that asks for any host capability — WASI included — fails to
//! instantiate. Guests are pure functions of the block they are given.

use std::path::Path;

use vize_s0::{String, cstr};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::contract::{
    Capability, Diagnostic, DiagnosticPart, GuestError, InputDialectGuest, LoweredBlock, Page,
    PartKind, Severity, SourceBlock, Span, Stage, Witness,
};

mod bindings {
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
    store: Store<()>,
    instance: bindings::InputDialect,
}

impl core::fmt::Debug for WasmGuest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WasmGuest")
    }
}

fn instantiate_error(error: &wasmtime::Error) -> GuestError {
    GuestError::Instantiate(cstr!("{error:#}"))
}

fn trap(error: &wasmtime::Error) -> GuestError {
    GuestError::Trap(cstr!("{error:#}"))
}

impl WasmGuest {
    /// Compile and instantiate the component at `path`.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load(path: &Path) -> Result<Self, GuestError> {
        let mut config = Config::new();
        // Trap messages cross the contract verbatim, so they carry no wasm
        // backtrace: its code offsets change with every guest build.
        config.wasm_backtrace_max_frames(None);
        let engine = Engine::new(&config).map_err(|error| instantiate_error(&error))?;
        let component =
            Component::from_file(&engine, path).map_err(|error| instantiate_error(&error))?;
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance = bindings::InputDialect::instantiate(&mut store, &component, &linker)
            .map_err(|error| instantiate_error(&error))?;
        Ok(Self { store, instance })
    }
}

impl InputDialectGuest for WasmGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        let offer = self
            .instance
            .vize_contracts_handshake()
            .call_get_capability(&mut self.store)
            .map_err(|error| trap(&error))?;
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
        let wit_lowering::LoweredBlock {
            surface,
            semantic,
            diagnostics,
        } = self
            .instance
            .vize_contracts_input_lowering()
            .call_lower_block(&mut self.store, &request)
            .map_err(|error| trap(&error))?;
        Ok(LoweredBlock {
            surface: page(surface),
            semantic: page(semantic),
            diagnostics: diagnostics.into_iter().map(diagnostic).collect(),
        })
    }
}

fn page(page: wit::Page) -> Page {
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

fn diagnostic(diagnostic: wit::Diagnostic) -> Diagnostic {
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
