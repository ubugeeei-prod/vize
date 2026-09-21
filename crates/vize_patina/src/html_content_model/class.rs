//! The violation classes: one per spec clause that can make a rendered
//! template disagree with the document the browser builds from it.

/// Which guarantee a violation breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// The HTML parser restructures the serialized template (§13.2.6): the DOM
    /// differs from the virtual DOM, so SSR hydration mismatches and
    /// `innerHTML`-stringified static content renders differently.
    Parser,
    /// The DOM is built as written but violates a content model (§4): the
    /// document is non-conforming.
    ContentModel,
}

/// A nesting-violation class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViolationClass {
    /// A start tag in the "closes p" set while a `p` is in button scope.
    ParagraphAutoClosed,
    /// `h1`–`h6` directly inside `h1`–`h6`.
    HeadingAutoClosed,
    /// `li`/`dd`/`dt` while the list-item loop reaches an open `li`/`dd`/`dt`.
    ListItemAutoClosed,
    /// `form` while a `form` is open: the start tag is ignored.
    NestedFormDropped,
    /// `a` (or `nobr`) while one is open: the adoption agency reparents it.
    FormattingAdopted,
    /// `button` while a `button` is in scope.
    ButtonAutoClosed,
    /// `select` or `input` while a `select` is in scope.
    SelectAutoClosed,
    /// `option`/`optgroup`/`hr`/`rb`/`rtc`/`rp`/`rt` popping an implied-end element.
    ImpliedEndTagClosed,
    /// A table part outside the table structure that accepts it.
    TablePartMisplaced,
    /// A table part whose required wrapper (`tbody`/`tr`/`colgroup`) the
    /// parser inserts.
    TableWrapperInserted,
    /// Content that closes an open `table`/section/row/`colgroup`.
    TableAutoClosed,
    /// Content foster-parented out of a table.
    FosterParented,
    /// A `form` with children inside table structure: inserted and popped at once.
    FormInTableEmptied,
    /// An HTML-only start tag breaking out of SVG/MathML foreign content.
    ForeignContentBreakout,
    /// `html`/`head`/`body`/`frameset`/`frame` inside body content: ignored.
    DocumentElementDropped,
    /// `image`, renamed to `img` by the parser.
    ImageRenamed,
    /// Element children of a raw-text/RCDATA element, or `plaintext`.
    RawTextContent,
    /// Children of an element the parser pops as soon as it inserts it.
    VoidElementContent,
    /// Non-phrasing content where the content model requires phrasing content.
    PhrasingContentExpected,
    /// Interactive content (or `tabindex`) inside `a`/`button`.
    InteractiveContentNested,
    /// A child the parent's content model does not list.
    ChildNotPermitted,
}

impl ViolationClass {
    /// Every class, in report-priority order.
    pub const ALL: [Self; 21] = [
        Self::ParagraphAutoClosed,
        Self::HeadingAutoClosed,
        Self::ListItemAutoClosed,
        Self::NestedFormDropped,
        Self::FormattingAdopted,
        Self::ButtonAutoClosed,
        Self::SelectAutoClosed,
        Self::ImpliedEndTagClosed,
        Self::TablePartMisplaced,
        Self::TableWrapperInserted,
        Self::TableAutoClosed,
        Self::FosterParented,
        Self::FormInTableEmptied,
        Self::ForeignContentBreakout,
        Self::DocumentElementDropped,
        Self::ImageRenamed,
        Self::RawTextContent,
        Self::VoidElementContent,
        Self::PhrasingContentExpected,
        Self::InteractiveContentNested,
        Self::ChildNotPermitted,
    ];

    /// Stable kebab-case identifier (diagnostic codes, seeded-defect manifests).
    pub const fn id(self) -> &'static str {
        match self {
            Self::ParagraphAutoClosed => "paragraph-auto-closed",
            Self::HeadingAutoClosed => "heading-auto-closed",
            Self::ListItemAutoClosed => "list-item-auto-closed",
            Self::NestedFormDropped => "nested-form-dropped",
            Self::FormattingAdopted => "formatting-adopted",
            Self::ButtonAutoClosed => "button-auto-closed",
            Self::SelectAutoClosed => "select-auto-closed",
            Self::ImpliedEndTagClosed => "implied-end-tag-closed",
            Self::TablePartMisplaced => "table-part-misplaced",
            Self::TableWrapperInserted => "table-wrapper-inserted",
            Self::TableAutoClosed => "table-auto-closed",
            Self::FosterParented => "foster-parented",
            Self::FormInTableEmptied => "form-in-table-emptied",
            Self::ForeignContentBreakout => "foreign-content-breakout",
            Self::DocumentElementDropped => "document-element-dropped",
            Self::ImageRenamed => "image-renamed",
            Self::RawTextContent => "raw-text-content",
            Self::VoidElementContent => "void-element-content",
            Self::PhrasingContentExpected => "phrasing-content-expected",
            Self::InteractiveContentNested => "interactive-content-nested",
            Self::ChildNotPermitted => "child-not-permitted",
        }
    }

    /// The guarantee this class breaks.
    pub const fn family(self) -> Family {
        match self {
            Self::PhrasingContentExpected
            | Self::InteractiveContentNested
            | Self::ChildNotPermitted => Family::ContentModel,
            _ => Family::Parser,
        }
    }

    /// The governing spec clause, as a fragment of
    /// `https://html.spec.whatwg.org/multipage/`.
    pub const fn spec(self) -> &'static str {
        match self {
            Self::ParagraphAutoClosed => "parsing.html#close-a-p-element",
            Self::HeadingAutoClosed
            | Self::ListItemAutoClosed
            | Self::NestedFormDropped
            | Self::ButtonAutoClosed
            | Self::SelectAutoClosed
            | Self::TablePartMisplaced
            | Self::DocumentElementDropped
            | Self::ImageRenamed => "parsing.html#parsing-main-inbody",
            Self::FormattingAdopted => "parsing.html#adoption-agency-algorithm",
            Self::ImpliedEndTagClosed => "parsing.html#generate-implied-end-tags",
            Self::TableWrapperInserted | Self::FormInTableEmptied => {
                "parsing.html#parsing-main-intable"
            }
            Self::TableAutoClosed => "parsing.html#parsing-main-intbody",
            Self::FosterParented => "parsing.html#foster-parent",
            Self::ForeignContentBreakout => "parsing.html#parsing-main-inforeign",
            Self::RawTextContent => "parsing.html#generic-raw-text-element-parsing-algorithm",
            Self::VoidElementContent => "syntax.html#void-elements",
            Self::PhrasingContentExpected => "dom.html#phrasing-content",
            Self::InteractiveContentNested => "dom.html#interactive-content",
            Self::ChildNotPermitted => "dom.html#concept-element-content-model",
        }
    }
}
