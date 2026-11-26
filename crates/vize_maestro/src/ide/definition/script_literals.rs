//! Whether a script offset sits in text that names no binding.
//!
//! The spelling-based definition fallback finds a binding by the identifier
//! under the cursor. Inside a string literal, a template literal's text or a
//! comment that "identifier" is data: `useTemplateRef('field')` names a
//! template ref, not a same-spelled `const field`.

use oxc_allocator::Allocator;
use oxc_ast::ast::{StringLiteral, TemplateElement};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Whether `offset` (into `script`) falls inside a string literal, the text of
/// a template literal, or a comment. Unparsable scripts answer `false`, so the
/// fallback keeps working while the user types.
pub(super) fn offset_is_in_literal_text(script: &str, offset: usize) -> bool {
    let Ok(offset) = u32::try_from(offset) else {
        return false;
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return false;
    }
    let mut finder = LiteralFinder {
        offset,
        found: parsed
            .program
            .comments
            .iter()
            .any(|comment| comment.span.start < offset && offset < comment.span.end),
    };
    finder.visit_program(&parsed.program);
    finder.found
}

struct LiteralFinder {
    offset: u32,
    found: bool,
}

impl<'a> Visit<'a> for LiteralFinder {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        // Strictly inside the quotes.
        self.found |= literal.span.start < self.offset && self.offset < literal.span.end;
    }

    fn visit_template_element(&mut self, element: &TemplateElement<'a>) {
        // A quasi's span covers only its text, so its edges are text too.
        self.found |= element.span.start <= self.offset && self.offset <= element.span.end;
        walk::walk_template_element(self, element);
    }
}

#[cfg(test)]
mod tests {
    use super::offset_is_in_literal_text;

    #[test]
    fn literal_text_and_comments_name_no_binding() {
        let script = "const field = 1\nconst el = useTemplateRef('field')\n// field\nconst t = `field ${field}`\n";
        let at = |needle: &str, nth: usize| script.match_indices(needle).nth(nth).unwrap().0 + 1;
        // `'field'`, the comment, and the template text are data.
        assert_eq!(
            [at("field'", 0), at("field\n", 0), at("field $", 0)]
                .map(|offset| { offset_is_in_literal_text(script, offset) }),
            [true, true, true]
        );
        // The declaration and the substitution are bindings.
        assert_eq!(
            [at("field =", 0), at("field}", 0)]
                .map(|offset| offset_is_in_literal_text(script, offset)),
            [false, false]
        );
    }
}
