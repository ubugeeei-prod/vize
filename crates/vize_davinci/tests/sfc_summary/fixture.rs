//! The `Button` interface the summary tests share, and the canonical pages.

use vize_davinci::summary::{AlphaEntry, AlphaPages, Facet, SfcSummary, Signature, Usage};
use vize_s0::String;

pub const BARE: &str = "\
[sfc-summary]
schema_version=1
component-signature=1
prop-types=1
emit-types=1
slot-types=1
reactivity-classes=1
component-references=1

[sfc-summary.component-signature]
name=Bare
params=

";

pub const BUTTON: &str = "\
[sfc-summary]
schema_version=1
component-signature=1
prop-types=1
emit-types=1
slot-types=1
reactivity-classes=1
component-references=1

[sfc-summary.component-signature]
name=Button
params=

[sfc-summary.prop-types]
disabled=boolean
label=string

[sfc-summary.emit-types]
click=MouseEvent

[sfc-summary.slot-types]
default=

[sfc-summary.reactivity-classes]
count=ref
label=shallowRef

[sfc-summary.component-references]
./Icon.vue=default

";

pub fn entry(name: &str, contract: &str) -> AlphaEntry {
    AlphaEntry {
        name: String::from(name),
        contract: String::from(contract),
    }
}

pub fn signature(name: &str, params: &str) -> Signature {
    Signature {
        name: String::from(name),
        params: String::from(params),
    }
}

/// The interface α pages of `Button`. The component body is not a field.
pub fn button_pages() -> AlphaPages {
    AlphaPages {
        signature: signature("Button", ""),
        props: vec![entry("label", "string"), entry("disabled", "boolean")],
        emits: vec![entry("click", "MouseEvent")],
        slots: vec![entry("default", "")],
        reactivity: vec![entry("count", "ref"), entry("label", "shallowRef")],
        components: vec![entry("./Icon.vue", "default")],
    }
}

pub fn summarize(pages: &AlphaPages) -> SfcSummary {
    SfcSummary::from_alpha(pages.clone()).expect("fixture pages are a valid interface")
}

pub fn users(summary: &SfcSummary) -> Vec<Usage> {
    vec![
        Usage::record("shell", summary, &[(Facet::Signature, "Button")]).expect("signature"),
        Usage::record("header", summary, &[(Facet::Signature, "Button")]).expect("signature"),
        Usage::record(
            "card",
            summary,
            &[(Facet::Prop, "label"), (Facet::Emit, "click")],
        )
        .expect("label and click"),
        Usage::record("toggle", summary, &[(Facet::Prop, "disabled")]).expect("disabled"),
        Usage::record("gallery", summary, &[(Facet::Component, "./Icon.vue")]).expect("Icon"),
        Usage::record("layout", summary, &[(Facet::Slot, "default")]).expect("default slot"),
        Usage::record("idle", summary, &[]).expect("records nothing"),
    ]
}

pub fn replace_once(text: &str, from: &str, to: &str) -> String {
    let at = text.find(from).expect("pattern");
    let mut out = String::from(&text[..at]);
    out.push_str(to);
    out.push_str(&text[at + from.len()..]);
    out
}
