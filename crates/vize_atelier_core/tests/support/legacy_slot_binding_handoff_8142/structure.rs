//! Test-only full Module AST observer. Raw references include framework/type
//! names; semantic vectors omit only bindings established by actual AST nodes.
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, CallExpression, Expression, FormalParameters,
    Function, IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier,
    StaticMemberExpression, TSTypeAnnotation, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, SourceType};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn observe(source: &str) -> Value {
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, source, SourceType::ts().with_module(true)).parse();
    let mut observer = Observer {
        source,
        owner: "render".to_owned(),
        imports: BTreeMap::new(),
        framework_bindings: BTreeSet::new(),
        render_machinery: BTreeSet::new(),
        slot_bindings: Vec::new(),
        display_calls: Vec::new(),
        reads: Vec::new(),
        identifiers: Vec::new(),
        type_depth: 0,
        ctx_object_depth: 0,
        slot_arrow: None,
        nested_functions: Vec::new(),
        arrow_count: 0,
        function_count: 0,
    };
    observer.visit_program(&parsed.program);
    json!({
        "source": source,
        "parse": {"panicked": parsed.panicked, "diagnosticsDebug": format!("{:#?}", parsed.diagnostics.iter().collect::<Vec<_>>())},
        "rawDiagnosticsContainerDebug": format!("{:#?}", parsed.diagnostics),
        "structure": {"slotBindings": observer.slot_bindings, "displayCalls": observer.display_calls, "reads": observer.reads, "nestedFunctions": observer.nested_functions},
        "allIdentifierReferences": observer.identifiers,
        "vueImportAliases": observer.imports,
        "generatedResolutionBindings": observer.framework_bindings,
        "renderMachineryBindings": observer.render_machinery,
    })
}

struct Observer<'s> {
    source: &'s str,
    owner: String,
    imports: BTreeMap<String, String>,
    framework_bindings: BTreeSet<String>,
    render_machinery: BTreeSet<String>,
    slot_bindings: Vec<Vec<String>>,
    display_calls: Vec<Value>,
    reads: Vec<Value>,
    identifiers: Vec<Value>,
    type_depth: usize,
    ctx_object_depth: usize,
    slot_arrow: Option<usize>,
    nested_functions: Vec<Value>,
    arrow_count: usize,
    function_count: usize,
}

fn binding_names(pattern: &BindingPattern<'_>, names: &mut Vec<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => names.push(format!("{}", id.name)),
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for item in arr.elements.iter().flatten() {
                binding_names(item, names);
            }
            if let Some(rest) = &arr.rest {
                binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => binding_names(&assign.left, names),
    }
}

fn parameter_names(params: &FormalParameters<'_>) -> Vec<String> {
    let mut names = Vec::new();
    for param in &params.items {
        binding_names(&param.pattern, &mut names);
    }
    if let Some(rest) = &params.rest {
        binding_names(&rest.rest.argument, &mut names);
    }
    names
}

impl<'a> Visit<'a> for Observer<'_> {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        if declaration.source.value == "vue" {
            for specifier in declaration.specifiers.iter().flatten() {
                if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
                    self.imports.insert(
                        format!("{}", specifier.local.name),
                        format!("{}", specifier.imported.name()),
                    );
                }
            }
        }
        walk::walk_import_declaration(self, declaration);
    }

    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &declaration.id
            && let Some(Expression::CallExpression(call)) = &declaration.init
            && let Expression::Identifier(callee) = &call.callee
            && matches!(
                self.imports.get(callee.name.as_str()).map(String::as_str),
                Some("resolveComponent" | "resolveFilter")
            )
        {
            self.framework_bindings.insert(format!("{}", id.name));
        }
        walk::walk_variable_declarator(self, declaration);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if function.id.as_ref().is_some_and(|id| id.name == "render") {
            self.render_machinery.extend(
                parameter_names(&function.params)
                    .into_iter()
                    .filter(|name| name != "_ctx"),
            );
            walk::walk_function(self, function, flags);
        } else {
            let owner = format!("{}/function:{}", self.owner, self.function_count);
            self.function_count += 1;
            self.nested_functions.push(json!({"owner": owner, "kind": "function", "bindings": parameter_names(&function.params)}));
            let previous = std::mem::replace(&mut self.owner, owner);
            walk::walk_function(self, function, flags);
            self.owner = previous;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.slot_arrow == Some(core::ptr::from_ref(arrow) as usize) {
            walk::walk_arrow_function_expression(self, arrow);
        } else {
            let owner = format!("{}/arrow:{}", self.owner, self.arrow_count);
            self.arrow_count += 1;
            self.nested_functions.push(json!({"owner": owner, "kind": "arrow", "bindings": parameter_names(&arrow.params)}));
            let previous = std::mem::replace(&mut self.owner, owner);
            walk::walk_arrow_function_expression(self, arrow);
            self.owner = previous;
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let imported = if let Expression::Identifier(callee) = &call.callee {
            self.imports.get(callee.name.as_str()).map(String::as_str)
        } else {
            None
        };
        if imported == Some("toDisplayString") {
            self.display_calls.push(
                json!({"owner": self.owner, "arguments": call.arguments.iter()
                .map(|argument| argument.span().source_text(self.source)).collect::<Vec<_>>()}),
            );
        }
        if imported == Some("withCtx") {
            let arrow = match call
                .arguments
                .first()
                .and_then(|argument| argument.as_expression())
            {
                Some(Expression::ArrowFunctionExpression(arrow)) => Some(&**arrow),
                _ => None,
            };
            let names = arrow
                .map(|arrow| parameter_names(&arrow.params))
                .unwrap_or_default();
            let previous_arrow = std::mem::replace(
                &mut self.slot_arrow,
                arrow.map(|arrow| core::ptr::from_ref(arrow) as usize),
            );
            let owner = format!("slot:{}", self.slot_bindings.len());
            self.slot_bindings.push(names);
            let previous = std::mem::replace(&mut self.owner, owner);
            walk::walk_call_expression(self, call);
            self.owner = previous;
            self.slot_arrow = previous_arrow;
        } else {
            walk::walk_call_expression(self, call);
        }
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Expression::Identifier(object) = &member.object
            && object.name == "_ctx"
            && self.type_depth == 0
        {
            self.reads.push(
                json!({"owner": self.owner, "name": format!("_ctx.{}", member.property.name)}),
            );
            self.ctx_object_depth += 1;
            walk::walk_static_member_expression(self, member);
            self.ctx_object_depth -= 1;
        } else {
            walk::walk_static_member_expression(self, member);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        let name = identifier.name.as_str();
        self.identifiers.push(
            json!({"owner": self.owner, "name": name, "span": {"start": identifier.span.start, "end": identifier.span.end},
            "typeOnly": self.type_depth != 0}),
        );
        if self.type_depth == 0
            && !(name == "_ctx" && self.ctx_object_depth != 0)
            && !self.imports.contains_key(name)
            && !self.framework_bindings.contains(name)
            && !self.render_machinery.contains(name)
        {
            self.reads.push(json!({"owner": self.owner, "name": name}));
        }
    }

    fn visit_ts_type_annotation(&mut self, annotation: &TSTypeAnnotation<'a>) {
        self.type_depth += 1;
        walk::walk_ts_type_annotation(self, annotation);
        self.type_depth -= 1;
    }
}
