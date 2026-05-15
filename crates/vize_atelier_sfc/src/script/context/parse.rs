//! OXC-based parsing and statement processing.
//!
//! Contains the core parsing logic that processes AST statements
//! and extracts macro calls, bindings, and type definitions.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, BindingPattern, Expression, Statement, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use vize_carton::{profile, String, ToCompactString};

use crate::types::BindingType;

use super::super::define_props_destructure::process_props_destructure;
use super::super::MacroCall;
use super::helpers::{
    extract_args_from_call, extract_macro_from_expr, extract_type_args_from_call,
    infer_binding_type, is_call_of, is_import_type_only,
};
use super::ScriptCompileContext;
use crate::script::build_interface_type_source;

impl ScriptCompileContext {
    /// Parse the source with OXC and extract information
    pub(super) fn parse_with_oxc(&mut self, source: &str) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("script.ts").unwrap_or_default();

        let ret = profile!(
            "atelier.script.context.oxc_parse",
            Parser::new(&allocator, source, source_type).parse()
        );

        if ret.panicked {
            return;
        }

        let program = ret.program;

        // First pass: collect all TypeScript interfaces and type aliases
        // This ensures they're available when resolving type references in macros
        for stmt in program.body.iter() {
            match stmt {
                Statement::TSInterfaceDeclaration(iface) => {
                    let name = iface.id.name.to_compact_string();
                    let body = build_interface_type_source(
                        source,
                        iface.id.span.end as usize,
                        iface.body.span.start as usize,
                        iface.body.span.end as usize,
                    );
                    self.interfaces.insert(name, body);
                }
                Statement::TSTypeAliasDeclaration(type_alias) => {
                    let name = type_alias.id.name.to_compact_string();
                    let type_start = type_alias.type_annotation.span().start as usize;
                    let type_end = type_alias.type_annotation.span().end as usize;
                    let type_body = String::from(&source[type_start..type_end]);
                    self.type_aliases.insert(name, type_body);
                }
                // Handle exported types: `export type X = ...` and `export interface X { ... }`
                Statement::ExportNamedDeclaration(export_decl) => {
                    if let Some(ref decl) = export_decl.declaration {
                        match decl {
                            oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
                                let name = iface.id.name.to_compact_string();
                                let body = build_interface_type_source(
                                    source,
                                    iface.id.span.end as usize,
                                    iface.body.span.start as usize,
                                    iface.body.span.end as usize,
                                );
                                self.interfaces.insert(name, body);
                            }
                            oxc_ast::ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                                let name = type_alias.id.name.to_compact_string();
                                let type_start = type_alias.type_annotation.span().start as usize;
                                let type_end = type_alias.type_annotation.span().end as usize;
                                let type_body = String::from(&source[type_start..type_end]);
                                self.type_aliases.insert(name, type_body);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Second pass: process all statements (macros, bindings, etc.)
        profile!("atelier.script.context.process_statements", {
            for stmt in program.body.iter() {
                self.process_statement(stmt, source);
            }
        });

        // Update flags
        self.has_define_props_call = self.macros.define_props.is_some();
        self.has_define_emits_call = self.macros.define_emits.is_some();
        self.has_define_expose_call = self.macros.define_expose.is_some();
        self.has_define_options_call = self.macros.define_options.is_some();
        self.has_define_slots_call = self.macros.define_slots.is_some();
        self.has_define_model_call = !self.macros.define_models.is_empty();
    }

    /// Collect type definitions (interfaces and type aliases) from additional source.
    /// Used to merge types from the normal `<script>` block into the context
    /// so that `defineProps<TypeRef>()` can resolve type references across blocks.
    pub fn collect_types_from(&mut self, source: &str) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path("script.ts").unwrap_or_default();
        let ret = profile!(
            "atelier.script.context.collect_types_parse",
            Parser::new(&allocator, source, source_type).parse()
        );
        if ret.panicked {
            return;
        }
        for stmt in ret.program.body.iter() {
            match stmt {
                Statement::TSInterfaceDeclaration(iface) => {
                    let name = iface.id.name.to_compact_string();
                    let body = build_interface_type_source(
                        source,
                        iface.id.span.end as usize,
                        iface.body.span.start as usize,
                        iface.body.span.end as usize,
                    );
                    self.interfaces.entry(name).or_insert(body);
                }
                Statement::TSTypeAliasDeclaration(type_alias) => {
                    let name = type_alias.id.name.to_compact_string();
                    let type_start = type_alias.type_annotation.span().start as usize;
                    let type_end = type_alias.type_annotation.span().end as usize;
                    let type_body = String::from(&source[type_start..type_end]);
                    self.type_aliases.entry(name).or_insert(type_body);
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    if let Some(ref decl) = export_decl.declaration {
                        match decl {
                            oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
                                let name = iface.id.name.to_compact_string();
                                let body = build_interface_type_source(
                                    source,
                                    iface.id.span.end as usize,
                                    iface.body.span.start as usize,
                                    iface.body.span.end as usize,
                                );
                                self.interfaces.entry(name).or_insert(body);
                            }
                            oxc_ast::ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                                let name = type_alias.id.name.to_compact_string();
                                let type_start = type_alias.type_annotation.span().start as usize;
                                let type_end = type_alias.type_annotation.span().end as usize;
                                let type_body = String::from(&source[type_start..type_end]);
                                self.type_aliases.entry(name).or_insert(type_body);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Process a statement
    fn process_statement(&mut self, stmt: &Statement<'_>, source: &str) {
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
                    // Extract binding name if this is a simple identifier binding
                    let binding_name = match &decl.id {
                        BindingPattern::BindingIdentifier(id) => Some(id.name.to_compact_string()),
                        _ => None,
                    };

                    // Check if init is a macro call
                    if let Some(init) = &decl.init {
                        if let Some(mut macro_call) = extract_macro_from_expr(init, source) {
                            // Attach binding name to macro call
                            macro_call.1.binding_name = binding_name.as_deref().map(Into::into);
                            self.register_macro(&macro_call.0, macro_call.1);
                        }

                        // Check for withDefaults wrapping
                        if let Expression::CallExpression(call) = init {
                            if is_call_of(call, "withDefaults") {
                                self.macros.with_defaults = Some(MacroCall {
                                    start: call.span.start as usize,
                                    end: call.span.end as usize,
                                    args: source[call.span.start as usize..call.span.end as usize]
                                        .into(),
                                    type_args: None,
                                    binding_name: binding_name.as_deref().map(Into::into),
                                });

                                // Also extract the inner defineProps
                                if let Some(Argument::CallExpression(inner_call)) =
                                    call.arguments.first()
                                {
                                    if is_call_of(inner_call, "defineProps") {
                                        let type_args =
                                            extract_type_args_from_call(inner_call, source);
                                        let props_call = MacroCall {
                                            start: inner_call.span.start as usize,
                                            end: inner_call.span.end as usize,
                                            args: extract_args_from_call(inner_call, source),
                                            type_args,
                                            binding_name: binding_name.as_deref().map(String::from),
                                        };
                                        self.extract_props_bindings(&props_call);
                                        self.macros.define_props = Some(props_call);
                                        self.has_define_props_call = true;
                                    }
                                }
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
                            if let Some(init) = &decl.init {
                                if let Some((macro_name, macro_call)) =
                                    extract_macro_from_expr(init, source)
                                {
                                    if macro_name == "defineProps" {
                                        is_props_destructure = true;

                                        // Register defineProps macro (for type args / runtime props)
                                        self.extract_props_bindings(&macro_call);
                                        self.macros.define_props = Some(macro_call.clone());
                                        self.has_define_props_call = true;

                                        // Use the proper process_props_destructure function
                                        let (destructure, binding_metadata, props_aliases) =
                                            process_props_destructure(obj_pat, source);

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
                                }
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
                                for prop in obj_pat.properties.iter() {
                                    if let BindingPattern::BindingIdentifier(id) = &prop.value {
                                        self.bindings
                                            .bindings
                                            .insert(id.name.to_compact_string(), destructure_type);
                                    }
                                }
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
                            for elem in arr_pat.elements.iter().flatten() {
                                if let BindingPattern::BindingIdentifier(id) = &elem {
                                    self.bindings
                                        .bindings
                                        .insert(id.name.to_compact_string(), destructure_type);
                                }
                            }
                        }
                        BindingPattern::AssignmentPattern(assign_pat) => {
                            if let BindingPattern::BindingIdentifier(id) = &assign_pat.left {
                                self.bindings
                                    .bindings
                                    .insert(id.name.to_compact_string(), BindingType::SetupConst);
                            }
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
                if let Expression::CallExpression(call) = &expr_stmt.expression {
                    if is_call_of(call, "withDefaults") {
                        self.macros.with_defaults = Some(MacroCall {
                            start: call.span.start as usize,
                            end: call.span.end as usize,
                            args: source[call.span.start as usize..call.span.end as usize].into(),
                            type_args: None,
                            binding_name: None,
                        });

                        // Also extract the inner defineProps
                        if let Some(Argument::CallExpression(inner_call)) = call.arguments.first() {
                            if is_call_of(inner_call, "defineProps") {
                                let type_args = extract_type_args_from_call(inner_call, source);
                                let props_call = MacroCall {
                                    start: inner_call.span.start as usize,
                                    end: inner_call.span.end as usize,
                                    args: extract_args_from_call(inner_call, source),
                                    type_args,
                                    binding_name: None,
                                };
                                self.extract_props_bindings(&props_call);
                                self.macros.define_props = Some(props_call);
                                self.has_define_props_call = true;
                            }
                        }
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
}
