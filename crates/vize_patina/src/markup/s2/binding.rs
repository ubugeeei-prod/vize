//! Binding-op reads for the S2 facade: one normalized view over every
//! attached S2 op, plus the authored S1 attributes S2 consumed or dropped.

use super::S2Markup;
use super::bound::S2Bound;
use super::surface::{SurfaceDirective, attr_span, attr_value};
use crate::markup::MarkupBindingKind;
use vize_s0::Span;
use vize_s2::op::{Attribute, BindingOp};

/// One authored item on an S2 element's opening tag, in authored order.
#[derive(Clone, Copy)]
pub(in crate::markup) enum S2Item<'a> {
    /// A static attribute S2 kept.
    Attribute {
        attribute: &'a Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
    /// An attached binding op S2 kept, with its authored attribute.
    Binding(S2Bound<'a>),
    /// An authored attribute S2 consumed or dropped (a `<slot>` outlet's
    /// `name`, a `v-if` carrier's static `key`, an ill-formed directive, …),
    /// read back off S1. Structural directives never appear: the facade
    /// consumes them into scopes.
    Surface {
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
}

impl<'a> S2Item<'a> {
    pub(in crate::markup) fn span(self) -> Span {
        match self {
            Self::Attribute { attribute, .. } => attribute.span,
            Self::Binding(binding) => binding.span(),
            Self::Surface { attr, doc } => attr_span(doc.source, attr),
        }
    }

    /// Whether the item is a directive spelling rather than a static attribute.
    pub(in crate::markup) fn is_directive(self) -> bool {
        match self {
            Self::Attribute { .. } => false,
            Self::Binding(_) => true,
            Self::Surface { attr, .. } => SurfaceDirective::parse(attr.name.text).is_some(),
        }
    }
}

/// Visit the authored items of an S2 element in authored order.
///
/// With an S1 surface the authored attribute list drives the order and every
/// S1 attribute resolves to the S2 attribute or binding carrying its exact
/// span, or — when S2 consumed or dropped it — to a [`S2Item::Surface`]. A
/// JSX projection has no surface tree and drops nothing (it refuses instead),
/// so its attributes and bindings are merged by span.
///
/// Share the authored-order reconstruction across rules; only the final
/// callback differs. Borrowing the callback does not allocate.
pub(in crate::markup) fn walk_items<'a>(
    doc: &'a S2Markup<'a>,
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    surface: Option<&'a vize_s1::Element<'a>>,
    visitor: &mut dyn FnMut(S2Item<'a>),
) {
    if let Some(element) = surface {
        for attr in &element.open.attrs {
            let span = attr_span(doc.source, attr);
            if let Some(attribute) = attributes.iter().find(|attribute| attribute.span == span) {
                visitor(S2Item::Attribute { attribute, doc });
            } else if let Some(op) = bindings.iter().find(|op| op_span(op) == span) {
                visitor(S2Item::Binding(S2Bound {
                    op,
                    surface: Some(attr),
                }));
            } else if !is_consumed_spelling(attr) {
                visitor(S2Item::Surface { attr, doc });
            }
        }
        return;
    }
    let (mut attribute_index, mut binding_index) = (0usize, 0usize);
    loop {
        let next_attribute = attributes.get(attribute_index);
        let next_binding = bindings.get(binding_index);
        match (next_attribute, next_binding) {
            (Some(attribute), Some(binding)) if attribute.span.start <= op_span(binding).start => {
                visitor(S2Item::Attribute { attribute, doc });
                attribute_index += 1;
            }
            (_, Some(binding)) => {
                visitor(S2Item::Binding(S2Bound {
                    op: binding,
                    surface: None,
                }));
                binding_index += 1;
            }
            (Some(attribute), None) => {
                visitor(S2Item::Attribute { attribute, doc });
                attribute_index += 1;
            }
            (None, None) => break,
        }
    }
}

/// Whether an authored attribute S2 left without an op is a spelling the
/// facade consumes rather than restores: a structural directive (it became a
/// scope), or the `v-pre` that opened a raw subtree (the parser drops it; a
/// nested `v-pre` inside the subtree is a frozen attribute S2 keeps).
pub(in crate::markup) fn is_consumed_spelling(attr: &vize_s1::Attribute<'_>) -> bool {
    attr.name.text == "v-pre"
        || SurfaceDirective::parse(attr.name.text)
            .is_some_and(|directive| directive.is_structural())
}

pub(in crate::markup) fn op_span(binding: &BindingOp<'_>) -> Span {
    match binding {
        BindingOp::Bind(bind) => bind.span,
        BindingOp::On(on) => on.span,
        BindingOp::Model(model) => model.span,
        BindingOp::SlotContent(content) => content.span,
        BindingOp::VueDirective(directive) => directive.span,
        BindingOp::VueCssBind(bind) => bind.span,
        BindingOp::VueSync(sync) => sync.span,
        BindingOp::VueSlotScope(scope) => scope.span,
        BindingOp::VueOnce(once) => once.span,
        BindingOp::VueMemo(memo) => memo.span,
        BindingOp::VueShow(show) => show.span,
        BindingOp::VueHtml(html) => html.span,
        BindingOp::VueText(text) => text.span,
        BindingOp::VueCloak(cloak) => cloak.span,
    }
}

/// The normalized binding class for a directive name.
pub(in crate::markup) fn kind_of_directive(name: &str) -> MarkupBindingKind {
    match name {
        "bind" => MarkupBindingKind::Bind,
        "on" => MarkupBindingKind::On,
        "model" => MarkupBindingKind::Model,
        _ => MarkupBindingKind::Custom,
    }
}

/// The trimmed authored value of a surface directive, when non-blank.
///
/// A valueless `v-bind` with a static argument reads the parser's same-name
/// expansion (`:name` ⇒ `name`). S2 only consumes such a spelling where the
/// argument is a plain identifier (a `<slot :name>` outlet's name), so the
/// camelized form is the argument slice itself.
pub(in crate::markup) fn surface_expression<'a>(
    attr: &'a vize_s1::Attribute<'a>,
    directive: &SurfaceDirective<'a>,
) -> Option<&'a str> {
    match attr_value(attr) {
        Some(value) => Some(value.trim()).filter(|text| !text.is_empty()),
        None if directive.name == "bind" && directive.arg_static => directive
            .arg
            .filter(|arg| !arg.contains('-') && attr.eq.is_none()),
        None => None,
    }
}
