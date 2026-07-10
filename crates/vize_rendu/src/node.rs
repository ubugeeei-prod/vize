//! Owned render-node vocabulary.

use crate::{RenduExpressionId, RenduNodeId, RenduProperty, RenduProvenance};

/// A static name or an expression that resolves a name at render time.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenduName {
    Static(Box<str>),
    Dynamic(RenduExpressionId),
}

impl RenduName {
    pub fn static_name(name: impl Into<Box<str>>) -> Self {
        Self::Static(name.into())
    }
}

/// Namespace of a host element. Custom values allow non-web renderers without
/// baking their frontend or backend crate into Rendu.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenduNamespace {
    Html,
    Svg,
    MathMl,
    Custom(Box<str>),
}

/// Whether a rendered expression is escaped or emitted raw.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum RenduEscapeMode {
    #[default]
    Escaped,
    Raw,
}

/// A binding pattern introduced by a slot or loop.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduBinding {
    pub pattern: Box<str>,
    pub provenance: RenduProvenance,
}

impl RenduBinding {
    pub fn new(pattern: impl Into<Box<str>>) -> Self {
        Self {
            pattern: pattern.into(),
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn with_provenance(mut self, provenance: RenduProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}

/// One ordered branch of an `if` node. `None` is the final else branch.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduIfBranch {
    pub condition: Option<RenduExpressionId>,
    pub body: Vec<RenduNodeId>,
    pub provenance: RenduProvenance,
}

impl RenduIfBranch {
    pub fn new(condition: Option<RenduExpressionId>, body: Vec<RenduNodeId>) -> Self {
        Self {
            condition,
            body,
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn with_provenance(mut self, provenance: RenduProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}

/// Render HIR node. Child relationships use typed arena indices so the root is
/// owned, compact, and independent of every producer AST.
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenduNode {
    Fragment {
        children: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    Element {
        tag: Box<str>,
        namespace: RenduNamespace,
        properties: Vec<RenduProperty>,
        children: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    Component {
        name: RenduName,
        properties: Vec<RenduProperty>,
        children: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    SlotOutlet {
        name: RenduName,
        properties: Vec<RenduProperty>,
        fallback: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    SlotContent {
        name: RenduName,
        bindings: Vec<RenduBinding>,
        children: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    Text {
        value: Box<str>,
        provenance: RenduProvenance,
    },
    Expression {
        expression: RenduExpressionId,
        escape: RenduEscapeMode,
        provenance: RenduProvenance,
    },
    Comment {
        value: Box<str>,
        provenance: RenduProvenance,
    },
    If {
        branches: Vec<RenduIfBranch>,
        provenance: RenduProvenance,
    },
    For {
        source: RenduExpressionId,
        value: RenduBinding,
        key: Option<RenduBinding>,
        index: Option<RenduBinding>,
        key_expression: Option<RenduExpressionId>,
        body: Vec<RenduNodeId>,
        provenance: RenduProvenance,
    },
    HoistRef {
        index: u32,
        provenance: RenduProvenance,
    },
}

impl RenduNode {
    pub fn provenance(&self) -> &RenduProvenance {
        match self {
            Self::Fragment { provenance, .. }
            | Self::Element { provenance, .. }
            | Self::Component { provenance, .. }
            | Self::SlotOutlet { provenance, .. }
            | Self::SlotContent { provenance, .. }
            | Self::Text { provenance, .. }
            | Self::Expression { provenance, .. }
            | Self::Comment { provenance, .. }
            | Self::If { provenance, .. }
            | Self::For { provenance, .. }
            | Self::HoistRef { provenance, .. } => provenance,
        }
    }

    pub(crate) fn visit_children(&self, mut visit: impl FnMut(RenduNodeId)) {
        match self {
            Self::Fragment { children, .. }
            | Self::Element { children, .. }
            | Self::Component { children, .. }
            | Self::SlotContent { children, .. } => children.iter().copied().for_each(&mut visit),
            Self::SlotOutlet { fallback, .. } => fallback.iter().copied().for_each(visit),
            Self::If { branches, .. } => branches
                .iter()
                .flat_map(|branch| branch.body.iter().copied())
                .for_each(visit),
            Self::For { body, .. } => body.iter().copied().for_each(visit),
            Self::Text { .. }
            | Self::Expression { .. }
            | Self::Comment { .. }
            | Self::HoistRef { .. } => {}
        }
    }
}
