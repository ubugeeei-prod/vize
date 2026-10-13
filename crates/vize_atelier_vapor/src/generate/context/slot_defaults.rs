//! Parsed reads for destructure patterns the textual scanner cannot follow:
//! defaults, computed keys, rest elements and annotated patterns.

use oxc_ast::ast::{BindingPattern, ObjectPattern, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_atelier_core::steps::expression::expression_is_safe_to_parse;
use vize_carton::{SmallVec, String, cstr};

use super::GenerateContext;
use crate::generate::destructure::{DestructureBinding, escape_js_string_literal};

impl GenerateContext<'_> {
    pub(super) fn resolve_slot_defaults(&mut self, pattern: &str, root: &str) {
        self.resolve_pattern_reads(pattern, root, |ctx, binding| {
            if let Some(scope) = ctx.slot_scopes.last_mut() {
                if !scope.names.contains(&binding.local) {
                    scope.names.push(binding.local.clone());
                }
                scope.read_overrides.push(binding);
            }
        });
    }

    /// The parsed reads of a destructured `v-for` value, on the loop scope
    /// that was just pushed.
    pub(in crate::generate) fn resolve_for_value_reads(&mut self, pattern: &str, root: &str) {
        self.resolve_pattern_reads(pattern, root, |ctx, binding| {
            if let Some(scope) = ctx.for_scopes.last_mut() {
                scope.read_overrides.push(binding);
            }
        });
    }

    /// `pattern` as a callback parameter: its computed keys and defaults read
    /// the enclosing scope, since the callback receives the raw value and the
    /// loop scope being closed does not exist inside it.
    pub(in crate::generate) fn resolve_pattern_parameter(&mut self, pattern: &str) -> String {
        const PREFIX: &str = "let ";
        if !expression_is_safe_to_parse(pattern) {
            return String::from(pattern);
        }
        let source = cstr!("{PREFIX}{pattern} = __slotProps");
        let allocator = vize_atelier_core::expr_parse_probe::parse_arena();
        let parsed = Parser::new(&allocator, &source, SourceType::ts().with_module(true)).parse();
        let declarator = match parsed.program.body.first() {
            Some(Statement::VariableDeclaration(declaration)) if parsed.diagnostics.is_empty() => {
                declaration.declarations.first()
            }
            _ => None,
        };
        let Some(declarator) = declarator else {
            return String::from(pattern);
        };
        // Expression spans inside the pattern, in source order.
        let mut spans = SmallVec::<[(usize, usize); 4]>::new();
        let mut pending = SmallVec::<[&BindingPattern<'_>; 8]>::new();
        pending.push(&declarator.id);
        while let Some(pattern) = pending.pop() {
            match pattern {
                BindingPattern::BindingIdentifier(_) => {}
                BindingPattern::AssignmentPattern(assignment) => {
                    let span = assignment.right.span();
                    spans.push((span.start as usize, span.end as usize));
                    pending.push(&assignment.left);
                }
                BindingPattern::ObjectPattern(object) => {
                    for property in &object.properties {
                        if property.computed {
                            let span = property.key.span();
                            spans.push((span.start as usize, span.end as usize));
                        }
                        pending.push(&property.value);
                    }
                    if let Some(rest) = &object.rest {
                        pending.push(&rest.argument);
                    }
                }
                BindingPattern::ArrayPattern(array) => {
                    pending.extend(array.elements.iter().flatten());
                    if let Some(rest) = &array.rest {
                        pending.push(&rest.argument);
                    }
                }
            }
        }
        if spans.is_empty() {
            return String::from(pattern);
        }
        spans.sort_unstable();
        let own_scope = self.for_scopes.pop();
        let mut parameter = String::with_capacity(pattern.len());
        let mut cursor = PREFIX.len();
        for (start, end) in spans {
            // A default nested in another default was resolved with it.
            if start < cursor {
                continue;
            }
            parameter.push_str(source.get(cursor..start).unwrap_or_default());
            let expression = source.get(start..end).unwrap_or_default();
            parameter.push_str(&self.resolve_expression(expression));
            cursor = end;
        }
        parameter.push_str(
            source
                .get(cursor..PREFIX.len() + pattern.len())
                .unwrap_or_default(),
        );
        self.for_scopes.extend(own_scope);
        parameter
    }

    /// Hand `bind` every local `pattern` declares, in authored order, with the
    /// expression that reads it from `root`. Defaults stay lazy, computed keys
    /// resolve in the enclosing scope and rest elements exclude their sibling
    /// keys. A later default can read an earlier local through `bind`'s scope.
    fn resolve_pattern_reads(
        &mut self,
        pattern: &str,
        root: &str,
        mut bind: impl FnMut(&mut Self, DestructureBinding),
    ) {
        if !expression_is_safe_to_parse(pattern) {
            return;
        }
        let source = cstr!("let {pattern} = __slotProps");
        let allocator = vize_atelier_core::expr_parse_probe::parse_arena();
        let parsed = Parser::new(&allocator, &source, SourceType::ts().with_module(true)).parse();
        if !parsed.diagnostics.is_empty() {
            return;
        }
        let Some(Statement::VariableDeclaration(declaration)) = parsed.program.body.first() else {
            return;
        };
        let Some(declarator) = declaration.declarations.first() else {
            return;
        };
        let mut pending = SmallVec::<[(&BindingPattern<'_>, String); 8]>::new();
        pending.push((&declarator.id, String::from(root)));
        while let Some((pattern, read)) = pending.pop() {
            match pattern {
                BindingPattern::BindingIdentifier(identifier) => {
                    let local = String::from(identifier.name.as_str());
                    bind(self, DestructureBinding { local, path: read });
                }
                BindingPattern::AssignmentPattern(assignment) => {
                    let span = assignment.right.span();
                    let Some(default) = source.get(span.start as usize..span.end as usize) else {
                        continue;
                    };
                    let default = self.resolve_expression(default);
                    self.use_helper("getDefaultValue");
                    pending.push((
                        &assignment.left,
                        cstr!("_getDefaultValue({read}, () => ({default}))"),
                    ));
                }
                BindingPattern::ObjectPattern(object) => {
                    // Pushed first, so the rest element binds after the keys
                    // it excludes.
                    if let Some(rest) = &object.rest {
                        let excluded = self.rest_excluded_keys(object, &source);
                        self.use_helper("getRestElement");
                        pending.push((
                            &rest.argument,
                            cstr!("_getRestElement({read}, [{excluded}])"),
                        ));
                    }
                    for property in object.properties.iter().rev() {
                        let segment = if !property.computed
                            && let PropertyKey::StaticIdentifier(identifier) = &property.key
                        {
                            cstr!(".{}", identifier.name)
                        } else {
                            let span = property.key.span();
                            let Some(key) = source.get(span.start as usize..span.end as usize)
                            else {
                                continue;
                            };
                            let key = if property.computed {
                                self.resolve_expression(key)
                            } else {
                                String::from(key)
                            };
                            cstr!("[{key}]")
                        };
                        pending.push((&property.value, cstr!("{read}{segment}")));
                    }
                }
                BindingPattern::ArrayPattern(array) => {
                    if let Some(rest) = &array.rest {
                        pending.push((
                            &rest.argument,
                            cstr!("{read}.slice({})", array.elements.len()),
                        ));
                    }
                    for (index, item) in array.elements.iter().enumerate().rev() {
                        if let Some(item) = item {
                            pending.push((item, cstr!("{read}[{index}]")));
                        }
                    }
                }
            }
        }
    }

    /// The keys an object rest element leaves out, as the runtime compares
    /// them: property names as strings, computed keys as their value.
    fn rest_excluded_keys(&mut self, object: &ObjectPattern<'_>, source: &str) -> String {
        let mut excluded = String::default();
        for property in &object.properties {
            let span = property.key.span();
            let Some(text) = source.get(span.start as usize..span.end as usize) else {
                continue;
            };
            let key = if property.computed {
                self.resolve_expression(text)
            } else {
                match &property.key {
                    PropertyKey::StaticIdentifier(identifier) => {
                        cstr!("\"{}\"", identifier.name)
                    }
                    PropertyKey::StringLiteral(literal) => {
                        cstr!("\"{}\"", escape_js_string_literal(literal.value.as_str()))
                    }
                    _ => cstr!("\"{}\"", escape_js_string_literal(text)),
                }
            };
            if !excluded.is_empty() {
                excluded.push_str(", ");
            }
            excluded.push_str(&key);
        }
        excluded
    }
}
