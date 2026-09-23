//! Volt-shaped output target (P6-6). One probe's S2 and S3 pages become one
//! HEEx module. Any other request traps. This is the exercise guest, not
//! Volt itself.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use vize_extension_sdk as _;

wit_bindgen::generate!({
    path: "../../contracts/wit",
    world: "output-target",
});

use exports::vize::contracts::emission::{EmitRequest, Emitted, Guest as Emission};
use exports::vize::contracts::handshake::{Capability, Guest as Handshake};
use vize::contracts::types::Page;

const S2: &str = include_str!("../fixtures/probe.s2.folio");
const S3: &str = include_str!("../fixtures/probe.s3.folio");
const DOCUMENT: &str = include_str!("../fixtures/probe.emit.folio");

struct Volt;

impl Handshake for Volt {
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

impl Emission for Volt {
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

export!(Volt);
