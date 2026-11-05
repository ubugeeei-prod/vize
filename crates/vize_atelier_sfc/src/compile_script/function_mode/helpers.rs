//! Utility functions for function-mode script compilation.
//!
//! Contains helpers for JavaScript reserved word checking,
//! runtime identifier collection, and top-level await detection.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Class, Function, IdentifierReference, PropertyDefinition, TSClassImplements, TSEnumDeclaration,
    TSGlobalDeclaration, TSIndexSignature, TSInterfaceDeclaration, TSModuleDeclaration,
    TSThisParameter, TSType, TSTypeAliasDeclaration, TSTypeAnnotation, TSTypeParameterDeclaration,
    TSTypeParameterInstantiation, VariableDeclaration,
};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::{
    walk_arrow_function_expression, walk_class, walk_for_of_statement, walk_function,
    walk_property_definition, walk_ts_enum_declaration, walk_ts_module_declaration,
    walk_variable_declaration,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{FxHashSet, String, ToCompactString};

/// Check if an identifier is a JavaScript reserved word (avoid shorthand).
pub(crate) fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}

/// Parse a transformed setup body; TS parses a superset of either lang.
pub(crate) fn parse_setup_body<'a>(allocator: &'a Allocator, code: &'a str) -> ParserReturn<'a> {
    crate::script::parse_script_text(allocator, code, SourceType::ts())
}

/// [`collect_runtime_identifier_references`] for a setup body not parsed yet.
pub(crate) fn runtime_identifier_references(code: &str) -> FxHashSet<String> {
    let allocator = Allocator::default();
    collect_runtime_identifier_references(&parse_setup_body(&allocator, code))
}

/// Identifiers the setup code references at runtime.
///
/// Reads the setup program as parsed, skipping TypeScript-only syntax: that is
/// exactly what the emitted JavaScript drops, so no strip, reprint or re-parse
/// is needed. A program that did not parse cleanly contributes nothing.
pub(crate) fn collect_runtime_identifier_references(
    parsed: &ParserReturn<'_>,
) -> FxHashSet<String> {
    // The emitted module keeps JSX; its JavaScript re-parse never accepted it.
    if parsed.panicked || !parsed.diagnostics.is_empty() || parsed.program.source_type.is_jsx() {
        return FxHashSet::default();
    }

    #[derive(Default)]
    struct RuntimeIdentifierVisitor {
        identifiers: FxHashSet<String>,
    }

    impl<'a> Visit<'a> for RuntimeIdentifierVisitor {
        fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
            self.identifiers
                .insert(ident.name.as_str().to_compact_string());
        }
        fn visit_ts_type(&mut self, _: &TSType<'a>) {}
        fn visit_ts_type_annotation(&mut self, _: &TSTypeAnnotation<'a>) {}
        fn visit_ts_type_parameter_declaration(&mut self, _: &TSTypeParameterDeclaration<'a>) {}
        fn visit_ts_type_parameter_instantiation(&mut self, _: &TSTypeParameterInstantiation<'a>) {}
        fn visit_ts_this_parameter(&mut self, _: &TSThisParameter<'a>) {}
        fn visit_ts_class_implements(&mut self, _: &TSClassImplements<'a>) {}
        fn visit_ts_index_signature(&mut self, _: &TSIndexSignature<'a>) {}
        fn visit_ts_interface_declaration(&mut self, _: &TSInterfaceDeclaration<'a>) {}
        fn visit_ts_type_alias_declaration(&mut self, _: &TSTypeAliasDeclaration<'a>) {}
        fn visit_ts_global_declaration(&mut self, _: &TSGlobalDeclaration<'a>) {}
        fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
            if !it.declare {
                walk_variable_declaration(self, it);
            }
        }
        fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
            if !it.declare {
                walk_function(self, it, flags);
            }
        }
        fn visit_class(&mut self, it: &Class<'a>) {
            if !it.declare {
                walk_class(self, it);
            }
        }
        fn visit_property_definition(&mut self, it: &PropertyDefinition<'a>) {
            if !it.declare {
                walk_property_definition(self, it);
            }
        }
        fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
            if !it.declare {
                walk_ts_enum_declaration(self, it);
            }
        }
        fn visit_ts_module_declaration(&mut self, it: &TSModuleDeclaration<'a>) {
            if !it.declare {
                walk_ts_module_declaration(self, it);
            }
        }
    }

    let mut visitor = RuntimeIdentifierVisitor::default();
    visitor.visit_program(&parsed.program);
    visitor.identifiers
}

/// Detect top-level await in setup code (ignores awaits inside nested functions).
pub fn contains_top_level_await(code: &str, is_ts: bool) -> bool {
    if !code.contains("await") {
        return false;
    }

    let allocator = Allocator::default();
    let source_type = if is_ts {
        SourceType::ts()
    } else {
        SourceType::default()
    };

    let mut wrapped = String::with_capacity(code.len() + 28);
    wrapped.push_str("async function __temp__() {\n");
    wrapped.push_str(code);
    wrapped.push_str("\n}");
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse();

    if !parse_result.diagnostics.is_empty() {
        return false;
    }

    #[derive(Default)]
    struct TopLevelAwaitVisitor {
        depth: usize,
        found: bool,
    }

    impl<'a> Visit<'a> for TopLevelAwaitVisitor {
        fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
            self.depth += 1;
            walk_function(self, it, flags);
            self.depth = self.depth.saturating_sub(1);
        }

        fn visit_arrow_function_expression(
            &mut self,
            it: &oxc_ast::ast::ArrowFunctionExpression<'a>,
        ) {
            self.depth += 1;
            walk_arrow_function_expression(self, it);
            self.depth = self.depth.saturating_sub(1);
        }

        fn visit_await_expression(&mut self, _it: &oxc_ast::ast::AwaitExpression<'a>) {
            if self.depth == 1 {
                self.found = true;
            }
        }

        fn visit_for_of_statement(&mut self, it: &oxc_ast::ast::ForOfStatement<'a>) {
            if self.depth == 1 && it.r#await {
                self.found = true;
            }
            walk_for_of_statement(self, it);
        }
    }

    let mut visitor = TopLevelAwaitVisitor::default();
    visitor.visit_program(&parse_result.program);
    visitor.found
}
