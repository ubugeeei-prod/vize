//! SSR root and fallthrough child-list emission.

use super::SsrCodegenContext;
use super::css_vars::RootCssVars;
use super::helpers::single_fallthrough_child_index;
use vize_atelier_core::TemplateChildNode;

impl<'a> SsrCodegenContext<'a> {
    /// Process root-level children and inherit `_attrs` into a single renderable
    /// root, matching Vue's fallthrough attrs behavior for SSR.
    pub(crate) fn process_root_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
    ) {
        self.process_children_with_fallthrough_attrs_and_css_vars(
            children,
            as_fragment,
            disable_nested_fragments,
            disable_comment,
            true,
            RootCssVars {
                enabled: self.options.ssr_css_vars.is_some(),
                template_wrapper: false,
            },
        );
    }

    pub(crate) fn process_children_with_fallthrough_attrs(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
    ) {
        self.process_children_with_fallthrough_attrs_and_css_vars(
            children,
            as_fragment,
            disable_nested_fragments,
            disable_comment,
            inherit_attrs,
            RootCssVars::default(),
        );
    }

    pub(crate) fn process_children_with_fallthrough_attrs_and_css_vars(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
        css_vars: RootCssVars,
    ) {
        if as_fragment {
            self.push_string_part_static("<!--[-->");
        }

        let fallthrough_child_index = if inherit_attrs && !as_fragment {
            single_fallthrough_child_index(children).unwrap_or(usize::MAX)
        } else {
            usize::MAX
        };

        for (index, child) in vize_atelier_core::walk_probe::ssr_children(children).enumerate() {
            self.process_child(
                child,
                disable_nested_fragments,
                disable_comment,
                fallthrough_child_index == index,
                css_vars,
            );
        }

        if as_fragment {
            self.push_string_part_static("<!--]-->");
        }
    }
}
