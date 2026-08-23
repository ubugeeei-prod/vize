#![allow(dead_code)]

use serde_json::Value;

pub struct RawInitialize;
pub struct RawSetContentMapperContributions;
pub struct RawSignatureHelp;

macro_rules! raw_request {
    ($request:ty, $method:literal) => {
        impl lsp_types::request::Request for $request {
            type Params = Value;
            type Result = Value;
            const METHOD: &'static str = $method;
        }
    };
}

raw_request!(RawInitialize, "initialize");
raw_request!(
    RawSetContentMapperContributions,
    "custom/setContentMapperContributions"
);
raw_request!(RawSignatureHelp, "textDocument/signatureHelp");

pub struct RawInitialized;

impl lsp_types::notification::Notification for RawInitialized {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}
