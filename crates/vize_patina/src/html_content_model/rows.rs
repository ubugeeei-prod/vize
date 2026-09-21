//! The names of the fact-table rows the checker reads.
//!
//! Each [`Row`] names one `(kind, name)` row of `whatwg.tsv`; the loader
//! requires the table to contain exactly these rows, once each.

/// Every row the checker reads, by `(kind, name)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Row {
    Special,
    Scope,
    ButtonScope,
    Marker,
    ImpliedEnd,
    ClosesP,
    Heading,
    ListItemLoopTransparent,
    TablePart,
    DocumentPart,
    RawText,
    ScriptingDependent,
    Void,
    ForeignBreakout,
    MathmlTextIntegration,
    HtmlIntegration,
    TableChildren,
    TableWrapped,
    SectionChildren,
    SectionWrapped,
    TrChildren,
    ColgroupChildren,
    Transparent,
    CatPhrasing,
    CatHeading,
    CatInteractive,
    CatScriptSupporting,
}

pub(super) const ROWS: [(Row, &str, &str); 27] = [
    (Row::Special, "set", "special"),
    (Row::Scope, "set", "scope"),
    (Row::ButtonScope, "set", "button-scope"),
    (Row::Marker, "set", "marker"),
    (Row::ImpliedEnd, "set", "implied-end"),
    (Row::ClosesP, "set", "closes-p"),
    (Row::Heading, "set", "heading"),
    (
        Row::ListItemLoopTransparent,
        "set",
        "list-item-loop-transparent",
    ),
    (Row::TablePart, "set", "table-part"),
    (Row::DocumentPart, "set", "document-part"),
    (Row::RawText, "set", "raw-text"),
    (Row::ScriptingDependent, "set", "scripting-dependent"),
    (Row::Void, "set", "void"),
    (Row::ForeignBreakout, "set", "foreign-breakout"),
    (Row::MathmlTextIntegration, "set", "mathml-text-integration"),
    (Row::HtmlIntegration, "set", "html-integration"),
    (Row::TableChildren, "set", "table-children"),
    (Row::TableWrapped, "set", "table-wrapped"),
    (Row::SectionChildren, "set", "section-children"),
    (Row::SectionWrapped, "set", "section-wrapped"),
    (Row::TrChildren, "set", "row-children"),
    (Row::ColgroupChildren, "set", "colgroup-children"),
    (Row::Transparent, "set", "transparent"),
    (Row::CatPhrasing, "category", "phrasing"),
    (Row::CatHeading, "category", "heading"),
    (Row::CatInteractive, "category", "interactive"),
    (Row::CatScriptSupporting, "category", "script-supporting"),
];
