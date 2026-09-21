//! Lazy RFC 823 selector lowering. See LICENSE for the Vue reference attribution.

mod aggregate;

use vize_armature::patterns::{MatchArm, MatchPattern, PatternKind};
use vize_s0::{String, cstr};

struct Selector<'a> {
    prefix: &'a str,
    next: usize,
    /// The statement a failed test runs: leave the arm, or fail the alternative.
    fail: String,
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

    fn reject_unless(&mut self, condition: &str) {
        self.reject_if(&cstr!("!({condition})"));
    }

    fn reject_if(&mut self, condition: &str) {
        let fail = &self.fail;
        self.tests.push(cstr!("if ({condition}) {fail}"));
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
                self.reject_if(&cstr!("{value} !== {}", literal.text));
            }
            PatternKind::Value(expression) => {
                let expected = self.temp();
                self.tests
                    .push(cstr!("const {expected} = {};", expression.text));
                self.reject_unless(&cstr!(
                    "{value} === {expected} || ({value} !== {value} && {expected} !== {expected})"
                ));
            }
            PatternKind::Or(patterns) => {
                let alternatives = patterns
                    .iter()
                    .map(|pattern| self.alternative(pattern, value))
                    .collect::<std::vec::Vec<_>>()
                    .join(" || ");
                self.reject_unless(&alternatives);
            }
            PatternKind::Object { properties, rest } => {
                self.object(properties, rest.as_ref(), value)
            }
            PatternKind::Array { elements, rest } => self.array(elements, rest.as_ref(), value),
        }
    }

    /// One `|` alternative as a boolean expression. A literal is a comparison;
    /// anything structural runs its tests in a function of its own.
    fn alternative(&mut self, pattern: &MatchPattern, value: &str) -> String {
        if let PatternKind::Literal(literal) = &pattern.kind {
            return cstr!("{value} === {}", literal.text);
        }
        let start = self.tests.len();
        let arm_fail = std::mem::replace(&mut self.fail, String::from("return false;"));
        self.emit(pattern, value);
        self.fail = arm_fail;
        let tests = self
            .tests
            .drain(start..)
            .collect::<std::vec::Vec<_>>()
            .join(" ");
        cstr!("(() => {{ {tests} return true; }})()")
    }
}

/// Selection returns the arm index followed by its captured binding values.
/// Guard and render scopes consume the same values; neither rereads the subject.
///
/// Each arm is a labelled block that a failed test leaves, so arms are tried in
/// source order without a closure per arm. An arm nothing can reject ends the
/// selector: later arms are unreachable and the no-match result is never needed.
pub(super) fn generate_selector(arms: &[MatchArm], subject: &str, prefix: &str) -> String {
    let mut emitter = Selector {
        prefix,
        next: 0,
        fail: String::default(),
        tests: std::vec::Vec::new(),
        copies: std::vec::Vec::new(),
        bindings: std::vec::Vec::new(),
    };
    let root = emitter.temp();
    let mut code = cstr!("(({root}) => {{ ");
    let mut exhaustive = false;
    for (index, arm) in arms.iter().enumerate() {
        emitter.tests.clear();
        emitter.copies.clear();
        emitter.bindings.clear();
        emitter.fail = cstr!("break a{index};");
        emitter.emit(&arm.pattern, &root);

        let mut result = cstr!("return [{index}");
        for binding in &arm.bindings {
            result.push_str(&cstr!(", {}", binding.name));
        }
        result.push_str("];");
        let refutable = !emitter.tests.is_empty() || arm.guard.is_some();
        if refutable {
            code.push_str(&cstr!("a{index}: {{ "));
        }
        for statement in emitter.tests.iter().chain(&emitter.copies) {
            code.push_str(statement);
            code.push(' ');
        }
        // Authored bindings must not put enclosing value-pattern reads in a TDZ,
        // so they are declared in a block of their own, after every test.
        let scoped = !emitter.bindings.is_empty();
        if scoped {
            code.push_str("{ ");
            for binding in &emitter.bindings {
                code.push_str(binding);
                code.push(' ');
            }
        }
        if let Some(guard) = &arm.guard {
            code.push_str(&cstr!("if (!({})) break a{index}; ", guard.text));
        }
        code.push_str(&result);
        code.push(' ');
        if scoped {
            code.push_str("} ");
        }
        if refutable {
            code.push_str("} ");
        } else {
            exhaustive = true;
            break;
        }
    }
    if !exhaustive {
        code.push_str("return [-1]; ");
    }
    // The subject is one argument: a comma expression or a spread would be read
    // as an argument list, so only those pay for a second pair of parentheses.
    // A trailing line comment must not swallow the closing parenthesis.
    let terminator = if subject.contains("//") { "\n" } else { "" };
    if subject.contains(',') || subject.trim_start().starts_with("...") {
        code.push_str(&cstr!("}})(({subject}{terminator}))"));
    } else {
        code.push_str(&cstr!("}})({subject}{terminator})"));
    }
    code
}
