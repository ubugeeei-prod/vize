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
            .retain(|name| !normalizes_to(name, &normalized));
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

/// `normalize(name) == normalized`, compared as it is produced.
///
/// Every element tag is checked against every unused setup binding, so
/// building each binding's normalized spelling again per tag was the cost.
fn normalizes_to(name: &str, normalized: &str) -> bool {
    if name.is_ascii() {
        return name
            .bytes()
            .filter(|byte| *byte != b'-')
            .map(|byte| byte.to_ascii_lowercase())
            .eq(normalized.bytes());
    }
    name.chars()
        .filter(|ch| *ch != '-')
        .flat_map(char::to_lowercase)
        .eq(normalized.chars())
}

#[cfg(test)]
mod tests {
    use super::{normalize, normalizes_to};

    #[test]
    fn normalizes_to_agrees_with_normalize() {
        let names = [
            "",
            "-",
            "Foo",
            "foo",
            "FooBar",
            "foo-bar",
            "foo--bar-",
            "-foo",
            "fooBar2",
            "v-focus",
            "vFocus",
            "Äpfel",
            "äpfel",
            "ÄPFEL-x",
            "İstanbul",
            "i̇stanbul",
            "Σίσυφος",
            "ΣΊΣΥΦΟΣ",
            "ß",
            "SS",
            "ǅungla",
            "ǆungla",
            "Ǆ-ungla",
        ];
        for name in names {
            for other in names {
                assert_eq!(
                    normalizes_to(name, &normalize(other)),
                    normalize(name) == normalize(other),
                    "{name:?} vs {other:?}"
                );
            }
        }
    }
}
