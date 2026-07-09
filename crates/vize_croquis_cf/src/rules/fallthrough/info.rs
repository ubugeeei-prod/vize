use super::{FallthroughInfo, is_standard_html_attr};

impl FallthroughInfo {
    /// Count passed attributes that are not declared props.
    pub fn fallthrough_attr_count(&self) -> usize {
        self.passed_attrs
            .iter()
            .filter(|attr| !self.declared_props.contains(*attr))
            .count()
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
        self.passed_attrs
            .iter()
            .filter(|attr| !self.declared_props.contains(*attr))
            .filter(|attr| is_standard_html_attr(attr))
            .count()
    }

    /// Count undeclared, unconsumed attrs that are not known safe HTML/listener attrs.
    pub fn risky_unconsumed_fallthrough_attr_count(&self) -> usize {
        if self.consumes_fallthrough_attrs() {
            return 0;
        }

        self.passed_attrs
            .iter()
            .filter(|attr| !self.declared_props.contains(*attr))
            .filter(|attr| !is_standard_html_attr(attr))
            .count()
    }
}
