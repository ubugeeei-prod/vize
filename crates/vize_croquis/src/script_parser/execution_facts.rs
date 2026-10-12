use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{walk_arrow_function_expression, walk_for_of_statement, walk_function};
use oxc_syntax::scope::ScopeFlags;

/// Detect script-setup top-level await from an already parsed AST.
///
/// A script setup block needs an async setup wrapper when `await` or
/// `for await` appears in the setup execution flow. Awaits inside nested
/// functions are not top-level and must not force the wrapper async.
pub(super) fn contains_top_level_await(program: &Program<'_>) -> bool {
    #[derive(Default)]
    struct TopLevelAwaitVisitor {
        function_depth: usize,
        found: bool,
    }

    impl<'a> Visit<'a> for TopLevelAwaitVisitor {
        fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
            self.function_depth += 1;
            walk_function(self, it, flags);
            self.function_depth = self.function_depth.saturating_sub(1);
        }

        fn visit_arrow_function_expression(
            &mut self,
            it: &oxc_ast::ast::ArrowFunctionExpression<'a>,
        ) {
            self.function_depth += 1;
            walk_arrow_function_expression(self, it);
            self.function_depth = self.function_depth.saturating_sub(1);
        }

        fn visit_await_expression(&mut self, _it: &oxc_ast::ast::AwaitExpression<'a>) {
            if self.function_depth == 0 {
                self.found = true;
            }
        }

        fn visit_for_of_statement(&mut self, it: &oxc_ast::ast::ForOfStatement<'a>) {
            if self.function_depth == 0 && it.r#await {
                self.found = true;
            }
            walk_for_of_statement(self, it);
        }
    }

    let mut visitor = TopLevelAwaitVisitor::default();
    visitor.visit_program(program);
    visitor.found
}

