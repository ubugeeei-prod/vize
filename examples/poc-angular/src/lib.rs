//! Proof of concept: an Angular component compiled through the level crates.
//!
//! The component module is the container: its `@Component` decorator names
//! the selector and holds the inline template. The L1 markup parser reads the
//! template in place, [`dialect`] lowers Angular's bindings and control flow
//! blocks to neutral L2 ops, the shared L2 to L3 lowering separates static
//! from dynamic, and [`target`] writes Ivy instructions: static ops in the
//! creation pass, dynamic ops in the update pass. The decorator becomes the
//! class's static `ɵfac` and `ɵcmp` definitions.
//! Not a supported product surface (2026-11-03 scope decision).

mod dialect;
mod target;

use oxc_ast::ast::{
    Class, Declaration, Expression, ObjectPropertyKind, Statement, TemplateLiteral,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use poc_support::{Error, View, js_string};
use vize_l0::{Allocator, Span, String, append, cstr};
use vize_l4::write::{NoLinks, Writer};

/// What the `@Component` decorator of one class says.
struct Decorated<'a> {
    class: &'a str,
    selector: &'a str,
    template: Span,
    /// The decorator with its trailing line break, removed from the output.
    decorator: Span,
    /// Where the template functions go: before the decorated statement.
    statement: u32,
    /// The class body's closing brace.
    body_end: u32,
}

/// Compile one Angular component module with an inline template.
pub fn compile(source: &str) -> Result<String, Error> {
    let allocator = Allocator::default();
    let component = decorated(source)?;
    let template = source
        .get(component.template.start as usize..component.template.end as usize)
        .unwrap_or_default();
    let (tree, errors) = vize_l1::markup::parse(&allocator, template);
    if let Some(error) = errors.first() {
        let at = component.template.start + error.offset;
        return Err(Error::new(
            cstr!("markup: {:?}", error.code),
            Span::new(at, at),
        ));
    }
    let region = dialect::Lowering::new(&allocator, source).region(tree.children.iter())?;
    let lowered = poc_support::lower(&allocator, &region)?;
    let mut target = target::Component::new(component.class);
    let (template_fn, decls, vars) = target.root(View::new(&lowered.program))?;

    let class = component.class;
    let mut definition = cstr!(
        "\n  static ɵfac = function {class}_Factory(__ngFactoryType__) {{\n    return new (__ngFactoryType__ || {class})();\n  }};\n"
    );
    append!(
        definition,
        "  static ɵcmp = /*@__PURE__*/ i0.ɵɵdefineComponent({{\n    type: {class},\n    selectors: [[{}]],\n    decls: {decls},\n    vars: {vars},\n",
        js_string(component.selector)
    );
    if !target.consts.is_empty() {
        append!(definition, "    consts: [{}],\n", target.consts.join(", "));
    }
    append!(
        definition,
        "    template: {},\n    encapsulation: 2,\n  }});\n",
        poc_support::indent(&template_fn, "    ")
    );

    let slice = |start: u32, end: u32| source.get(start as usize..end as usize).unwrap_or_default();
    let end = u32::try_from(source.len()).unwrap_or(u32::MAX);
    let mut out = Writer::<NoLinks>::with_capacity(source.len() * 3);
    out.push("import * as i0 from \"@angular/core\";\n");
    out.push(slice(0, component.statement));
    for function in &target.functions {
        out.push(&cstr!("{function}\n\n"));
    }
    out.push(slice(component.statement, component.decorator.start));
    out.push(slice(component.decorator.end, component.body_end));
    out.push(&definition);
    out.push(slice(component.body_end, end));
    Ok(out.finish().text)
}

/// Find the exported class decorated with `@Component({ selector, template })`.
fn decorated(source: &str) -> Result<Decorated<'_>, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if let Some(error) = parsed.diagnostics.iter().next() {
        return Err(Error::new(cstr!("{error}"), Span::new(0, 0)));
    }
    for statement in &parsed.program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        let Some(Declaration::ClassDeclaration(class)) = &export.declaration else {
            continue;
        };
        if let Some(found) = component(source, statement.span().start, class)? {
            return Ok(found);
        }
    }
    Err(Error::new("no exported @Component class", Span::new(0, 0)))
}

fn component<'s>(
    source: &'s str,
    statement: u32,
    class: &Class<'_>,
) -> Result<Option<Decorated<'s>>, Error> {
    let [decorator] = class.decorators.as_slice() else {
        return Ok(None);
    };
    let Expression::CallExpression(call) = &decorator.expression else {
        return Ok(None);
    };
    let (Expression::Identifier(callee), Some(Expression::ObjectExpression(options))) = (
        &call.callee,
        call.arguments
            .first()
            .and_then(|argument| argument.as_expression()),
    ) else {
        return Ok(None);
    };
    if callee.name.as_str() != "Component" {
        return Ok(None);
    }
    let fail =
        |message: &str| Error::new(message, Span::new(decorator.span.start, decorator.span.end));
    let mut selector = None;
    let mut template = None;
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return Err(fail("unsupported @Component option"));
        };
        match (property.key.static_name().as_deref(), &property.value) {
            (Some("selector"), Expression::StringLiteral(value)) => {
                selector = source.get(value.span.start as usize + 1..value.span.end as usize - 1);
            }
            (Some("template"), Expression::TemplateLiteral(literal)) => {
                template = Some(
                    template_content(literal)
                        .ok_or_else(|| fail("the template must be a plain template literal"))?,
                );
            }
            _ => return Err(fail("unsupported @Component option")),
        }
    }
    let name = class
        .id
        .as_ref()
        .map(|id| id.span)
        .and_then(|span| source.get(span.start as usize..span.end as usize));
    let (Some(class_name), Some(selector), Some(template)) = (name, selector, template) else {
        return Err(fail(
            "@Component needs a class name, a selector and a template",
        ));
    };
    let mut decorator_end = decorator.span.end;
    if source.as_bytes().get(decorator_end as usize) == Some(&b'\n') {
        decorator_end += 1;
    }
    Ok(Some(Decorated {
        class: class_name,
        selector,
        template,
        decorator: Span::new(decorator.span.start, decorator_end),
        statement: statement.min(decorator.span.start),
        body_end: class.body.span.end - 1,
    }))
}

fn template_content(literal: &TemplateLiteral<'_>) -> Option<Span> {
    let [quasi] = literal.quasis.as_slice() else {
        return None;
    };
    literal
        .expressions
        .is_empty()
        .then(|| Span::new(quasi.span.start, quasi.span.end))
}

#[cfg(test)]
mod tests {
    use super::compile;

    const COUNTER: &str = include_str!("../input/counter.component.ts");
    const SNAPSHOT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/__snapshots__/counter.component.js"
    );

    #[test]
    fn counter_compiles_to_an_ivy_component() {
        let output = compile(COUNTER);
        assert!(output.is_ok(), "{output:?}");
        let output = output.unwrap_or_default();
        assert_eq!(poc_support::check_module(&output), Ok(()));
        assert_eq!(poc_support::snapshot(SNAPSHOT, &output), Ok(()));
    }

    #[test]
    fn structural_directives_are_refused_with_their_span() {
        let source =
            "@Component({ selector: \"x\", template: `<p *ngIf=\"a\"></p>` })\nexport class X {}\n";
        let refused = compile(source);
        assert_eq!(refused.map_err(|error| error.span.start), Err(42));
    }
}
