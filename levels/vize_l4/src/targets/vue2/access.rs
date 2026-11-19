//! Checked instance spelling for caller-owned actual binding identities.

use crate::expr::{
    AccessError, AccessProvider, AccessSpelling,
    vue::{Access, Binding},
};
use alloc::vec::Vec;
use vize_l0::{String, ToCompactString};
use vize_l2::resolution::{BindingId, Occurrence, Usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue2AccessError {
    UnorderedOrDuplicateIdentity,
    UnsupportedBinding,
}

struct Entry<'a> {
    binding: Binding<'a>,
    alias: Option<String>,
}

pub struct Vue2Access<'a> {
    entries: Vec<Entry<'a>>,
}

impl<'a> Vue2Access<'a> {
    pub fn checked(bindings: &[Binding<'a>]) -> Result<Self, Vue2AccessError> {
        if bindings
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left.id >= right.id))
        {
            return Err(Vue2AccessError::UnorderedOrDuplicateIdentity);
        }
        let mut entries = Vec::with_capacity(bindings.len());
        for &binding in bindings {
            if !matches!(
                binding.access,
                Access::Context
                    | Access::Data
                    | Access::Props
                    | Access::Options
                    | Access::PropsAliased { .. }
                    | Access::Local
                    | Access::Global
            ) {
                return Err(Vue2AccessError::UnsupportedBinding);
            }
            let alias = match binding.access {
                Access::PropsAliased { property } => {
                    Some(serde_json::Value::from(property).to_compact_string())
                }
                _ => None,
            };
            entries.push(Entry { binding, alias });
        }
        Ok(Self { entries })
    }

    pub(super) fn is_verbatim(&self, binding: BindingId) -> bool {
        self.entries
            .binary_search_by_key(&binding, |entry| entry.binding.id)
            .ok()
            .and_then(|index| self.entries.get(index))
            .is_some_and(|entry| matches!(entry.binding.access, Access::Local | Access::Global))
    }

    pub(super) fn for_receiver(&self, receiver: &str) -> RenderAccess<'_, 'a> {
        RenderAccess {
            registry: self,
            dot: vize_l0::cstr!("{receiver}."),
            bracket: vize_l0::cstr!("{receiver}["),
        }
    }

    fn spelling_in<'s>(
        &'s self,
        occurrence: &Occurrence<'_>,
        dot: &'s str,
        bracket: &'s str,
    ) -> Result<AccessSpelling<'s>, AccessError> {
        let index = self
            .entries
            .binary_search_by_key(&occurrence.binding, |entry| entry.binding.id)
            .map_err(|_| AccessError::MissingBinding)?;
        let entry = self.entries.get(index).ok_or(AccessError::MissingBinding)?;
        if occurrence.usage != Usage::Read
            && matches!(
                entry.binding.access,
                Access::Global | Access::Props | Access::PropsAliased { .. }
            )
        {
            return Err(AccessError::UnsupportedUsage);
        }
        match entry.binding.access {
            Access::Local | Access::Global => Ok(AccessSpelling::Verbatim),
            Access::PropsAliased { .. } => Ok(AccessSpelling::Rewrite {
                prefix: bracket,
                replacement: entry.alias.as_deref(),
                suffix: "]",
                helper: None,
            }),
            Access::Context | Access::Data | Access::Props | Access::Options => {
                Ok(AccessSpelling::Rewrite {
                    prefix: dot,
                    replacement: None,
                    suffix: "",
                    helper: None,
                })
            }
            _ => Err(AccessError::UnsupportedUsage),
        }
    }
}

pub(super) struct RenderAccess<'r, 'a> {
    registry: &'r Vue2Access<'a>,
    dot: String,
    bracket: String,
}

impl AccessProvider for RenderAccess<'_, '_> {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        self.registry
            .spelling_in(occurrence, &self.dot, &self.bracket)
    }
}

impl AccessProvider for Vue2Access<'_> {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        self.spelling_in(occurrence, "_vm.", "_vm[")
    }
}
