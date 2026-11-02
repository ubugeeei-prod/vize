//! Readers and test writers for component usages.
//!
//! Field storage stays inside this crate. Everyone else reads through these
//! functions, or records a usage through these methods. The readers answer
//! from storage exactly as the `ComponentUsages` group's sites would, without
//! resolving every tag's identity per call (codegen asks once per scope).

use vize_carton::CompactString;

use crate::Croquis;
use crate::croquis::ComponentUsage;

/// Template component usages, in source order, borrowed.
pub fn component_usage_slice(croquis: &Croquis) -> &[ComponentUsage] {
    &croquis.component_usages
}

/// Template component usages, in source order.
pub fn component_usage_list(croquis: &Croquis) -> Vec<ComponentUsage> {
    component_usage_slice(croquis).to_vec()
}

/// Names from the used-component set, in that set's iteration order,
/// then usage names the set does not contain.
pub fn used_component_name_list(croquis: &Croquis) -> Vec<CompactString> {
    let mut names: Vec<CompactString> = croquis.used_components.iter().cloned().collect();
    let mut extra: Vec<CompactString> = Vec::new();
    for usage in &croquis.component_usages {
        if !croquis.used_components.contains(usage.name.as_str()) && !extra.contains(&usage.name) {
            extra.push(usage.name.clone());
        }
    }
    names.append(&mut extra);
    names
}

pub fn used_components_empty(croquis: &Croquis) -> bool {
    croquis.used_components.is_empty() && croquis.component_usages.is_empty()
}

pub fn used_component_contains(croquis: &Croquis, name: &str) -> bool {
    croquis.used_components.contains(name)
        || croquis
            .component_usages
            .iter()
            .any(|usage| usage.name.as_str() == name)
}

impl Croquis {
    pub fn note_used_component(&mut self, name: impl AsRef<str>) {
        self.used_components
            .insert(CompactString::new(name.as_ref()));
    }

    pub fn note_component_usage(&mut self, usage: ComponentUsage) {
        self.component_usages.push(usage);
    }
}

#[cfg(test)]
#[path = "access_tests.rs"]
mod tests;
