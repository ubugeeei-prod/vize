use vize_rendu::{
    RenduEscapeMode, RenduExpressionKind, RenduNamespace, RenduProvenance, RenduSource,
};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            pub const fn index(self) -> usize {
                self.0 as usize
            }

            pub const fn raw(self) -> u32 {
                self.0
            }

            pub(super) fn from_index(index: usize) -> Self {
                Self(u32::try_from(index).expect("Vapor plan arena exceeds u32::MAX entries"))
            }
        }
    };
}

define_id!(VaporBlockId);
define_id!(VaporExpressionId);

/// Fully owned output of Rendu-to-Vapor planning.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporPlan {
    pub(crate) sources: Vec<RenduSource>,
    pub(crate) expressions: Vec<VaporExpression>,
    pub(crate) blocks: Vec<VaporBlock>,
    pub(crate) entry: VaporBlockId,
}

impl VaporPlan {
    pub fn sources(&self) -> &[RenduSource] {
        &self.sources
    }

    pub fn expressions(&self) -> &[VaporExpression] {
        &self.expressions
    }

    pub fn blocks(&self) -> &[VaporBlock] {
        &self.blocks
    }

    pub const fn entry(&self) -> VaporBlockId {
        self.entry
    }

    pub fn expression(&self, id: VaporExpressionId) -> Option<&VaporExpression> {
        self.expressions.get(id.index())
    }

    pub fn block(&self, id: VaporBlockId) -> Option<&VaporBlock> {
        self.blocks.get(id.index())
    }
}

/// Producer expression copied into the plan so it can outlive its Rendu root.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporExpression {
    pub code: Box<str>,
    pub kind: RenduExpressionKind,
    pub provenance: RenduProvenance,
}

/// One independently mountable or reactive operation block.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporBlock {
    pub operations: Vec<VaporOperation>,
    pub provenance: RenduProvenance,
}

/// Owned name used by component, slot, and property operations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VaporName {
    Static(Box<str>),
    Dynamic(VaporExpressionId),
}

/// Loop- or slot-local binding pattern.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporBinding {
    pub pattern: Box<str>,
    pub provenance: RenduProvenance,
}

/// Attribute material after frontend lowering.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VaporAttributeValue {
    Static(Box<str>),
    Expression(VaporExpressionId),
}

/// Preserved custom directive for target-specific Vapor lowering.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporDirective {
    pub name: Box<str>,
    pub argument: Option<VaporName>,
    pub expression: Option<VaporExpressionId>,
    pub modifiers: Vec<Box<str>>,
    pub provenance: RenduProvenance,
}

/// Property plan independent of SFC, JSX, and parser AST types.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VaporProperty {
    Attribute {
        name: VaporName,
        value: Option<VaporAttributeValue>,
        provenance: RenduProvenance,
    },
    Directive(VaporDirective),
    Spread {
        expression: VaporExpressionId,
        provenance: RenduProvenance,
    },
}

impl VaporProperty {
    pub fn provenance(&self) -> &RenduProvenance {
        match self {
            Self::Attribute { provenance, .. } | Self::Spread { provenance, .. } => provenance,
            Self::Directive(directive) => &directive.provenance,
        }
    }
}

/// One conditional arm with a separately mountable body.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporBranch {
    pub condition: Option<VaporExpressionId>,
    pub body: VaporBlockId,
    pub provenance: RenduProvenance,
}

/// Frontend-neutral Vapor mount and reactive-update operations.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VaporOperation {
    /// A subtree with no runtime dependencies, serialized once as a template.
    StaticHtml {
        html: Box<str>,
        provenance: RenduProvenance,
    },
    Fragment {
        body: VaporBlockId,
        provenance: RenduProvenance,
    },
    Element {
        tag: Box<str>,
        namespace: RenduNamespace,
        properties: Vec<VaporProperty>,
        body: VaporBlockId,
        provenance: RenduProvenance,
    },
    Component {
        name: VaporName,
        properties: Vec<VaporProperty>,
        body: VaporBlockId,
        provenance: RenduProvenance,
    },
    SlotOutlet {
        name: VaporName,
        properties: Vec<VaporProperty>,
        fallback: VaporBlockId,
        provenance: RenduProvenance,
    },
    SlotContent {
        name: VaporName,
        bindings: Vec<VaporBinding>,
        body: VaporBlockId,
        provenance: RenduProvenance,
    },
    DynamicText {
        expression: VaporExpressionId,
        escape: RenduEscapeMode,
        provenance: RenduProvenance,
    },
    Conditional {
        branches: Vec<VaporBranch>,
        provenance: RenduProvenance,
    },
    Iterate {
        source: VaporExpressionId,
        value: VaporBinding,
        key: Option<VaporBinding>,
        index: Option<VaporBinding>,
        key_expression: Option<VaporExpressionId>,
        body: VaporBlockId,
        provenance: RenduProvenance,
    },
    HoistRef {
        index: u32,
        provenance: RenduProvenance,
    },
    Unsupported {
        description: Box<str>,
        provenance: RenduProvenance,
    },
}

impl VaporOperation {
    pub fn provenance(&self) -> &RenduProvenance {
        match self {
            Self::StaticHtml { provenance, .. }
            | Self::Fragment { provenance, .. }
            | Self::Element { provenance, .. }
            | Self::Component { provenance, .. }
            | Self::SlotOutlet { provenance, .. }
            | Self::SlotContent { provenance, .. }
            | Self::DynamicText { provenance, .. }
            | Self::Conditional { provenance, .. }
            | Self::Iterate { provenance, .. }
            | Self::HoistRef { provenance, .. }
            | Self::Unsupported { provenance, .. } => provenance,
        }
    }
}
