//! The TS-48 expression-world guest: answers `analyze` for the committed
//! probe batch with the committed facts and projection pages, byte for byte,
//! and traps on any other batch. The SDK supplies its `no_std` runtime.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

// Links the SDK's runtime (allocator, `cabi_realloc`, `memcmp`, panic
// handler) into this component.
use vize_extension_sdk as _;

wit_bindgen::generate!({
    path: "../../../../vize_extension_sdk/wit",
    world: "expression-dialect",
});

use exports::vize::contracts::expression_analysis::{
    Analysis, ExpressionBatch, Guest as ExpressionAnalysis,
};
use exports::vize::contracts::handshake::{Capability, Guest as Handshake};
use vize::contracts::types::Page;

const FACTS: &str = include_str!("../../../fixtures/expression/probe.facts.folio");
const PROJECTION: &str = include_str!("../../../fixtures/expression/probe.projection.folio");

struct Echo;

impl Handshake for Echo {
    fn get_capability() -> Capability {
        Capability {
            protocol_version: 1,
            features: ["facts-page@1", "projection-page@1"]
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }
}

impl ExpressionAnalysis for Echo {
    fn analyze(batch: ExpressionBatch) -> Analysis {
        let sources: Vec<&str> = batch
            .expressions
            .iter()
            .map(|e| e.source.as_str())
            .collect();
        if sources != ["msg + suffix", "count * 2"] {
            core::arch::wasm32::unreachable()
        }
        Analysis {
            facts: Page {
                schema_version: 1,
                text: String::from(FACTS),
            },
            projection: Page {
                schema_version: 1,
                text: String::from(PROJECTION),
            },
            diagnostics: Vec::new(),
        }
    }
}

export!(Echo);
