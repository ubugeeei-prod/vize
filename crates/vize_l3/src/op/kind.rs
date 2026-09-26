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
            Self::SetProp => "impeto.set-prop",
            Self::SetDynamicProps => "impeto.set-dynamic-props",
            Self::SetText => "impeto.set-text",
            Self::SetEvent => "impeto.set-event",
            Self::SetHtml => "impeto.set-html",
            Self::SetTemplateRef => "impeto.set-template-ref",
            Self::InsertNode => "impeto.insert-node",
            Self::PrependNode => "impeto.prepend-node",
            Self::Directive => "impeto.directive",
            Self::If => "impeto.if",
            Self::For => "impeto.for",
            Self::CreateComponent => "impeto.create-component",
            Self::SlotOutlet => "impeto.slot-outlet",
            Self::GetTextChild => "impeto.get-text-child",
            Self::ChildRef => "impeto.child-ref",
            Self::NextRef => "impeto.next-ref",
        }
    }

    /// Parse a stable stage-wide mnemonic.
    #[must_use]
    pub const fn from_mnemonic(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"impeto.set-prop" => Some(Self::SetProp),
            b"impeto.set-dynamic-props" => Some(Self::SetDynamicProps),
            b"impeto.set-text" => Some(Self::SetText),
            b"impeto.set-event" => Some(Self::SetEvent),
            b"impeto.set-html" => Some(Self::SetHtml),
            b"impeto.set-template-ref" => Some(Self::SetTemplateRef),
            b"impeto.insert-node" => Some(Self::InsertNode),
            b"impeto.prepend-node" => Some(Self::PrependNode),
            b"impeto.directive" => Some(Self::Directive),
            b"impeto.if" => Some(Self::If),
            b"impeto.for" => Some(Self::For),
            b"impeto.create-component" => Some(Self::CreateComponent),
            b"impeto.slot-outlet" => Some(Self::SlotOutlet),
            b"impeto.get-text-child" => Some(Self::GetTextChild),
            b"impeto.child-ref" => Some(Self::ChildRef),
            b"impeto.next-ref" => Some(Self::NextRef),
            _ => None,
        }
    }
}
