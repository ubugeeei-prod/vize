//! Template writes only suppress advice; token ambiguity is conservative.
use super::super::template_surface::{
    DecodedSource, DirectiveArgument, directive, element_verbatim, static_attribute_value,
};
use super::{SfcScriptContext, analysis::Inventory};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{CompactString, FxHashSet};
use vize_l1::markup::{SurfaceChild, SurfaceTree};

pub(super) fn template_writes(sfc: SfcScriptContext<'_>) -> FxHashSet<CompactString> {
    let mut writes = FxHashSet::default();
    if let Some(root) = sfc.template_surface {
        collect_template_writes(root, &root.children, false, &mut writes);
    }
    writes
}

fn collect_template_writes(
    root: &SurfaceTree<'_>,
    children: &[SurfaceChild<'_>],
    inherited_verbatim: bool,
    writes: &mut FxHashSet<CompactString>,
) {
    for child in children {
        if let SurfaceChild::Interpolation(interpolation) = child
            && !inherited_verbatim
            && let Some(source) = DecodedSource::interpolation(root.source, &interpolation.content)
        {
            collect_expression_writes(source.text(), writes);
        }
        if let SurfaceChild::Element(element) = child {
            let verbatim = element_verbatim(element, inherited_verbatim);
            for attribute in &element.open.attrs {
                if let Some(directive) = directive(root, attribute, verbatim) {
                    let Some(value) = directive.value else {
                        continue;
                    };
                    let is_bound_ref = directive.name == "bind"
                        && directive.argument == Some(DirectiveArgument::Static("ref"));
                    if directive.name == "model" || is_bound_ref {
                        collect_tokens(value.text(), writes);
                    } else {
                        collect_expression_writes(value.text(), writes);
                    }
                } else if attribute.name.text == "ref"
                    && let Some(value) = static_attribute_value(root, attribute)
                {
                    writes.insert(CompactString::new(value.text()));
                }
            }
            collect_template_writes(root, &element.children, verbatim, writes);
        }
    }
}

fn collect_expression_writes(text: &str, writes: &mut FxHashSet<CompactString>) {
    if !(text.contains('=')
        || text.contains("++")
        || text.contains("--")
        || text.contains("delete ")
        || text.contains('('))
    {
        return;
    }
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(
        &allocator,
        text,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        // Unknown expressions cannot establish absence of writes.
        collect_tokens(text, writes);
    } else {
        writes.extend(Inventory::collect(&parsed.program).written_names());
    }
}

fn collect_tokens(text: &str, writes: &mut FxHashSet<CompactString>) {
    // Over-collecting tokens can only suppress advice, never create findings.
    for token in text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
        if !token.is_empty() {
            writes.insert(CompactString::new(token));
        }
    }
}
