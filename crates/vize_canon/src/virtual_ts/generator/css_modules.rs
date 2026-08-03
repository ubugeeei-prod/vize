//! Script-side CSS-module typing for `useCssModule()`.

use std::collections::BTreeMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, cstr};

use crate::virtual_ts::types::{CSS_MODULE_GLOBAL_MARKER, VirtualTsOptions};

/// Inserts a type assertion immediately after a top-level `useCssModule()`
/// initializer. Vue's public return type is intentionally open-ended, while an
/// SFC's authored inline module can be narrower. Calls with a dynamic name and
/// modules that require the index-signature fallback are left untouched.
pub(super) struct CssModuleAssertions {
    insertions: Vec<(usize, String)>,
    index: usize,
}

impl CssModuleAssertions {
    pub(super) fn new(script: &str, options: &VirtualTsOptions) -> Self {
        let module_types: BTreeMap<&str, &str> = options
            .template_globals
            .iter()
            .filter(|global| global.default_value == CSS_MODULE_GLOBAL_MARKER)
            .map(|global| (global.name.as_str(), global.type_annotation.as_str()))
            .collect();
        if module_types.is_empty() {
            return Self {
                insertions: Vec::new(),
                index: 0,
            };
        }

        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::tsx()).parse();
        if parsed.panicked {
            return Self {
                insertions: Vec::new(),
                index: 0,
            };
        }

        let mut local_names = FxHashSet::default();
        for statement in parsed.program.body.iter() {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.source.value.as_str() != "vue" {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                if specifier.imported.name().as_str() == "useCssModule" {
                    local_names.insert(specifier.local.name.as_str());
                }
            }
        }

        let mut insertions = Vec::new();
        if !local_names.is_empty() {
            for statement in parsed.program.body.iter() {
                let Statement::VariableDeclaration(declaration) = statement else {
                    continue;
                };
                for declarator in declaration.declarations.iter() {
                    let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
                        continue;
                    };
                    let Expression::Identifier(callee) = &call.callee else {
                        continue;
                    };
                    if !local_names.contains(callee.name.as_str()) {
                        continue;
                    }
                    let module_name = match call.arguments.as_slice() {
                        [] => "$style",
                        [Argument::StringLiteral(name)] => name.value.as_str(),
                        _ => continue,
                    };
                    let Some(type_annotation) = module_types.get(module_name) else {
                        continue;
                    };
                    insertions.push((call.span.end as usize, cstr!(" as {type_annotation}")));
                }
            }
        }
        insertions.sort_by_key(|(offset, _)| *offset);
        Self {
            insertions,
            index: 0,
        }
    }

    pub(super) fn splice_output_line<'a>(
        &mut self,
        output_line: &mut std::borrow::Cow<'a, str>,
        line_start: usize,
    ) {
        while self.index < self.insertions.len() && self.insertions[self.index].0 <= line_start {
            self.index += 1;
        }
        let line_end = line_start + output_line.len();
        if self.index >= self.insertions.len() || self.insertions[self.index].0 > line_end {
            return;
        }

        let mut rewritten = String::default();
        let mut copied_until = 0usize;
        while self.index < self.insertions.len() {
            let (offset, assertion) = &self.insertions[self.index];
            if *offset > line_end {
                break;
            }
            let column = *offset - line_start;
            if output_line.is_char_boundary(column) {
                rewritten.push_str(&output_line[copied_until..column]);
                rewritten.push_str(assertion);
                copied_until = column;
            }
            self.index += 1;
        }
        if copied_until != 0 {
            rewritten.push_str(&output_line[copied_until..]);
            *output_line = std::borrow::Cow::Owned(rewritten.into());
        }
    }
}
