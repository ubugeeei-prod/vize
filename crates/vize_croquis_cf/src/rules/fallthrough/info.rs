use super::is_standard_html_attr;
use crate::registry::FileId;
use vize_carton::{CompactString, FxHashSet};

/// Information about fallthrough attributes for a component.
#[derive(Debug, Clone)]
pub struct FallthroughInfo {
    /// File ID of the component.
    pub file_id: FileId,
    /// Whether `inheritAttrs: false` is set.
    pub inherit_attrs_disabled: bool,
    /// Whether $attrs is used in template.
    pub uses_attrs: bool,
    /// Whether $attrs is explicitly bound (v-bind="$attrs").
    pub binds_attrs: bool,
    /// Number of root elements in template.
    pub root_element_count: usize,
    /// Attributes passed by parent components.
    pub passed_attrs: FxHashSet<CompactString>,
    /// Passed attributes that are neither declared props nor declared event listeners.
    pub fallthrough_attrs: FxHashSet<CompactString>,
    /// Fallthrough names represented by runtime directive argument expressions.
    pub dynamic_name_fallthrough_attrs: FxHashSet<CompactString>,
    /// Props declared by this component.
    pub declared_props: FxHashSet<CompactString>,
    /// Events declared by this component.
    pub declared_events: FxHashSet<CompactString>,
    /// Template content start offset (relative to template block).
    pub template_start: u32,
    /// Template content end offset (relative to template block).
    pub template_end: u32,
}

impl FallthroughInfo {
    /// Check if fallthrough may cause issues.
    pub fn has_potential_issues(&self) -> bool {
        // Multiple roots without explicit $attrs
        if self.root_element_count > 1 && !self.binds_attrs {
            return true;
        }

        // inheritAttrs: false but $attrs not used
        if self.inherit_attrs_disabled && !self.uses_attrs && !self.binds_attrs {
            return true;
        }

        // Attributes passed that are neither declared props nor declared listeners
        if !self.fallthrough_attrs.is_empty() && !self.uses_attrs && self.root_element_count > 1 {
            return true;
        }

        false
    }
}

impl FallthroughInfo {
    /// Count passed attributes that are neither declared props nor declared listeners.
    pub fn fallthrough_attr_count(&self) -> usize {
        self.fallthrough_attrs.len()
    }

    /// Single-root components inherit fallthrough attrs automatically unless disabled.
    pub fn automatically_inherits_attrs(&self) -> bool {
        self.root_element_count == 1 && !self.inherit_attrs_disabled
    }

    /// Whether this component has a path that consumes fallthrough attrs.
    pub fn consumes_fallthrough_attrs(&self) -> bool {
        self.automatically_inherits_attrs() || self.uses_attrs || self.binds_attrs
    }

    /// Count undeclared passed attrs that have a consumption path.
    pub fn consumed_fallthrough_attr_count(&self) -> usize {
        if self.consumes_fallthrough_attrs() {
            self.fallthrough_attr_count()
        } else {
            0
        }
    }

    /// Count undeclared passed attrs that are not consumed.
    pub fn unconsumed_fallthrough_attr_count(&self) -> usize {
        if self.consumes_fallthrough_attrs() {
            0
        } else {
            self.fallthrough_attr_count()
        }
    }

    /// Count undeclared passed attrs that Vue commonly forwards safely.
    pub fn safe_standard_fallthrough_attr_count(&self) -> usize {
        self.fallthrough_attrs
            .iter()
            .filter(|attr| {
                !self.dynamic_name_fallthrough_attrs.contains(*attr) && is_standard_html_attr(attr)
            })
            .count()
    }

    /// Count undeclared, unconsumed attrs that are not known safe HTML/listener attrs.
    pub fn risky_unconsumed_fallthrough_attr_count(&self) -> usize {
        if self.consumes_fallthrough_attrs() {
            return 0;
        }

        self.fallthrough_attrs
            .iter()
            .filter(|attr| {
                self.dynamic_name_fallthrough_attrs.contains(*attr) || !is_standard_html_attr(attr)
            })
            .count()
    }
}
