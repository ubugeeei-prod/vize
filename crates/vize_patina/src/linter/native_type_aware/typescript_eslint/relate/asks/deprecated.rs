//! no-deprecated asks: the deprecated uses, members and keys whose symbols the relation pass checks.
use super::context::{call_like, inside_import, is_declaration};
use super::{Ancestor, Walk, key, kind, span};
use crate::linter::corsa_session::{DeprecationAsk, DeprecationKind};
use serde_json::{Value, json};
use vize_l0::String as CompactString;

impl<'v> Walk<'_, '_, 'v> {
    pub(super) fn deprecation(&mut self, value: &Value, kind: Option<DeprecationKind>) {
        let Some(key) = key(value) else {
            return;
        };
        let ask = kind.and_then(|kind| {
            let node = (self.map)(span(value)?)?;
            Some(DeprecationAsk { node, kind })
        });
        match ask {
            Some(ask) => {
                self.pending.deprecations.push(key);
                self.pending.request.deprecations.push(ask);
            }
            None => self.pending.fixed.push((key, "__deprecated", json!(false))),
        }
    }

    /// typescript-eslint `no-deprecated`'s `Identifier`, `PrivateIdentifier`
    /// and `Super` listeners.
    pub(super) fn deprecated_use(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let Some(&(parent, field)) = ancestors.last() else {
            return;
        };
        let skip = match kind(parent) {
            "ExportNamedDeclaration" | "ExportAllDeclaration" => true,
            "MemberExpression" => {
                field == "property" && parent.get("computed").and_then(Value::as_bool) == Some(true)
            }
            "ExportSpecifier" => field != "exported",
            _ => false,
        };
        if skip || is_declaration(field, ancestors) || inside_import(ancestors) {
            self.deprecation(value, None);
            return;
        }
        let deprecation = (|| {
            if kind(parent) == "ExportSpecifier" {
                return Some(DeprecationKind::ExportAlias);
            }
            if let Some(call) = call_like(ancestors) {
                return Some(DeprecationKind::CallLike {
                    call: (self.map)(span(call)?)?,
                });
            }
            if kind(parent) == "JSXAttribute" {
                return None;
            }
            if kind(parent) == "Property" && kind(value) != "Super" {
                let (object, _) = *ancestors.iter().rev().nth(1)?;
                return Some(DeprecationKind::ObjectProperty {
                    object: (self.map)(span(object)?)?,
                    name: CompactString::from(value.get("name")?.as_str()?),
                });
            }
            Some(DeprecationKind::Symbol)
        })();
        self.deprecation(value, deprecation);
    }

    /// `checkMemberExpression`: `object[literal]`.
    pub(super) fn deprecated_member(&mut self, value: &Value) {
        if value.get("computed").and_then(Value::as_bool) != Some(true) {
            return;
        }
        let deprecation = (|| {
            Some(DeprecationKind::Member {
                object: (self.map)(span(value.get("object")?)?)?,
                property: (self.map)(span(value.get("property")?)?)?,
            })
        })();
        self.deprecation_on(value, value.get("property"), deprecation);
    }

    /// `checkObjectLiteralProperty`: a key of an object literal names a
    /// property of the literal's contextual type.
    pub(super) fn deprecated_key(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let Some(&(parent, _)) = ancestors.last() else {
            return;
        };
        if kind(parent) != "ObjectExpression" {
            return;
        }
        let Some(key_node) = value.get("key") else {
            return;
        };
        let computed = value.get("computed").and_then(Value::as_bool) == Some(true);
        let name = if computed {
            None
        } else if kind(key_node) == "Identifier" {
            key_node
                .get("name")
                .and_then(Value::as_str)
                .map(CompactString::from)
        } else {
            match key_node.get("value") {
                Some(Value::String(text)) => Some(CompactString::from(text.as_str())),
                Some(Value::Number(number)) => Some(vize_l0::cstr!("{number}")),
                _ => None,
            }
        };
        let deprecation = (|| {
            let key = if computed {
                Some((self.map)(span(key_node)?)?)
            } else {
                None
            };
            if name.is_none() && key.is_none() {
                return None;
            }
            Some(DeprecationKind::ContextualProperty {
                object: (self.map)(span(parent)?)?,
                name: name.clone(),
                key,
            })
        })();
        if let (Some(key), Some(name)) = (key(value), &name) {
            self.pending
                .fixed
                .push((key, "__deprecatedName", json!(name.as_str())));
        }
        self.deprecation_on(value, Some(key_node), deprecation);
    }

    /// A deprecation reported on `node`'s `reported` child; the reported
    /// name is that child's static text.
    pub(super) fn deprecation_on(
        &mut self,
        value: &Value,
        reported: Option<&Value>,
        kind: Option<DeprecationKind>,
    ) {
        if let (Some(key), Some(range)) = (key(value), reported.and_then(span)) {
            self.pending.fixed.push((
                key,
                "__deprecationReportRange",
                json!({ "start": range.0, "end": range.1 }),
            ));
        }
        self.deprecation(value, kind);
    }
}
