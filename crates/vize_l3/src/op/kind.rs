/// The 16 operation kinds generalized from Vapor's current flat operation set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpKind {
    SetProp = 0,
    SetDynamicProps = 1,
    SetText = 2,
    SetEvent = 3,
    SetHtml = 4,
    SetTemplateRef = 5,
    InsertNode = 6,
    PrependNode = 7,
    Directive = 8,
    If = 9,
    For = 10,
    CreateComponent = 11,
    SlotOutlet = 12,
    GetTextChild = 13,
    ChildRef = 14,
    NextRef = 15,
}

impl OpKind {
    /// Stable stage-wide mnemonic.
    #[must_use]
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::SetProp => "l3.set-prop",
            Self::SetDynamicProps => "l3.set-dynamic-props",
            Self::SetText => "l3.set-text",
            Self::SetEvent => "l3.set-event",
            Self::SetHtml => "l3.set-html",
            Self::SetTemplateRef => "l3.set-template-ref",
            Self::InsertNode => "l3.insert-node",
            Self::PrependNode => "l3.prepend-node",
            Self::Directive => "l3.directive",
            Self::If => "l3.if",
            Self::For => "l3.for",
            Self::CreateComponent => "l3.create-component",
            Self::SlotOutlet => "l3.slot-outlet",
            Self::GetTextChild => "l3.get-text-child",
            Self::ChildRef => "l3.child-ref",
            Self::NextRef => "l3.next-ref",
        }
    }

    /// Parse a stable stage-wide mnemonic.
    #[must_use]
    pub const fn from_mnemonic(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"l3.set-prop" => Some(Self::SetProp),
            b"l3.set-dynamic-props" => Some(Self::SetDynamicProps),
            b"l3.set-text" => Some(Self::SetText),
            b"l3.set-event" => Some(Self::SetEvent),
            b"l3.set-html" => Some(Self::SetHtml),
            b"l3.set-template-ref" => Some(Self::SetTemplateRef),
            b"l3.insert-node" => Some(Self::InsertNode),
            b"l3.prepend-node" => Some(Self::PrependNode),
            b"l3.directive" => Some(Self::Directive),
            b"l3.if" => Some(Self::If),
            b"l3.for" => Some(Self::For),
            b"l3.create-component" => Some(Self::CreateComponent),
            b"l3.slot-outlet" => Some(Self::SlotOutlet),
            b"l3.get-text-child" => Some(Self::GetTextChild),
            b"l3.child-ref" => Some(Self::ChildRef),
            b"l3.next-ref" => Some(Self::NextRef),
            _ => None,
        }
    }
}
