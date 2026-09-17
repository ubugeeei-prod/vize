use vize_croquis::patterns::PatternDiagnostic;
use vize_relief::{PropNode, RootNode, TemplateChildNode};

/// A disabled experimental directive must not be silently treated as an
/// ordinary custom directive by a type-checking entry point.
pub(crate) fn disabled_diagnostics(root: Option<&RootNode<'_>>) -> Vec<PatternDiagnostic> {
    let mut diagnostics = Vec::new();
    let Some(root) = root else { return diagnostics };
    let mut children: Vec<_> = root.children.iter().collect();
    while let Some(child) = children.pop() {
        if let TemplateChildNode::Element(element) = child {
            for prop in &element.props {
                if let PropNode::Directive(dir) = prop
                    && matches!(dir.name, "match" | "when")
                {
                    diagnostics.push(PatternDiagnostic {
                        message: "`v-match` / `v-when` require `experimentals.patternedTemplate`."
                            .into(),
                        start: dir.loc.span.start,
                        end: dir.loc.span.end,
                        warning: false,
                    });
                }
            }
            children.extend(element.children.iter().rev());
        }
    }
    diagnostics
}
