//! Remove proven template reads from the script's unused candidate relation.

use vize_carton::CompactString;
use vize_relief::{ExpressionNode, PropNode};

use super::Drawer;

impl Drawer {
    pub(super) fn read_setup_ref_prop(&mut self, prop: &PropNode<'_>) {
        if self.croquis.unused_bindings.is_empty() {
            return;
        }
        let name = match prop {
            PropNode::Attribute(attr) if attr.name == "ref" => attr
                .value
                .as_ref()
                .map(|value| CompactString::new(value.content)),
            PropNode::Directive(dir)
                if dir.name == "bind"
                    && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "ref") =>
            {
                match &dir.exp {
                    Some(ExpressionNode::Simple(exp)) => exp
                        .js_ast
                        .and_then(|js| js.as_expression())
                        .and_then(|js| match js.ast {
                            oxc_ast::ast::Expression::StringLiteral(value) => {
                                Some(CompactString::new(value.value.as_str()))
                            }
                            _ => None,
                        }),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(name) = name {
            self.croquis
                .unused_bindings
                .retain(|candidate| candidate.as_str() != name.as_str());
        }
    }

    pub(super) fn read_setup_tag(&mut self, tag: &str) {
        if self.croquis.unused_bindings.is_empty() {
            return;
        }
        // Vue resolves Foo, foo-bar and Foo.Bar through the setup scope.
        let root = tag.split('.').next().unwrap_or(tag);
        let normalized = normalize(root);
        self.croquis
            .unused_bindings
            .retain(|name| normalize(name) != normalized);
    }

    pub(super) fn read_setup_directive(&mut self, directive: &str) {
        if !self.croquis.unused_bindings.is_empty() {
            let spelling = vize_carton::cstr!("v-{directive}");
            self.read_setup_tag(&spelling);
        }
    }
}

/// Vue's component/directive resolution ignores hyphens and case.
pub(crate) fn normalize(name: &str) -> CompactString {
    name.chars()
        .filter(|ch| *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}
