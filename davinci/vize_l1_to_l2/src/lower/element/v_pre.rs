//! Literal names on either side of Vue's v-pre boundary.
//!
//! Before the opening v-pre, the full authored head is retained. After it and
//! in descendants, the existing L1 head provider removes only the longhand
//! argument separator. Quotes, brackets and every modifier remain opaque.

pub use vize_l1::markup::directive::frozen_attribute_name;
