//! Component tags that resolve to a script binding
//! (`CodegenContext::resolve_component_binding` /
//! `push_component_binding_tag`, non-inline): the tag, its camelized and
//! PascalCased spellings, with props bindings kept only as a fallback,
//! and a dotted `Foo.Bar` suffix carried through.

use vize_s0::{String, camelize, capitalize};

use super::super::EmitCx;
use super::super::helper::Helper;
use super::super::options::{BindingKind, BindingTable};

pub(in crate::emit) struct ComponentBinding<'a> {
    pub(in crate::emit) name: String,
    pub(in crate::emit) kind: BindingKind,
    pub(in crate::emit) suffix: Option<&'a str>,
}

/// The binding a component tag resolves to, if the table names one.
pub(in crate::emit) fn resolve<'a>(
    table: &BindingTable,
    component: &'a str,
) -> Option<ComponentBinding<'a>> {
    let (base, suffix) = match component.split_once('.') {
        Some((base, suffix)) => (base, Some(suffix)),
        None => (component, None),
    };
    let (name, kind) = resolve_base(table, base)?;
    Some(ComponentBinding { name, kind, suffix })
}

/// Whether the emit resolves `component` through the binding table.
pub(in crate::emit) fn resolves(cx: &EmitCx<'_>, component: &str) -> bool {
    cx.scope
        .bindings()
        .is_some_and(|table| resolve(table, component).is_some())
}

/// Push the `$setup.Name` (`.suffix`) tag; `false` when the tag is not a
/// binding and the caller falls back to the resolved asset.
pub(in crate::emit) fn push_tag(cx: &mut EmitCx<'_>, component: &str) -> bool {
    let Some(binding) = cx
        .scope
        .bindings()
        .and_then(|table| resolve(table, component))
    else {
        return false;
    };
    // `push_component_binding_tag`: an inlined render function reads the
    // binding straight from the closure, through `_unref` when the script
    // may have rebound it.
    let inline = cx.scope.inline();
    let needs_unref = inline
        && matches!(
            binding.kind,
            BindingKind::SetupLet | BindingKind::SetupMaybeRef | BindingKind::SetupRef
        );
    if needs_unref {
        cx.buf.use_helper(Helper::Unref);
        cx.buf.push(Helper::Unref.alias());
        cx.buf.push("(");
    }
    if !inline {
        cx.buf.push("$setup.");
    }
    cx.buf.push(binding.name.as_str());
    if needs_unref {
        cx.buf.push(")");
    }
    if let Some(suffix) = binding.suffix {
        cx.buf.push(".");
        cx.buf.push(suffix);
    }
    true
}

fn resolve_base(table: &BindingTable, name: &str) -> Option<(String, BindingKind)> {
    let mut prop_fallback = None;
    if let Some(binding) = candidate(table, String::from(name), &mut prop_fallback) {
        return Some(binding);
    }
    let camel = camelize(name);
    if camel.as_str() != name
        && let Some(binding) = candidate(table, camel.clone(), &mut prop_fallback)
    {
        return Some(binding);
    }
    let pascal = capitalize(&camel);
    if pascal.as_str() != name
        && pascal.as_str() != camel.as_str()
        && let Some(binding) = candidate(table, pascal, &mut prop_fallback)
    {
        return Some(binding);
    }
    prop_fallback
}

fn candidate(
    table: &BindingTable,
    candidate: String,
    prop_fallback: &mut Option<(String, BindingKind)>,
) -> Option<(String, BindingKind)> {
    let kind = table.kind(candidate.as_str())?;
    if kind.is_props() {
        if prop_fallback.is_none() {
            *prop_fallback = Some((candidate, kind));
        }
        return None;
    }
    Some((candidate, kind))
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use crate::emit::options::{BindingKind, BindingTable};

    #[test]
    fn tags_resolve_exact_camel_pascal_with_props_fallback() {
        let table = BindingTable::new(
            [
                ("MyComp", BindingKind::SetupConst),
                ("fooBar", BindingKind::ExternalModule),
                ("Icon", BindingKind::Props),
                ("Ns", BindingKind::SetupConst),
            ],
            [],
            true,
        );
        assert_eq!(resolve(&table, "MyComp").unwrap().name.as_str(), "MyComp");
        assert_eq!(resolve(&table, "my-comp").unwrap().name.as_str(), "MyComp");
        assert_eq!(resolve(&table, "foo-bar").unwrap().name.as_str(), "fooBar");
        let icon = resolve(&table, "icon").unwrap();
        assert_eq!(icon.name.as_str(), "Icon");
        assert_eq!(icon.kind, BindingKind::Props);
        let dotted = resolve(&table, "Ns.Item").unwrap();
        assert_eq!(dotted.name.as_str(), "Ns");
        assert_eq!(dotted.suffix, Some("Item"));
        assert!(resolve(&table, "other").is_none());
    }
}
