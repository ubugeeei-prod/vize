//! The TS-48 output-world guest: answers `emit` for the committed probe
//! pages with the committed emit document, byte for byte, and traps on any
//! other request. The SDK supplies its `no_std` runtime.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

// Links the SDK's runtime (allocator, `cabi_realloc`, `memcmp`, panic
// handler) into this component.
use vize_extension_sdk as _;

wit_bindgen::generate!({
    path: "../../../../vize_extension_sdk/wit",
    world: "output-target",
});

use exports::vize::contracts::emission::{EmitRequest, Emitted, Guest as Emission};
use exports::vize::contracts::handshake::{Capability, Guest as Handshake};
use vize::contracts::types::Page;

const S2: &str = include_str!("../../../fixtures/output/probe.s2.folio");
const S3: &str = include_str!("../../../fixtures/output/probe.s3.folio");
const DOCUMENT: &str = include_str!("../../../fixtures/output/probe.emit.folio");

struct Echo;

impl Handshake for Echo {
    fn get_capability() -> Capability {
        Capability {
            protocol_version: 1,
            features: ["emit-document-page@1", "s2-page@1", "s3-page@1"]
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }
}

impl Emission for Echo {
    fn emit(request: EmitRequest) -> Emitted {
        if request.s2.text != S2 || request.s3.text != S3 {
            core::arch::wasm32::unreachable()
        }
        Emitted {
            document: Page {
                schema_version: 1,
                text: String::from(DOCUMENT),
            },
            diagnostics: Vec::new(),
        }
    }
}

export!(Echo);
