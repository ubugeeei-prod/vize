//! [`ForeignExpr`] - the non-JS expression dialect payload.
//!
//! Retained neutral-IR payload for opaque expression dialects and historical
//! dump codecs. No non-JavaScript authoring dialect is currently shipped.
//! Source spans remain exact without invoking the JavaScript parser.

include!("../lang/foreign.rs");
