//! Binding metadata conversion and registration for SFC compilation.
//!
//! Handles converting between Croquis and legacy binding formats,
//! and registering bindings from normal `<script>` blocks.

use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ImportDeclarationSpecifier, Statement,
    VariableDeclaration, VariableDeclarationKind,
};
use vize_carton::ToCompactString;

use crate::types::{BindingMetadata, BindingType};

/// Convert Croquis BindingMetadata (CompactString keys) to legacy BindingMetadata (String keys)
pub(super) fn croquis_to_legacy_bindings(
    src: &vize_croquis::analysis::BindingMetadata,
) -> BindingMetadata {
    let mut dst = BindingMetadata::default();
    dst.is_script_setup = src.is_script_setup;
    for (name, bt) in src.iter() {
        dst.bindings.insert(name.to_compact_string(), bt);
    }
    for (local, key) in &src.props_aliases {
        dst.props_aliases
            .insert(local.to_compact_string(), key.to_compact_string());
    }
    dst
}

/// Collect bindings from normal `<script>` block.
///
/// When both `<script>` and `<script setup>` exist, top-level imports and
/// declarations from the normal script are accessible in the template.
/// Uses OXC parser for accurate import and declaration extraction (handles
/// `import { Form as PForm }`, default imports, and namespace imports).
pub(super) fn collect_normal_script_bindings(content: &str) -> BindingMetadata {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();
    let mut bindings = BindingMetadata::default();

    if ret.panicked {
        return bindings;
    }

    for stmt in ret.program.body.iter() {
        match stmt {
            // Register import bindings: import { Foo, Bar as Baz } from '...'
            Statement::ImportDeclaration(decl) => {
                // Skip type-only imports (import type { ... } from '...')
                if decl.import_kind.is_type() {
                    continue;
                }
                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                // Skip type-only specifiers
                                if s.import_kind.is_type() {
                                    continue;
                                }
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                let local = s.local.name.to_compact_string();
                                bindings
                                    .bindings
                                    .entry(local)
                                    .or_insert(BindingType::SetupConst);
                            }
                        }
                    }
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                register_variable_declaration(var_decl, &mut bindings);
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    bindings
                        .bindings
                        .entry(id.name.to_compact_string())
                        .or_insert(BindingType::SetupConst);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    bindings
                        .bindings
                        .entry(id.name.to_compact_string())
                        .or_insert(BindingType::SetupConst);
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(ref declaration) = decl.declaration {
                    match declaration {
                        Declaration::VariableDeclaration(var_decl) => {
                            register_variable_declaration(var_decl, &mut bindings);
                        }
                        Declaration::FunctionDeclaration(func) => {
                            if let Some(id) = &func.id {
                                bindings
                                    .bindings
                                    .entry(id.name.to_compact_string())
                                    .or_insert(BindingType::SetupConst);
                            }
                        }
                        Declaration::ClassDeclaration(class) => {
                            if let Some(id) = &class.id {
                                bindings
                                    .bindings
                                    .entry(id.name.to_compact_string())
                                    .or_insert(BindingType::SetupConst);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    bindings
}

fn register_variable_declaration(
    var_decl: &VariableDeclaration<'_>,
    bindings: &mut BindingMetadata,
) {
    for declarator in &var_decl.declarations {
        let binding_type = infer_variable_binding_type(var_decl.kind, declarator.init.as_ref());
        register_binding_pattern(&declarator.id, binding_type, bindings);
    }
}

fn infer_variable_binding_type(
    kind: VariableDeclarationKind,
    init: Option<&Expression<'_>>,
) -> BindingType {
    let Some(init) = init else {
        return match kind {
            VariableDeclarationKind::Const
            | VariableDeclarationKind::Using
            | VariableDeclarationKind::AwaitUsing => BindingType::SetupConst,
            VariableDeclarationKind::Let | VariableDeclarationKind::Var => BindingType::SetupLet,
        };
    };

    if let Expression::CallExpression(call) = init
        && let Expression::Identifier(callee) = &call.callee
    {
        match callee.name.as_str() {
            "ref" | "shallowRef" | "customRef" | "toRef" | "toRefs" | "computed"
            | "useTemplateRef" => return BindingType::SetupRef,
            "reactive" | "shallowReactive" | "readonly" | "shallowReadonly" => {
                return BindingType::SetupReactiveConst;
            }
            _ => {}
        }
    }

    if kind == VariableDeclarationKind::Const {
        if is_literal(init) {
            return BindingType::LiteralConst;
        }
        if matches!(
            init,
            Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ObjectExpression(_)
                | Expression::ArrayExpression(_)
        ) {
            return BindingType::SetupConst;
        }
    }

    match kind {
        VariableDeclarationKind::Const => BindingType::SetupMaybeRef,
        VariableDeclarationKind::Let | VariableDeclarationKind::Var => BindingType::SetupLet,
        VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => {
            BindingType::SetupConst
        }
    }
}

fn is_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        Expression::TemplateLiteral(tl) => tl.expressions.is_empty(),
        _ => false,
    }
}

fn register_binding_pattern(
    pattern: &BindingPattern<'_>,
    binding_type: BindingType,
    bindings: &mut BindingMetadata,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            bindings
                .bindings
                .entry(id.name.to_compact_string())
                .or_insert(binding_type);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                register_binding_pattern(&prop.value, binding_type, bindings);
            }
            if let Some(rest) = obj.rest.as_ref() {
                register_binding_pattern(&rest.argument, binding_type, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                register_binding_pattern(element, binding_type, bindings);
            }
            if let Some(rest) = arr.rest.as_ref() {
                register_binding_pattern(&rest.argument, binding_type, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            register_binding_pattern(&assign.left, binding_type, bindings);
        }
    }
}

/// Merge normal script bindings without changing the target metadata flags.
pub(super) fn merge_normal_script_bindings(
    target: &mut BindingMetadata,
    normal_bindings: &BindingMetadata,
) {
    for (name, binding_type) in &normal_bindings.bindings {
        target.bindings.entry(name.clone()).or_insert(*binding_type);
    }
}
