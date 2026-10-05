use super::super::{DescriptorIssueCode as Code, policy::Policy};
use super::Vue1DescriptorObservation;
use crate::container::{Vue, vue::DescriptorOptions};
use crate::dialect::vue1::surface::parse_component_block;
use vize_l0::{Allocator, Span};

impl Vue {
    /// Explicit V1/Vue/default-delimiter, scriptless/styleless, zero-attribute selection.
    /// The original splitter runs once. Unsupported envelopes never start the
    /// Component parser; supported envelopes retain its complete normal owner.
    /// ```
    /// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
    /// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}};
    /// let arena = Allocator::default();
    /// let owner = Vue.observe_vue1_descriptor(&arena,
    ///     "<template>{{ value }}</template>", DescriptorOptions {
    ///         version: VueVersion::V1, dialect: VueDialect::Vue,
    ///         template: SurfaceParseOptions::default(),
    ///     });
    /// let selected = owner.selected().unwrap();
    /// let component = selected.component();
    /// let text = component.text_for(component.children().next().unwrap()).unwrap();
    /// assert_eq!(text.binding().syntax().unwrap().source().text(), "value");
    /// ```
    pub fn observe_vue1_descriptor<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        options: DescriptorOptions,
    ) -> Vue1DescriptorObservation<'a> {
        let mut state = Policy::new_vue1(allocator, source, options);
        let container = crate::container::vue::split_with(
            allocator,
            source,
            |index, block, uncertain, self_closing, closing| {
                let code = if block.name.eq_ignore_ascii_case("script") {
                    Some(Code::UnsupportedScript)
                } else if block.name.eq_ignore_ascii_case("style") {
                    Some(Code::UnsupportedStyle)
                } else if !block.name.eq_ignore_ascii_case("template") {
                    Some(Code::UnsupportedBlock)
                } else if !block.attrs.is_empty() {
                    // The emitted original header has already been scanned.
                    // This first V1 envelope refuses attributes in O(1), with
                    // no second header classification or value observation.
                    Some(Code::UnsupportedAttribute)
                } else {
                    None
                };
                if let Some(code) = code {
                    state.issue(code, Some(index), block.open_tag);
                } else {
                    state.record(index, block, uncertain, self_closing, closing);
                }
            },
        );
        state.finish();
        let mut owner = Vue1DescriptorObservation {
            container,
            root: state.root,
            options,
            issues: state.issues,
            selection: state.template,
            component: None,
        };
        if owner.issues.is_empty() && owner.container.errors.is_empty() {
            if let Some((selection, _)) = super::view::frame(&owner) {
                owner.component = Some(parse_component_block(allocator, selection.selection.block));
                #[cfg(test)]
                super::hooks::after_park(&owner);
            } else {
                owner.issues.push(super::super::DescriptorIssue {
                    code: Code::InvalidSourceFrame,
                    container_index: owner.selection.as_ref().map(|s| s.selection.index),
                    span: Span::new(0, 0),
                });
            }
        }
        owner
    }
}
