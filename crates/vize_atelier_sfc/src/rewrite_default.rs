//! Rewrite default export to a variable declaration.
//!
//! This module transforms `export default` declarations to variable declarations,
//! allowing the compiler to inject properties like render functions.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString, profile};

use crate::module_map::{Runs, TracedText};

/// Rewrite `export default` to a const declaration with the given name.
/// Returns (rewritten_code, has_default_export)
pub fn rewrite_default(input: &str, as_name: &str, is_ts: bool) -> (String, bool) {
    let (code, has_default, _) = rewrite_default_traced(input, as_name, is_ts);
    (code, has_default)
}

/// [`rewrite_default`] plus the rewritten code's provenance in `input`: text
/// outside the rewritten statements is copied, and each rewrite is anchored at
/// the statement it replaced (Davinci P3-9 source maps).
pub(crate) fn rewrite_default_traced(
    input: &str,
    as_name: &str,
    is_ts: bool,
) -> (String, bool, Runs) {
    let source_type = if is_ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let allocator = Allocator::default();
    let ret = profile!(
        "atelier.normal_script.rewrite_default.parse",
        Parser::new(&allocator, input, source_type).parse()
    );

    if !ret.diagnostics.is_empty() {
        // If parsing fails, return original code
        return (
            input.to_compact_string(),
            false,
            Runs::identity(input.len()),
        );
    }

    let program = ret.program;

    // Check if there's a default export
    let has_default = program.body.iter().any(|stmt| {
        matches!(stmt, Statement::ExportDefaultDeclaration(_))
            || matches!(stmt, Statement::ExportNamedDeclaration(decl)
                if decl.specifiers.iter().any(|s| {
                    matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                        || matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default")
                }))
    });

    if !has_default {
        // No default export - append empty object
        let mut output = input.to_compact_string();
        output.push_str("\nconst ");
        output.push_str(as_name);
        output.push_str(" = {}");
        return (output, false, Runs::identity(input.len()));
    }

    // Find and rewrite the default export
    let mut output = TracedText::with_capacity(input, input.len() + as_name.len() + 32);
    let mut last_end = 0;

    for stmt in program.body.iter() {
        match stmt {
            Statement::ExportDefaultDeclaration(decl) => {
                // Copy everything before this statement
                output.copy(last_end, decl.span.start as usize);
                output.mark(decl.span.start as usize);

                match &decl.declaration {
                    ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                        // export default class Foo {} -> class Foo {} \n const as_name = Foo
                        if let Some(id) = &class_decl.id {
                            let class_start = class_decl.span.start as usize;
                            output.copy(class_start, decl.span.end as usize);
                            output.push_str("\nconst ");
                            output.push_str(as_name);
                            output.push_str(" = ");
                            output.push_str(id.name.as_str());
                        } else {
                            // Anonymous class - wrap in const
                            output.push_str("const ");
                            output.push_str(as_name);
                            output.push_str(" = ");
                            let class_start = class_decl.span.start as usize;
                            output.copy(class_start, decl.span.end as usize);
                        }
                    }
                    ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) => {
                        // export default function foo() {} -> function foo() {} \n const as_name = foo
                        if let Some(id) = &func_decl.id {
                            let func_start = func_decl.span.start as usize;
                            output.copy(func_start, decl.span.end as usize);
                            output.push_str("\nconst ");
                            output.push_str(as_name);
                            output.push_str(" = ");
                            output.push_str(id.name.as_str());
                        } else {
                            // Anonymous function - wrap in const
                            output.push_str("const ");
                            output.push_str(as_name);
                            output.push_str(" = ");
                            let func_start = func_decl.span.start as usize;
                            output.copy(func_start, decl.span.end as usize);
                        }
                    }
                    _ => {
                        // export default {...} -> const as_name = {...}
                        output.push_str("const ");
                        output.push_str(as_name);
                        output.push_str(" = ");
                        let expr_start = decl.declaration.span().start as usize;
                        let expr_end = decl.declaration.span().end as usize;
                        output.copy(expr_start, expr_end);
                    }
                }

                last_end = decl.span.end as usize;
            }
            Statement::ExportNamedDeclaration(named_decl) => {
                // Handle: export { foo as default }
                let has_default_specifier = named_decl.specifiers.iter().any(|s| {
                    matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                        || matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default")
                });

                if has_default_specifier {
                    // Copy everything before this statement
                    output.copy(last_end, named_decl.span.start as usize);
                    output.mark(named_decl.span.start as usize);

                    if let Some(source) = &named_decl.source {
                        // export { default } from '...' or export { foo as default } from '...'
                        for specifier in &named_decl.specifiers {
                            let is_default = matches!(&specifier.exported,
                                oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                                || matches!(&specifier.exported,
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default");

                            if is_default {
                                let local_name = match &specifier.local {
                                    oxc_ast::ast::ModuleExportName::IdentifierName(name) => {
                                        name.name.as_str()
                                    }
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) => {
                                        name.name.as_str()
                                    }
                                    _ => "default",
                                };

                                // Add import for the default
                                output.push_str("import { ");
                                output.push_str(local_name);
                                output.push_str(" as __VUE_DEFAULT__ } from '");
                                output.push_str(source.value.as_str());
                                output.push_str("'\n");
                            }
                        }

                        // Rebuild export without the default specifier
                        let other_specifiers: Vec<_> = named_decl
                            .specifiers
                            .iter()
                            .filter(|s| {
                                !matches!(&s.exported,
                                    oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                                    && !matches!(&s.exported,
                                        oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default")
                            })
                            .collect();

                        if !other_specifiers.is_empty() {
                            output.push_str("export { ");
                            for (i, spec) in other_specifiers.iter().enumerate() {
                                if i > 0 {
                                    output.push_str(", ");
                                }
                                let local = match &spec.local {
                                    oxc_ast::ast::ModuleExportName::IdentifierName(name) => {
                                        name.name.as_str()
                                    }
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) => {
                                        name.name.as_str()
                                    }
                                    _ => continue,
                                };
                                let exported = match &spec.exported {
                                    oxc_ast::ast::ModuleExportName::IdentifierName(name) => {
                                        name.name.as_str()
                                    }
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) => {
                                        name.name.as_str()
                                    }
                                    _ => continue,
                                };
                                if local == exported {
                                    output.push_str(local);
                                } else {
                                    output.push_str(local);
                                    output.push_str(" as ");
                                    output.push_str(exported);
                                }
                            }
                            output.push_str(" } from '");
                            output.push_str(source.value.as_str());
                            output.push_str("'\n");
                        }

                        output.push_str("const ");
                        output.push_str(as_name);
                        output.push_str(" = __VUE_DEFAULT__");
                    } else {
                        // export { foo as default } (no source)
                        for specifier in &named_decl.specifiers {
                            let is_default = matches!(&specifier.exported,
                                oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                                || matches!(&specifier.exported,
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default");

                            if is_default {
                                let local_name = match &specifier.local {
                                    oxc_ast::ast::ModuleExportName::IdentifierName(name) => {
                                        name.name.as_str()
                                    }
                                    oxc_ast::ast::ModuleExportName::IdentifierReference(name) => {
                                        name.name.as_str()
                                    }
                                    _ => "default",
                                };

                                // Rebuild export without the default specifier
                                let other_specifiers: Vec<_> = named_decl
                                    .specifiers
                                    .iter()
                                    .filter(|s| {
                                        !matches!(&s.exported,
                                            oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                                            && !matches!(&s.exported,
                                                oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default")
                                    })
                                    .collect();

                                if !other_specifiers.is_empty() {
                                    output.push_str("export { ");
                                    for (i, spec) in other_specifiers.iter().enumerate() {
                                        if i > 0 {
                                            output.push_str(", ");
                                        }
                                        let local = match &spec.local {
                                            oxc_ast::ast::ModuleExportName::IdentifierName(
                                                name,
                                            ) => name.name.as_str(),
                                            oxc_ast::ast::ModuleExportName::IdentifierReference(
                                                name,
                                            ) => name.name.as_str(),
                                            _ => continue,
                                        };
                                        let exported = match &spec.exported {
                                            oxc_ast::ast::ModuleExportName::IdentifierName(
                                                name,
                                            ) => name.name.as_str(),
                                            oxc_ast::ast::ModuleExportName::IdentifierReference(
                                                name,
                                            ) => name.name.as_str(),
                                            _ => continue,
                                        };
                                        if local == exported {
                                            output.push_str(local);
                                        } else {
                                            output.push_str(local);
                                            output.push_str(" as ");
                                            output.push_str(exported);
                                        }
                                    }
                                    output.push_str(" }\n");
                                }

                                output.push_str("const ");
                                output.push_str(as_name);
                                output.push_str(" = ");
                                output.push_str(local_name);
                                break;
                            }
                        }
                    }

                    last_end = named_decl.span.end as usize;
                }
            }
            _ => {}
        }
    }

    // Copy remaining content
    if last_end < input.len() {
        output.copy(last_end, input.len());
    }

    let (output, runs) = output.into_parts();
    (output, has_default, runs)
}

#[cfg(test)]
mod tests;
