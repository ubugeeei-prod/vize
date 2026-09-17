use super::{SfcDescriptor, SfcTemplateBlock};
use vize_carton::String;

fn is_match_attribute(name: &str) -> bool {
    name == "v-match" || name.starts_with("v-match:") || name.starts_with("v-match.")
}

impl SfcTemplateBlock<'_> {
    /// Whether the SFC header declares patterned-template semantics.
    pub fn has_root_match(&self) -> bool {
        self.attrs.keys().any(|name| is_match_attribute(name))
    }
}

impl SfcDescriptor<'_> {
    /// Hash template content and any root match directive affecting its behavior.
    pub fn template_hash(&self) -> Option<String> {
        let template = self.template.as_ref()?;
        if !template.has_root_match() {
            return Some(vize_carton::hash::content_hash(&template.content));
        }
        let mut directives: Vec<_> = template
            .attrs
            .iter()
            .filter(|(name, _)| is_match_attribute(name))
            .collect();
        directives.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let mut source = String::from(template.content.as_ref());
        for (name, value) in directives {
            source.push('\0');
            source.push_str(name);
            source.push('\0');
            source.push_str(value);
        }
        Some(vize_carton::hash::content_hash(&source))
    }
}
