//! Statement-level binding and compiler-macro extraction.

use oxc_ast::ast::{
    Argument, BindingPattern, Expression, ObjectPattern, PropertyKey, Statement,
    VariableDeclarationKind,
};
use oxc_span::GetSpan;
use vize_carton::{String, ToCompactString};

use crate::types::BindingType;

use super::super::super::{MacroCall, process_props_destructure};
use super::super::ScriptCompileContext;
use super::super::helpers::{
    extract_args_from_call, extract_macro_from_expr, extract_type_args_from_call,
    infer_binding_type, is_call_of, is_import_type_only, macro_binding_name,
    register_binding_pattern,
};

impl ScriptCompileContext {
    /// Process a statement
    pub(super) fn process_statement(&mut self, stmt: &Statement<'_>, source: &str) {
        match stmt {
            Statement::ImportDeclaration(import_decl) => {
                // Skip type-only import declarations: import type { ... } from '...'
                if import_decl.import_kind.is_type() || is_import_type_only(import_decl, source) {
                    return;
                }

                // Process imports - add them to bindings so template knows about them
                if let Some(specifiers) = &import_decl.specifiers {
                    for specifier in specifiers.iter() {
                        match specifier {
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                // Named import: import { foo } from 'bar'
                                // Skip type-only imports: import { type Foo } from 'bar'
                                if !spec.import_kind.is_type() {
                                    let name = spec.local.name.to_compact_string();
                                    // Imports are treated as setup-maybe-ref since we don't know their type
                                    self.bindings
                                        .bindings
                                        .insert(name, BindingType::SetupMaybeRef);
                                }
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(
                                spec,
                            ) => {
                                // Default import: import Foo from 'bar'
                                let name = spec.local.name.to_compact_string();
                                // Default imports of .vue files are typically components
                                self.bindings.bindings.insert(name, BindingType::SetupConst);
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                                spec,
                            ) => {
                                // Namespace import: import * as foo from 'bar'
                                let name = spec.local.name.to_compact_string();
                                self.bindings.bindings.insert(name, BindingType::SetupConst);
                            }
                        }
                    }
                }
            }
            Statement::VariableDeclaration(var_decl) => {
                for decl in var_decl.declarations.iter() {
                    let binding_name = macro_binding_name(&decl.id);

                    // Check if init is a macro call
                    if let Some(init) = &decl.init {
                        if let Some(mut macro_call) = extract_macro_from_expr(init, source) {
                            // Attach binding name to macro call
                            macro_call.1.binding_name = binding_name.clone();
                            self.register_macro(&macro_call.0, macro_call.1);
                        }

                        // Check for withDefaults wrapping
                        if let Expression::CallExpression(call) = init
                            && is_call_of(call, "withDefaults")
                        {
                            self.macros.with_defaults = Some(MacroCall::new(
                                call.span.start as usize,
                                call.span.end as usize,
                                source[call.span.start as usize..call.span.end as usize].into(),
                                None,
                                binding_name.as_deref().map(Into::into),
                            ));

                            // Also extract the inner defineProps
                            if let Some(Argument::CallExpression(inner_call)) =
                                call.arguments.first()
                                && is_call_of(inner_call, "defineProps")
                            {
                                let type_args = extract_type_args_from_call(inner_call, source);
                                let props_call = MacroCall::new(
                                    inner_call.span.start as usize,
                                    inner_call.span.end as usize,
                                    extract_args_from_call(inner_call, source),
                                    type_args,
                                    binding_name.as_deref().map(String::from),
                                );
                                self.extract_props_bindings(&props_call);
                                self.macros.define_props = Some(props_call);
                                self.has_define_props_call = true;
                            }
                        }
                    }

                    // Extract binding name(s)
                    match &decl.id {
                        BindingPattern::BindingIdentifier(id) => {
                            let name = id.name.to_compact_string();

                            // Determine binding type
                            let binding_type = if let Some(init) = &decl.init {
                                infer_binding_type(init, var_decl.kind)
                            } else {
                                match var_decl.kind {
                                    VariableDeclarationKind::Const => BindingType::SetupConst,
                                    VariableDeclarationKind::Let => BindingType::SetupLet,
                                    VariableDeclarationKind::Var => BindingType::SetupLet,
                                    VariableDeclarationKind::Using
                                    | VariableDeclarationKind::AwaitUsing => {
                                        BindingType::SetupConst
                                    }
                                }
                            };

                            self.bindings.bindings.insert(name, binding_type);
                        }
                        BindingPattern::ObjectPattern(obj_pat) => {
                            // Handle destructuring like: const { prop1, prop2 } = defineProps()
                            let mut is_props_destructure = false;
                            if let Some(init) = &decl.init
                                && let Some((macro_name, macro_call)) =
                                    extract_macro_from_expr(init, source)
                                && macro_name == "defineProps"
                            {
                                is_props_destructure = true;

                                // Register defineProps macro (for type args / runtime props)
                                self.extract_props_bindings(&macro_call);
                                self.macros.define_props = Some(macro_call.clone());
                                self.has_define_props_call = true;

                                // Use the proper process_props_destructure function
                                let (destructure, binding_metadata, props_aliases) =
                                    process_props_destructure(obj_pat, source);
                                self.record_props_destructure_default_spans(obj_pat);

                                // Merge binding metadata
                                for (name, binding_type) in binding_metadata {
                                    self.bindings.bindings.insert(name, binding_type);
                                }

                                // Store props aliases
                                for (local, key) in props_aliases {
                                    self.bindings.props_aliases.insert(local, key);
                                }

                                self.macros.props_destructure = Some(destructure);
                            }

                            // Register each destructured binding (skip for props destructure)
                            if !is_props_destructure {
                                // Infer binding type from the initializer.
                                // For `const { x, y } = useComposable()`, each destructured
                                // property might be a ref, so we use the same inference as
                                // non-destructured declarations. This ensures _unref() is
                                // applied in templates for composable returns.
                                let destructure_type = if let Some(init) = &decl.init {
                                    infer_binding_type(init, var_decl.kind)
                                } else {
                                    match var_decl.kind {
                                        VariableDeclarationKind::Const => BindingType::SetupConst,
                                        _ => BindingType::SetupLet,
                                    }
                                };
                                register_binding_pattern(
                                    &mut self.bindings,
                                    &decl.id,
                                    destructure_type,
                                );
                            }
                        }
                        BindingPattern::ArrayPattern(arr_pat) => {
                            let destructure_type = if let Some(init) = &decl.init {
                                infer_binding_type(init, var_decl.kind)
                            } else {
                                match var_decl.kind {
                                    VariableDeclarationKind::Const => BindingType::SetupConst,
                                    _ => BindingType::SetupLet,
                                }
                            };
                            let is_define_model = decl
                                .init
                                .as_ref()
                                .and_then(|init| extract_macro_from_expr(init, source))
                                .is_some_and(|(name, _)| name == "defineModel");
                            for (index, elem) in arr_pat.elements.iter().enumerate() {
                                let Some(elem) = elem else {
                                    continue;
                                };
                                let binding_type = if is_define_model {
                                    if index == 0 {
                                        BindingType::SetupRef
                                    } else {
                                        BindingType::SetupConst
                                    }
                                } else {
                                    destructure_type
                                };
                                register_binding_pattern(&mut self.bindings, elem, binding_type);
                            }
                            if let Some(rest) = arr_pat.rest.as_ref() {
                                register_binding_pattern(
                                    &mut self.bindings,
                                    &rest.argument,
                                    destructure_type,
                                );
                            }
                        }
                        BindingPattern::AssignmentPattern(assign_pat) => {
                            register_binding_pattern(
                                &mut self.bindings,
                                &assign_pat.left,
                                BindingType::SetupConst,
                            );
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    self.bindings
                        .bindings
                        .insert(id.name.to_compact_string(), BindingType::SetupConst);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    self.bindings
                        .bindings
                        .insert(id.name.to_compact_string(), BindingType::SetupConst);
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                // Handle standalone macro calls like defineExpose({...})
                if let Some(macro_call) = extract_macro_from_expr(&expr_stmt.expression, source) {
                    self.register_macro(&macro_call.0, macro_call.1);
                }

                // Handle standalone withDefaults(defineProps<...>(), {...})
                if let Expression::CallExpression(call) = &expr_stmt.expression
                    && is_call_of(call, "withDefaults")
                {
                    self.macros.with_defaults = Some(MacroCall::new(
                        call.span.start as usize,
                        call.span.end as usize,
                        source[call.span.start as usize..call.span.end as usize].into(),
                        None,
                        None,
                    ));

                    // Also extract the inner defineProps
                    if let Some(Argument::CallExpression(inner_call)) = call.arguments.first()
                        && is_call_of(inner_call, "defineProps")
                    {
                        let type_args = extract_type_args_from_call(inner_call, source);
                        let props_call = MacroCall::new(
                            inner_call.span.start as usize,
                            inner_call.span.end as usize,
                            extract_args_from_call(inner_call, source),
                            type_args,
                            None,
                        );
                        self.extract_props_bindings(&props_call);
                        self.macros.define_props = Some(props_call);
                        self.has_define_props_call = true;
                    }
                }
            }
            // TypeScript declarations are handled in the first pass
            Statement::TSInterfaceDeclaration(_) | Statement::TSTypeAliasDeclaration(_) => {}
            _ => {}
        }
    }

    /// Register a macro call
    pub(super) fn register_macro(&mut self, name: &str, call: MacroCall) {
        match name {
            "defineProps" => {
                // Extract prop names and add to bindings
                self.extract_props_bindings(&call);
                self.macros.define_props = Some(call);
            }
            "defineEmits" => self.macros.define_emits = Some(call),
            "defineExpose" => self.macros.define_expose = Some(call),
            "defineOptions" => self.macros.define_options = Some(call),
            "defineSlots" => self.macros.define_slots = Some(call),
            "defineModel" => self.macros.define_models.push(call),
            "withDefaults" => {
                // Note: Props are extracted from the inner defineProps call
                // in the separate withDefaults handling block in process_statement
                self.macros.with_defaults = Some(call);
            }
            _ => {}
        }
    }

    fn record_props_destructure_default_spans(&mut self, pattern: &ObjectPattern<'_>) {
        for property in &pattern.properties {
            let name = match &property.key {
                PropertyKey::StaticIdentifier(identifier) => identifier.name.to_compact_string(),
                PropertyKey::StringLiteral(literal) => literal.value.to_compact_string(),
                _ => continue,
            };
            let BindingPattern::AssignmentPattern(assignment) = &property.value else {
                continue;
            };
            let span = assignment.right.span();
            self.props_destructure_default_spans
                .insert(name, (span.start, span.end));
        }
    }
}
