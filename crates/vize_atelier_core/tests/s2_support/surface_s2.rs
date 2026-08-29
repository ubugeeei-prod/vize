//! The S2 half of the surface projection ([`super::surface`]): one
//! owner's attribute and binding lists into the shared owner shape,
//! split from [`super::s2_lane`] under the source budget. Binding ids
//! are positional (owner line, then bindings in order), which is how
//! the fault table resolves and the branch-key exclusion lands.

use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::folio::{FolioAttribute, FolioBinding, FolioExpr, FolioName, FolioOp};

use super::s2_lane::{S2Projection, Tables};
use super::surface::{PBind, PDirective, PModel, PName, PSurface, is_simple_ident};

fn expr_text(expr: &FolioExpr) -> String {
    match expr {
        FolioExpr::Js { source, .. }
        | FolioExpr::Foreign { source, .. }
        | FolioExpr::Opaque { source, .. }
        | FolioExpr::Filter { source, .. } => String::from(source.trim()),
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
                let prop = if component {
                    Some(
                        model
                            .argument
                            .as_ref()
                            .map(p_name)
                            .unwrap_or_else(|| PName::Static("modelValue".into())),
                    )
                } else {
                    model.argument.as_ref().map(p_name)
                };
                surface.models.push(PModel {
                    value: Some(expr_text(&model.contract.read)),
                    prop,
                    mods: model
                        .attributes
                        .iter()
                        .filter(|attribute| !matches!(attribute.name.as_str(), "element-kind"))
                        .map(|attribute| attribute.name.as_str().into())
                        .collect(),
                    component,
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
            // Codegen-only dialect bindings: represented on S2, not part of
            // the bind/on/model/directive surface this projection
            // compares. Legacy still counts them under builtins_excluded.
            FolioBinding::VueOnce(_)
            | FolioBinding::VueMemo(_)
            | FolioBinding::VueShow(_)
            | FolioBinding::VueHtml(_)
            | FolioBinding::VueText(_) => {}
            // Pre-pass only: the legacy pass desugars these before the
            // comparator runs. Arms exist so a missed desugar is a
            // surface mismatch, never a compile-time silence.
            FolioBinding::VueSync(sync) => surface.binds.push(PBind {
                name: PName::Static(sync.name.as_str().into()),
                mods: sync.modifiers.iter().map(|m| m.as_str().into()).collect(),
                value: Some(Some(expr_text(&sync.value))),
            }),
            FolioBinding::VueSlotScope(_) => {}
        }
    }
    fold_sync_products(&mut surface);
    surface
}

/// Vue 2 `.sync` legalizes to a bind + `update:` listener that share
/// the authored span. The legacy collector reconstructs that product
/// as a model (span-shared bind/on). After legalize, S2 has the same
/// pair as ordinary bind+on; fold them back so the projection is
/// lane-neutral. The handler text is the legalize product, not a
/// user-authored `@update:` listener.
fn fold_sync_products(surface: &mut PSurface) {
    let mut bind_index = 0;
    while bind_index < surface.binds.len() {
        let PName::Static(prop) = &surface.binds[bind_index].name else {
            bind_index += 1;
            continue;
        };
        let prop = prop.clone();
        let Some(Some(value)) = surface.binds[bind_index].value.clone() else {
            bind_index += 1;
            continue;
        };
        let mut want_on = String::from("update:");
        want_on.push_str(prop.as_str());
        let mut want_handler = String::from("$event => ((");
        want_handler.push_str(value.as_str());
        want_handler.push_str(") = $event)");
        let Some(on_index) = surface.ons.iter().position(|on| {
            matches!(&on.name, PName::Static(name) if name.as_str() == want_on.as_str())
                && on
                    .value
                    .as_ref()
                    .and_then(|inner| inner.as_ref().map(String::as_str))
                    == Some(want_handler.as_str())
        }) else {
            bind_index += 1;
            continue;
        };
        let _bind = surface.binds.remove(bind_index);
        let _on = surface.ons.remove(on_index);
        surface.models.push(PModel {
            value: Some(value),
            prop: Some(PName::Static(prop)),
            // Legacy reconstructs the span-shared product without the
            // leftover bind modifiers (`.camel` rides a stub product
            // bind the S2 legalize does not emit). Empty matches that.
            mods: Vec::new(),
            component: true,
        });
    }
}

pub fn p_name(name: &FolioName) -> PName {
    match name {
        FolioName::Static(text) => PName::Static(text.as_str().into()),
        FolioName::Dynamic(expr) => PName::Dynamic(Some(expr_text(expr))),
    }
}
