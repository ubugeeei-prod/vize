use super::Cx;

impl Cx<'_> {
    pub(crate) fn is_custom_element(&self, tag: &str) -> bool {
        self.custom_element_predicate
            .is_some_and(|predicate| predicate(tag))
            || self
                .custom_element_patterns
                .iter()
                .any(|pattern| crate::emit::tag_pattern_matches(pattern.as_str(), tag))
    }
}
