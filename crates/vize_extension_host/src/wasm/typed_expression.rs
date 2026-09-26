//! In-process hosting of `typed-expression-dialect` guests, under the same
//! sandbox (empty linker, fuel, memory ceiling) as input-dialect guests.

use std::path::Path;

use vize_l0::String;

use super::{Sandbox, diagnostic, instantiate_error, page};
use crate::contract::{Capability, GuestError, GuestLimits, Span};
use crate::expression::Analysis;
use crate::typed_expression::{TypedExpressionBatch, TypedExpressionGuest};

mod bindings {
    wasmtime::component::bindgen!({
        path: "../vize_extension_sdk/wit",
        world: "typed-expression-dialect",
        with: {
            "vize:contracts/types": crate::wasm::bindings::vize::contracts::types,
        },
    });
}

use bindings::exports::vize::contracts::typed_expression_analysis as wit;

/// An typed-expression-dialect component guest instantiated in this process.
pub struct WasmTypedExpressionGuest {
    sandbox: Sandbox,
    instance: bindings::TypedExpressionDialect,
}

impl core::fmt::Debug for WasmTypedExpressionGuest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WasmTypedExpressionGuest")
    }
}

impl WasmTypedExpressionGuest {
    /// Compile and instantiate the component at `path` under `limits`.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load_with(path: &Path, limits: GuestLimits) -> Result<Self, GuestError> {
        let (mut sandbox, component, linker) = Sandbox::load(path, limits)?;
        let instance =
            bindings::TypedExpressionDialect::instantiate(&mut sandbox.store, &component, &linker)
                .map_err(|error| instantiate_error(&error))?;
        Ok(Self { sandbox, instance })
    }
}

impl TypedExpressionGuest for WasmTypedExpressionGuest {
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

    fn analyze_typed(&mut self, batch: &TypedExpressionBatch) -> Result<Analysis, GuestError> {
        let request = wit::TypedExpressionBatch {
            environment: batch
                .environment
                .iter()
                .map(|binding| wit::TypedBinding {
                    name: binding.name.as_str().into(),
                    kind: binding.kind.as_str().into(),
                    signature: binding.signature.as_str().into(),
                })
                .collect(),
            expressions: batch
                .expressions
                .iter()
                .map(|expression| wit::TypedExpression {
                    id: expression.id,
                    source: expression.source.as_str().into(),
                    span: span(expression.span),
                    locals: expression
                        .locals
                        .iter()
                        .map(|binding| wit::TypedBinding {
                            name: binding.name.as_str().into(),
                            kind: binding.kind.as_str().into(),
                            signature: binding.signature.as_str().into(),
                        })
                        .collect(),
                    expected: demand(expression.expected),
                })
                .collect(),
        };
        self.sandbox.arm()?;
        let answer = self
            .instance
            .vize_contracts_typed_expression_analysis()
            .call_analyze_typed(&mut self.sandbox.store, &request);
        let answer = answer.map_err(|error| self.sandbox.failure(&error))?;
        let facts = answer.facts;
        let projection = answer.projection;
        let diagnostics = answer.diagnostics;
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

fn demand(demand: crate::typed_expression::Demand) -> wit::Demand {
    use crate::typed_expression::Demand;
    match demand {
        Demand::Value => wit::Demand::Value,
        Demand::Show => wit::Demand::Show,
        Demand::Condition => wit::Demand::Condition,
        Demand::Name => wit::Demand::Name,
        Demand::Handler => wit::Demand::Handler,
        Demand::Statement => wit::Demand::Statement,
    }
}
