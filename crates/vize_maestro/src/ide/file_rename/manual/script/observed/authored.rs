//! Collect file-absolute occurrences directly into the editor's response.
use super::{AuthoredProgram, ScriptModuleSpecifier, ScriptQueryProducts, Span};
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportExpression, Program, StringLiteral, TSImportType,
};
use oxc_ast_visit::{Visit, walk};

struct AuthoredSpecifiers<'a> {
    out: &'a mut ScriptQueryProducts,
    start: u32,
}

impl AuthoredSpecifiers<'_> {
    fn push(&mut self, literal: &StringLiteral<'_>) {
        self.out.module_specifiers.push(ScriptModuleSpecifier {
            span: Span::new(
                self.start + literal.span.start + 1,
                self.start + literal.span.end - 1,
            ),
            specifier: literal.value.as_str().into(),
        });
    }
}

impl<'a> Visit<'a> for AuthoredSpecifiers<'_> {
    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        if let Expression::StringLiteral(literal) = &expression.source {
            self.push(literal);
        }
        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(identifier) = &call.callee
            && identifier.name.as_str() == "require"
            && let Some(Argument::StringLiteral(literal)) = call.arguments.first()
        {
            self.push(literal);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.push(&import.source);
        walk::walk_ts_import_type(self, import);
    }
}

impl ScriptQueryProducts {
    pub(crate) fn read_authored_specifiers(&mut self, syntax: &AuthoredProgram<'_>) {
        self.read_retained_specifiers(
            &syntax.parsed().program,
            syntax.block().start(),
            !Self::static_specifiers_only(syntax),
        );
    }

    pub(crate) fn read_retained_specifiers(
        &mut self,
        program: &Program<'_>,
        base: u32,
        visit_nested: bool,
    ) {
        for statement in &program.body {
            self.read_static_specifier(base, statement);
        }
        if visit_nested {
            AuthoredSpecifiers {
                out: self,
                start: base,
            }
            .visit_program(program);
        }
    }
}
