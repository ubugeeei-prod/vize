//! Custom directive arguments are binding values, not JavaScript property keys.

use super::DirectiveValueBinding;
use vize_carton::{FxHashMap, FxHashSet};
use vize_relief::{DirectiveNode, ExpressionNode};

#[derive(Default)]
pub(crate) struct DirectiveValueBindings {
    pub(crate) values: FxHashMap<(u32, u32), DirectiveValueBinding>,
    pub(crate) dynamic_arguments: FxHashSet<(u32, u32)>,
    pub(super) enabled: bool,
}

pub(super) fn collect(directive: &DirectiveNode<'_>, bindings: &mut DirectiveValueBindings) {
    if vize_carton::is_builtin_directive(directive.name) {
        return;
    }
    if let Some(argument) = directive.arg.as_ref().filter(|argument| match argument {
        ExpressionNode::Simple(argument) => !argument.is_static,
        ExpressionNode::Compound(_) => true,
    }) {
        let span = argument.loc().span;
        bindings.dynamic_arguments.insert((span.start, span.end));
    }
}

#[cfg(test)]
mod tests {
    use super::{DirectiveValueBindings, collect};
    use vize_relief::{PropNode, TemplateChildNode};

    #[test]
    fn parser_identity_and_exact_spans_separate_custom_from_builtin_arguments() {
        let source = concat!(
            "<div v-clickoutside:[contentRef]=\"handler\"\n",
            " v-click-outside:[popperRef].once = \"handler\"",
            " v-Custom:[caseArg] v-click-outside:static=\"handler\"",
            " v-bind:[bindArg]=\"value\" v-model:[modelArg]=\"value\"",
            " v-slot:[slotArg] v-on:[eventArg]=\"handler\"></div>",
        );
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, source);
        assert_eq!(format!("{errors:?}"), "[]");
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("the original parsed element is required");
        };
        let mut bindings = DirectiveValueBindings::default();
        for property in &element.props {
            if let PropNode::Directive(directive) = property {
                collect(directive, &mut bindings);
            }
        }
        let mut actual: Vec<_> = bindings.dynamic_arguments.iter().copied().collect();
        actual.sort();
        let expected: Vec<_> = ["contentRef", "popperRef", "caseArg"]
            .into_iter()
            .map(|name| {
                let start = source.find(name).unwrap() as u32;
                (start, start + name.len() as u32)
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(bindings.enabled, false);
    }
}
