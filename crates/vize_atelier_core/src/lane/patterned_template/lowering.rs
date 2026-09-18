//! Lazy RFC 823 selector lowering. See LICENSE for the Vue reference attribution.

mod aggregate;

use vize_armature::patterns::{MatchArm, MatchPattern, PatternKind};
use vize_s0::{String, cstr};

struct Selector<'a> {
    prefix: &'a str,
    next: usize,
    tests: std::vec::Vec<String>,
    copies: std::vec::Vec<String>,
    bindings: std::vec::Vec<String>,
}

impl Selector<'_> {
    fn temp(&mut self) -> String {
        let name = cstr!("{}_{}", self.prefix, self.next);
        self.next += 1;
        name
    }

    fn emit(&mut self, pattern: &MatchPattern, value: &str) {
        match &pattern.kind {
            PatternKind::Wildcard => {}
            PatternKind::Binding(binding) => {
                self.bindings
                    .push(cstr!("const {} = {value};", binding.name));
            }
            PatternKind::As { pattern, binding } => {
                self.emit(pattern, value);
                self.bindings
                    .push(cstr!("const {} = {value};", binding.name));
            }
            PatternKind::Literal(literal) => {
                self.tests
                    .push(cstr!("if (!({value} === {})) return null;", literal.text));
            }
            PatternKind::Value(expression) => {
                let expected = self.temp();
                self.tests.push(cstr!(
                    "const {expected} = {}; if (!({value} === {expected} || ({value} !== {value} && {expected} !== {expected}))) return null;",
                    expression.text
                ));
            }
            PatternKind::Or(patterns) => {
                let mut alternatives = std::vec::Vec::new();
                for pattern in patterns {
                    let start = self.tests.len();
                    self.emit(pattern, value);
                    let tests = self
                        .tests
                        .drain(start..)
                        .collect::<std::vec::Vec<_>>()
                        .join(" ");
                    alternatives.push(cstr!("(() => {{ {tests} return true; }})()"));
                }
                self.tests
                    .push(cstr!("if (!({})) return null;", alternatives.join(" || ")));
            }
            PatternKind::Object { properties, rest } => {
                self.object(properties, rest.as_ref(), value)
            }
            PatternKind::Array { elements, rest } => self.array(elements, rest.as_ref(), value),
        }
    }
}

/// Selection returns the arm index followed by its captured binding values.
/// Guard and render scopes consume the same values; neither rereads the subject.
pub(super) fn generate_selector(arms: &[MatchArm], subject: &str, prefix: &str) -> String {
    let mut emitter = Selector {
        prefix,
        next: 0,
        tests: std::vec::Vec::new(),
        copies: std::vec::Vec::new(),
        bindings: std::vec::Vec::new(),
    };
    let root = emitter.temp();
    let mut code = cstr!("(({root}) => {{ ");
    for (index, arm) in arms.iter().enumerate() {
        emitter.tests.clear();
        emitter.copies.clear();
        emitter.bindings.clear();
        emitter.emit(&arm.pattern, &root);
        let result = emitter.temp();
        code.push_str(&cstr!("{{ const {result} = (() => {{ "));
        code.push_str(&emitter.tests.join(" "));
        code.push(' ');
        code.push_str(&emitter.copies.join(" "));
        // Authored bindings must not put enclosing value-pattern reads in a TDZ.
        code.push_str(" { ");
        code.push_str(&emitter.bindings.join(" "));
        if let Some(guard) = &arm.guard {
            code.push_str(&cstr!(" if (!({})) return null;", guard.text));
        }
        code.push_str(&cstr!(" return [{index}"));
        for binding in &arm.bindings {
            code.push_str(&cstr!(", {}", binding.name));
        }
        code.push_str(&cstr!(
            "]; }} }})(); if ({result} !== null) return {result}; }} "
        ));
    }
    // The authored subject may end in a line comment.
    code.push_str(&cstr!("return [-1]; }})(({subject}\n))"));
    code
}
