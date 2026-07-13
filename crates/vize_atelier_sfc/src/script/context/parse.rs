//! OXC-based parsing and statement processing.
//!
//! Contains the core parsing logic that processes AST statements
//! and extracts macro calls, bindings, and type definitions.

mod statements;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Program, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use vize_carton::{String, ToCompactString, profile};

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

        self.process_program(&ret.program, source);
    }

    /// Extract information from an already-parsed program.
    ///
    /// Parse-free core of [`Self::parse_with_oxc`] for callers that already
    /// hold an oxc `Program` for `source` (the SFC compiler's parse-once
    /// pipeline). `source` must be the exact text the program was parsed from.
    pub(super) fn process_program(&mut self, program: &Program<'_>, source: &str) {
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
        self.collect_types_from_program(&ret.program, source);
    }

    /// Merge local interface and type-alias declarations from an already
    /// parsed normal script block.
    pub fn collect_types_from_program(&mut self, program: &Program<'_>, source: &str) {
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

    /// Merge owned local type facts captured by another script block.
    pub fn merge_types_from(&mut self, other: &Self) {
        for (name, body) in &other.interfaces {
            self.interfaces
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
        for (name, body) in &other.type_aliases {
            self.type_aliases
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
    }
}
