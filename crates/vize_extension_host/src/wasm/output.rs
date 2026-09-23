//! In-process hosting of `output-target` guests, under the same sandbox
//! (empty linker, fuel, memory ceiling) as the other worlds.

use std::path::Path;

use vize_s0::String;

use super::{Sandbox, diagnostic, instantiate_error, page};
use crate::contract::{Capability, GuestError, GuestLimits};
use crate::output::{EmitRequest, Emitted, OutputTargetGuest};

mod bindings {
    wasmtime::component::bindgen!({
        path: "../vize_extension_sdk/wit",
        world: "output-target",
        with: {
            "vize:contracts/types": crate::wasm::bindings::vize::contracts::types,
        },
    });
}

use bindings::exports::vize::contracts::emission as wit;

/// An output-target component guest instantiated in this process.
pub struct WasmOutputGuest {
    sandbox: Sandbox,
    instance: bindings::OutputTarget,
}

impl core::fmt::Debug for WasmOutputGuest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WasmOutputGuest")
    }
}

impl WasmOutputGuest {
    /// Compile and instantiate the component at `path` under `limits`.
    ///
    /// # Errors
    ///
    /// [`GuestError::Instantiate`] with wasmtime's message.
    pub fn load_with(path: &Path, limits: GuestLimits) -> Result<Self, GuestError> {
        let (mut sandbox, component, linker) = Sandbox::load(path, limits)?;
        let instance = bindings::OutputTarget::instantiate(&mut sandbox.store, &component, &linker)
            .map_err(|error| instantiate_error(&error))?;
        Ok(Self { sandbox, instance })
    }
}

impl OutputTargetGuest for WasmOutputGuest {
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
                .map(|feature| String::from(feature.as_str()))
                .collect(),
        })
    }

    fn emit(&mut self, request: &EmitRequest) -> Result<Emitted, GuestError> {
        let request = wit::EmitRequest {
            s2: wit_page(&request.s2),
            s3: wit_page(&request.s3),
        };
        self.sandbox.arm()?;
        let answer = self
            .instance
            .vize_contracts_emission()
            .call_emit(&mut self.sandbox.store, &request);
        let wit::Emitted {
            document,
            diagnostics,
        } = answer.map_err(|error| self.sandbox.failure(&error))?;
        Ok(Emitted {
            document: page(document),
            diagnostics: diagnostics.into_iter().map(diagnostic).collect(),
        })
    }
}

fn wit_page(page: &crate::contract::Page) -> wit::Page {
    wit::Page {
        schema_version: page.schema_version,
        text: page.text.as_str().into(),
    }
}
