//! The S2 half of the surface projection ([`super::surface`]): one
//! owner's attribute and binding lists into the shared owner shape,
//! split from [`super::s2_lane`] under the source budget. Binding ids
//! are positional (owner line, then bindings in order), which is how
//! the fault table resolves and the branch-key exclusion lands.

use vize_carton::String;
use vize_davinci::id::NodeId;
use vize_disegno::folio::{FolioAttribute, FolioBinding, FolioExpr, FolioName, FolioOp};

use super::s2_lane::{S2Projection, Tables};
use super::surface::{PBind, PDirective, PModel, PName, PSurface, is_simple_ident};

fn expr_text(expr: &FolioExpr) -> String {
    match expr {
        FolioExpr::Js { source, .. }
        | FolioExpr::Foreign { source, .. }
        | FolioExpr::Opaque { source, .. } => String::from(source.trim()),
    }
}

/// Whether an owner's first params-bearing `ui.slot-content` opens a
/// destructuring-params scope over its children (the shared predicate;
/// the S2 twin of the legacy `enter_v_slot_scope_if_needed` mirror).
pub fn opens_pattern_scope(bindings: &[FolioBinding], children: &[FolioOp]) -> bool {
    if children.is_empty() {
        return false;
    }
    bindings
        .iter()
        .find_map(|binding| match binding {
            FolioBinding::SlotContent(content) => content.params.as_ref().map(|params| {
                let text = expr_text(params);
                !text.is_empty() && !is_simple_ident(text.as_str())
            }),
            _ => None,
        })
        .unwrap_or(false)
}

/// One owner's surface from its attribute and binding lists.
#[allow(clippy::too_many_arguments)]
pub fn surface_of(
    attributes: &[FolioAttribute],
    bindings: &[FolioBinding],
    owner_index: u32,
    tables: &Tables<'_>,
    pattern_scoped: bool,
    excluded_bind: Option<usize>,
    component: bool,
    out: &mut S2Projection,
) -> PSurface {
    let mut surface = PSurface {
        pattern_scoped,
        ..PSurface::default()
    };
    for attribute in attributes {
        surface
            .attrs
            .push((attribute.name.clone(), attribute.value.clone()));
    }
    for (index, binding) in bindings.iter().enumerate() {
        if Some(index) == excluded_bind {
            out.keys_excluded += 1;
            continue;
        }
        let id = NodeId::from_index(owner_index + 1 + u32::try_from(index).expect("fits"));
        match binding {
            FolioBinding::Bind(bind) => surface.binds.push(PBind {
                name: match &bind.name {
                    None => PName::Spread,
                    Some(name) => p_name(name),
                },
                mods: bind.modifiers.iter().map(|m| m.as_str().into()).collect(),
                value: bind.value.as_ref().map(|value| Some(expr_text(value))),
            }),
            FolioBinding::On(on) => surface.ons.push(PBind {
                name: match &on.name {
                    None => PName::Spread,
                    Some(name) => p_name(name),
                },
                mods: on.modifiers.iter().map(|m| m.as_str().into()).collect(),
                value: on.handler.as_ref().map(|handler| Some(expr_text(handler))),
            }),
            FolioBinding::Model(model) => {
                if id.is_some_and(|id| tables.model_faults.get(id).is_some()) {
                    out.models_invalid += 1;
                    continue;
                }
                let argument = model.attributes.iter().find_map(|attribute| {
                    (attribute.name.as_str() == "argument")
                        .then(|| attribute.value.clone())
                        .flatten()
                });
                let prop = if component {
                    Some(argument.unwrap_or_else(|| "modelValue".into()))
                } else {
                    argument
                };
                surface.models.push(PModel {
                    value: Some(expr_text(&model.contract.read)),
                    prop,
                    mods: model
                        .attributes
                        .iter()
                        .filter(|attribute| {
                            !matches!(attribute.name.as_str(), "element-kind" | "argument")
                        })
                        .map(|attribute| attribute.name.as_str().into())
                        .collect(),
                    component,
                    dynamic_arg: false,
                });
            }
            FolioBinding::SlotContent(_) => {}
            FolioBinding::VueDirective(directive) => surface.directives.push(PDirective {
                name: directive.name.as_str().into(),
                arg: directive.argument.as_ref().map(p_name),
                mods: directive
                    .modifiers
                    .iter()
                    .map(|m| m.as_str().into())
                    .collect(),
                value: directive.value.as_ref().map(|value| Some(expr_text(value))),
            }),
            FolioBinding::VueCssBind(_) => {}
        }
    }
    surface
}

pub fn p_name(name: &FolioName) -> PName {
    match name {
        FolioName::Static(text) => PName::Static(text.as_str().into()),
        FolioName::Dynamic(expr) => PName::Dynamic(Some(expr_text(expr))),
    }
}
