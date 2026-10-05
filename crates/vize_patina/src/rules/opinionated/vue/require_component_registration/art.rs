//! Art registration from the genuine shared descriptor/script summary.

use crate::context::LintContext;
use vize_croquis::{Croquis, ScopeKind};
use vize_l0::{SourceRoot, String, ToCompactString};

pub(super) fn is_context(context: &LintContext<'_>) -> bool {
    context.filename.ends_with(".art.vue")
        && context.sfc_descriptor().is_some_and(|descriptor| {
            descriptor
                .custom_blocks
                .iter()
                .any(|block| block.block_type == "art")
        })
}

fn analysis<'a>(context: &'a LintContext<'_>) -> Option<&'a Croquis> {
    if !is_context(context) {
        return None;
    }
    context.analysis()
}

pub(super) fn setup_bindings(context: &LintContext<'_>) -> Option<Vec<String>> {
    Some(
        analysis(context)?
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::ScriptSetup)
            .flat_map(|scope| scope.bindings())
            .map(|(name, _)| name.to_compact_string())
            .collect(),
    )
}

pub(super) fn is_target(context: &LintContext<'_>, tag: &str) -> bool {
    if let Some(art) = analysis(context).and_then(|analysis| analysis.macros.define_art()) {
        return super::component_name_matches(tag, art.component_name.as_str());
    }
    let Some(descriptor) = context.sfc_descriptor().filter(|_| is_context(context)) else {
        return false;
    };
    let Some(fragment) = SourceRoot::new(descriptor.source.as_ref())
        .ok()
        .and_then(|root| root.whole_block().span_of(context.source))
    else {
        return false;
    };
    let Some(component) = descriptor
        .custom_blocks
        .iter()
        .find(|block| {
            block.block_type == "art"
                && block.loc.start <= fragment.start as usize
                && fragment.end as usize <= block.loc.end
        })
        .and_then(|block| block.attrs.get("component"))
    else {
        return false;
    };
    let without_query = component.split(['?', '#']).next().unwrap_or(component);
    let basename = without_query
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(without_query);
    let stem = basename.rsplit_once('.').map_or(basename, |(stem, _)| stem);
    let binding = vize_croquis::naming::to_pascal_case(stem);
    super::component_name_matches(tag, binding.as_str())
}
