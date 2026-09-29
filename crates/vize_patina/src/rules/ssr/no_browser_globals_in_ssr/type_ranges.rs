use oxc_allocator::Allocator;
use oxc_ast::ast::TSType;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::String;

pub(super) fn type_ranges(expr: &str) -> Vec<(usize, usize)> {
    const PREFIX: &str = "const __vize_ssr_expr = (";
    let mut source = String::with_capacity(PREFIX.len() + expr.len() + 2);
    source.push_str(PREFIX);
    source.push_str(expr);
    source.push_str(");");

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }

    struct TypeRanges(Vec<(usize, usize)>);
    impl<'a> Visit<'a> for TypeRanges {
        fn visit_ts_type(&mut self, ty: &TSType<'a>) {
            let span = ty.span();
            self.0.push((span.start as usize, span.end as usize));
        }
    }

    let mut ranges = TypeRanges(Vec::new());
    ranges.visit_program(&parsed.program);
    ranges
        .0
        .into_iter()
        .filter_map(|(start, end)| {
            let offset = PREFIX.len();
            (start >= offset).then_some((start - offset, end.saturating_sub(offset)))
        })
        .collect()
}
