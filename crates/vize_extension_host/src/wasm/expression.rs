//! In-process hosting of `expression-dialect` guests, under the same
//! sandbox (empty linker, fuel, memory ceiling) as input-dialect guests.

use std::path::Path;

use vize_s0::String;

use super::{Sandbox, diagnostic, instantiate_error, page};
use crate::contract::{Capability, GuestError, GuestLimits, Span};
use crate::expression::{Analysis, ExpressionBatch, ExpressionDialectGuest};

mod bindings {
    wasmtime::component::bindgen!({
        path: "../vize_extension_sdk/wit",
        world: "expression-dialect",
        with: {
            "vize:contracts/types": crate::wasm::bindings::vize::contracts::types,
        },
    });
}

use bindings::exports::vize::contracts::expression_analysis as wit;

/// An expression-dialect component guest instantiated in this process.
pub struct WasmExpressionGuest {
    sandbox: Sandbox,
    instance: bindings::ExpressionDialect,
}

impl core::fmt::Debug for WasmExpressionGuest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WasmExpressionGuest")
    }
}

impl WasmExpressionGuest {
    /// Compile and instantiate the component at `path` under `limits`.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load_with(path: &Path, limits: GuestLimits) -> Result<Self, GuestError> {
        let (mut sandbox, component, linker) = Sandbox::load(path, limits)?;
        let instance =
            bindings::ExpressionDialect::instantiate(&mut sandbox.store, &component, &linker)
                .map_err(|error| instantiate_error(&error))?;
        Ok(Self { sandbox, instance })
    }
}

impl ExpressionDialectGuest for WasmExpressionGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        self.sandbox.arm()?;
        let offer = self
            .instance
            .vize_contracts_handshake()
            .call_get_capability(&mut self.sandbox.store);
        let offer = offer.map_err(|error| self.sandbox.failure(&error))?;
        Ok(Capability {
            protocol_version: offer.protocol_version,
            features: offer
                .features
                .iter()
                .map(|f| String::from(f.as_str()))
                .collect(),
        })
    }

    fn analyze(&mut self, batch: &ExpressionBatch) -> Result<Analysis, GuestError> {
        let request = wit::ExpressionBatch {
            environment: batch
                .environment
                .iter()
                .map(|binding| wit::Binding {
                    name: binding.name.as_str().into(),
                    kind: binding.kind.as_str().into(),
                })
                .collect(),
            expressions: batch
                .expressions
                .iter()
                .map(|expression| wit::Expression {
                    id: expression.id,
                    source: expression.source.as_str().into(),
                    span: span(expression.span),
                })
                .collect(),
        };
        self.sandbox.arm()?;
        let answer = self
            .instance
            .vize_contracts_expression_analysis()
            .call_analyze(&mut self.sandbox.store, &request);
        let wit::Analysis {
            facts,
            projection,
            diagnostics,
        } = answer.map_err(|error| self.sandbox.failure(&error))?;
        Ok(Analysis {
            facts: page(facts),
            projection: page(projection),
            diagnostics: diagnostics.into_iter().map(diagnostic).collect(),
        })
    }
}

fn span(span: Span) -> wit::Span {
    wit::Span {
        start: span.start,
        end: span.end,
    }
}
