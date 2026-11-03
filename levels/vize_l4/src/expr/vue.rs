//! Vue accessor policy beside the generic span writer.

use alloc::vec::Vec;
use vize_l0::{String, ToCompactString};
use vize_l2::resolution::{BindingId, Occurrence, Usage};

use crate::runtime::{Helper, Vocabulary};

use super::{AccessError, AccessProvider, AccessSpelling};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessStyle {
    Function,
    Inline,
    Server,
}

/// Facts supplied by the Vue script/template binder, never inferred by L4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access<'a> {
    SetupConst,
    SetupLet,
    SetupMaybeRef,
    SetupRef,
    Props,
    PropsAliased { property: &'a str },
    Data,
    Options,
    Context,
    Local,
    Global,
}

#[derive(Debug, Clone, Copy)]
pub struct Binding<'a> {
    pub id: BindingId,
    pub access: Access<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingError {
    UnorderedOrDuplicateIdentity,
}

struct Entry<'a> {
    binding: Binding<'a>,
    quoted_alias: Option<String>,
}

/// Stable identity lookup and checked Vue accessor spelling for one render.
///
/// Prop aliases are escaped once as complete quoted property keys. The caller
/// passes the exact target vocabulary, so helper IDs remain runtime-local.
pub struct VueAccess<'a> {
    entries: Vec<Entry<'a>>,
    style: AccessStyle,
    unref: Option<Helper>,
}

impl<'a> VueAccess<'a> {
    pub fn checked(
        bindings: &[Binding<'a>],
        style: AccessStyle,
        vocabulary: &Vocabulary,
    ) -> Result<Self, BindingError> {
        if bindings
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left.id >= right.id))
        {
            return Err(BindingError::UnorderedOrDuplicateIdentity);
        }
        let entries = bindings
            .iter()
            .map(|binding| Entry {
                binding: *binding,
                quoted_alias: match binding.access {
                    Access::PropsAliased { property } => Some(quote_property(property)),
                    _ => None,
                },
            })
            .collect();
        Ok(Self {
            entries,
            style,
            unref: vocabulary.helper("unref"),
        })
    }
}

impl AccessProvider for VueAccess<'_> {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        let index = self
            .entries
            .binary_search_by_key(&occurrence.binding, |entry| entry.binding.id)
            .map_err(|_| AccessError::MissingBinding)?;
        let entry = self.entries.get(index).ok_or(AccessError::MissingBinding)?;
        let access = entry.binding.access;
        let writes = occurrence.usage != Usage::Read;
        if writes
            && matches!(
                access,
                Access::SetupConst | Access::Props | Access::PropsAliased { .. } | Access::Global
            )
        {
            return Err(AccessError::UnsupportedUsage);
        }
        let rewrite = |prefix, replacement, suffix, helper| AccessSpelling::Rewrite {
            prefix,
            replacement,
            suffix,
            helper,
        };
        match access {
            Access::Local | Access::Global => Ok(AccessSpelling::Verbatim),
            Access::SetupConst if self.style == AccessStyle::Inline => Ok(AccessSpelling::Verbatim),
            Access::SetupRef if self.style == AccessStyle::Inline => {
                Ok(rewrite("", None, ".value", None))
            }
            Access::SetupMaybeRef if self.style == AccessStyle::Inline && writes => {
                Ok(rewrite("", None, ".value", None))
            }
            Access::SetupLet if self.style == AccessStyle::Inline && writes => {
                // Correct let/ref assignment needs the complete parent and RHS;
                // a name-only splice must not turn it into an unconditional ref.
                Err(AccessError::UnsupportedUsage)
            }
            Access::SetupLet | Access::SetupMaybeRef if self.style == AccessStyle::Inline => {
                let helper = self.unref.ok_or(AccessError::MissingHelper)?;
                Ok(if occurrence.constructor {
                    rewrite("(_unref(", None, "))", Some(helper))
                } else {
                    rewrite("_unref(", None, ")", Some(helper))
                })
            }
            Access::SetupConst | Access::SetupLet | Access::SetupMaybeRef | Access::SetupRef => {
                Ok(rewrite("$setup.", None, "", None))
            }
            Access::Props => Ok(rewrite(
                if self.style == AccessStyle::Inline {
                    "__props."
                } else {
                    "$props."
                },
                None,
                "",
                None,
            )),
            Access::PropsAliased { .. } => Ok(rewrite(
                if self.style == AccessStyle::Inline {
                    "__props["
                } else {
                    "$props["
                },
                entry.quoted_alias.as_deref(),
                "]",
                None,
            )),
            Access::Data => Ok(rewrite(
                if self.style == AccessStyle::Inline {
                    "_ctx."
                } else {
                    "$data."
                },
                None,
                "",
                None,
            )),
            Access::Options => Ok(rewrite(
                if self.style == AccessStyle::Inline {
                    "_ctx."
                } else {
                    "$options."
                },
                None,
                "",
                None,
            )),
            Access::Context => Ok(rewrite("_ctx.", None, "", None)),
        }
    }
}

fn quote_property(property: &str) -> String {
    // JSON's string serialization is total for Rust UTF-8 strings. Using the
    // Value's formatter avoids an impossible fallible serialization branch.
    serde_json::Value::from(property).to_compact_string()
}
