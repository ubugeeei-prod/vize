//! Owned branch-key facts and extraction from an actual L2 branch carrier.
//!
//! Input lowerings retain grammar-specific wrapper capture and publication.
//! This receiver reads the completed branch itself, with no separate caller
//! span, and returns owned facts for DOM, SSR and refusal consumers. The
//! transitional lowering still owns its side table; this is not an Artifact
//! seal or native source-admission contract.

use alloc::vec::Vec as StdVec;
use vize_l0::{Span, String, Vec};

use crate::expr::ExprRef;
use crate::op::{Attribute, BindOp, BindingOp, DynamicName, IfBranch, Op};

/// One branch's extracted `key`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchKey {
    /// Which spelling carried the key.
    pub kind: BranchKeyKind,
    /// The authored attribute's range.
    pub span: Span,
}

/// The two key spellings a branch carrier can author.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKeyKind {
    /// A static `key` attribute; `None` for a bare `key`, which never
    /// collides.
    Static(Option<String>),
    /// A `:key` binding: the trimmed authored value text, plus the
    /// index of the carrier binding that carries it (`None` for a
    /// wrapper-captured key, which has no op).
    Dynamic {
        /// The trimmed value text (the parser's same-name expansion
        /// applied: a valueless `:key` reads `key`).
        source: String,
        /// The carrier's binding index, for the surface exclusion.
        bind_index: Option<usize>,
    },
}

impl BranchKey {
    /// Authored collision text; attribute and binding kinds remain distinct.
    #[must_use]
    pub fn collision_text(&self) -> Option<&str> {
        match &self.kind {
            BranchKeyKind::Static(value) => value.as_deref(),
            BranchKeyKind::Dynamic { source, .. } if source.is_empty() => None,
            BranchKeyKind::Dynamic { source, .. } => Some(source.as_str()),
        }
    }
}

/// Per-`ui.if` branch-key facts, one slot per branch in authored order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IfFacts {
    /// `branches[i]` is branch `i`'s extracted key, when it had one.
    pub branches: StdVec<Option<BranchKey>>,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<BranchKey>();
    assert_owned::<BranchKeyKind>();
    assert_owned::<IfFacts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BranchKey>() == 48);
    assert!(core::mem::size_of::<IfFacts>() == 24);
};

/// The attribute/binding surface of a branch's carrier op: the branch
/// region's single root when it is an element, component, or slot outlet
/// and it is the branch's own carrier (`op.span == branch_span`).
fn carrier_surface<'w, 'a>(
    branch: &'w mut IfBranch<'a>,
) -> Option<(
    &'w mut Vec<'a, Attribute<'a>>,
    &'w mut Vec<'a, BindingOp<'a>>,
)> {
    let branch_span = branch.span;
    let [op] = branch.region.ops.as_mut_slice() else {
        return None;
    };
    match op {
        Op::Element(element) if element.span == branch_span => {
            let element = &mut **element;
            Some((&mut element.attributes, &mut element.bindings))
        }
        Op::Component(component) if component.span == branch_span => {
            let component = &mut **component;
            Some((&mut component.attributes, &mut component.bindings))
        }
        Op::Slot(slot) if slot.span == branch_span => {
            let slot = &mut **slot;
            Some((&mut slot.attributes, &mut slot.bindings))
        }
        Op::Element(_)
        | Op::Component(_)
        | Op::Slot(_)
        | Op::Text(_)
        | Op::Comment(_)
        | Op::Interpolation(_)
        | Op::JsxChildValue(_)
        | Op::If(_)
        | Op::For(_)
        | Op::OriginalFor(_) => None,
    }
}

/// A `ui.bind` whose argument spells `key` by static name or legacy
/// dynamic-argument content.
/// The binding as a key `ui.bind`, when it is one.
fn key_bind<'b, 'a>(binding: &'b BindingOp<'a>) -> Option<&'b BindOp<'a>> {
    match binding {
        BindingOp::Bind(bind) => match bind.name {
            Some(DynamicName::Static("key")) => Some(bind),
            Some(DynamicName::Dynamic(ExprRef::Js(js))) if js.source == "key" => Some(bind),
            _ => None,
        },
        BindingOp::On(_)
        | BindingOp::Model(_)
        | BindingOp::SlotContent(_)
        | BindingOp::SlotsObject(_)
        | BindingOp::VueDirective(_)
        | BindingOp::VueCssBind(_)
        | BindingOp::VueSync(_)
        | BindingOp::VueSlotScope(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueMemo(_)
        | BindingOp::VueShow(_)
        | BindingOp::VueHtml(_)
        | BindingOp::VueText(_)
        | BindingOp::VueCloak(_) => None,
    }
}

/// Extract the first authored key from this completed branch's own carrier.
///
/// The branch supplies both its region and extent. Static keys leave the
/// attribute surface; dynamic keys retain their original binding and index.
pub fn take_carrier_key(branch: &mut IfBranch<'_>) -> Option<BranchKey> {
    let (attributes, bindings) = carrier_surface(branch)?;
    let static_at = attributes
        .iter()
        .enumerate()
        .find(|(_, attribute)| attribute.name == "key")
        .map(|(index, attribute)| (attribute.span.start, index));
    let dynamic_at = bindings
        .iter()
        .enumerate()
        .find_map(|(index, binding)| key_bind(binding).map(|bind| (bind, index)));
    match (static_at, dynamic_at) {
        (Some((attr_start, index)), Some((bind, _))) if attr_start < bind.span.start => {
            Some(take_static(attributes, index))
        }
        (Some((_, index)), None) => Some(take_static(attributes, index)),
        (_, Some((bind, index))) => Some(read_dynamic(bind, index)),
        (None, None) => None,
    }
}

fn take_static<'a>(attributes: &mut Vec<'a, Attribute<'a>>, index: usize) -> BranchKey {
    let attribute = attributes.remove(index);
    BranchKey {
        kind: BranchKeyKind::Static(attribute.value.map(String::from)),
        span: attribute.span,
    }
}

fn read_dynamic(bind: &BindOp<'_>, index: usize) -> BranchKey {
    let source = bind
        .value
        .as_ref()
        .map(|value| String::from(value.source()))
        .unwrap_or_default();
    BranchKey {
        kind: BranchKeyKind::Dynamic {
            source,
            bind_index: Some(index),
        },
        span: bind.span,
    }
}
