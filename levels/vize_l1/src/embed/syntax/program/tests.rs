use oxc_ast::ast::{Declaration, Expression, Statement};
use vize_l0::{Allocator, Span};

use super::{EmbedSource, Lang, NativeSyntax, ProgramGoal, ProgramOptions, parse_program_once};
use crate::embed::syntax::{EmbedHole, parse_once};
use crate::embed::{Embed, Grammar, Shape, prepare_attribute_value};

const JS: &str = "/* leading */\nimport { createApp } from 'vue';\nconst data = [1, 2, 3, 4];\nfunction total(items) { let result = 0; for (const item of items) result += item; return result; }\nexport const app = createApp({ data: () => ({ total: total(data) }) });\n// trailing";
const TS: &str = "/* typed */\ninterface Model { id: number; label?: string }\ntype State<T> = { value: T; readonly ready: boolean };\nexport function pick<T extends Model>(input: T[]): State<T | undefined> { return { value: input[0], ready: true }; }";
const JSX: &str = "/* jsx */\nimport { h } from 'vue';\nconst items = ['one', 'two'];\nexport const view = <section aria-label=\"x\">{items.map((label, index) => <button key={index}>{label}</button>)}</section>;";
const TSX: &str = "/* tsx */\ntype Props = { title: string; values: number[] };\nexport const Panel = ({ title, values }: Props) => (<main><h1>{title}</h1><ul>{values.map((value: number) => <li key={value}>{value}</li>)}</ul></main>);";

fn source(text: &str) -> EmbedSource<'_> {
    EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap()
}

fn parse<'a>(allocator: &'a Allocator, text: &'a str, lang: Lang, jsx: bool) -> NativeSyntax<'a> {
    parse_program_once(
        allocator,
        source(text),
        ProgramOptions {
            lang,
            jsx,
            goal: ProgramGoal::Module,
        },
    )
}

#[test]
fn ordinary_js_program_exceeds_scaffold_and_retains_real_statement_children() {
    let allocator = Allocator::default();
    assert!(!super::super::admission::allows_small_input(JS));
    let tree = parse(&allocator, JS, Lang::Js, false);
    assert_eq!(tree.hole(), None);
    let program = tree.program().unwrap();
    assert_eq!(program.source_text, JS);
    assert_eq!(program.body.len(), 4);
    assert!(matches!(program.body[0], Statement::ImportDeclaration(_)));
    assert!(matches!(program.body[1], Statement::VariableDeclaration(_)));
    let Statement::FunctionDeclaration(function) = &program.body[2] else {
        panic!("function")
    };
    let body = function.body.as_ref().unwrap();
    assert_eq!(body.statements.len(), 3);
    assert!(matches!(body.statements[1], Statement::ForOfStatement(_)));
    let Statement::ExportNamedDeclaration(export) = &program.body[3] else {
        panic!("export")
    };
    let Some(Declaration::VariableDeclaration(variable)) = &export.declaration else {
        panic!("variable")
    };
    assert!(matches!(
        variable.declarations[0].init,
        Some(Expression::CallExpression(_))
    ));
    let mut comments = tree.comments();
    assert_eq!(comments.next().unwrap().text().unwrap(), "/* leading */");
    assert_eq!(comments.next().unwrap().text().unwrap(), "// trailing");
    assert!(comments.next().is_none());
    assert_eq!(tree.authored_span(function.span), Ok(Span::new(74, 172)));
    assert_eq!(tree.diagnostics().count(), 0);
}

#[test]
fn ts_program_retains_types_generics_and_return_annotations() {
    let allocator = Allocator::default();
    assert!(!super::super::admission::allows_small_input(TS));
    let tree = parse(&allocator, TS, Lang::Ts, false);
    assert_eq!(tree.hole(), None);
    let program = tree.program().unwrap();
    assert_eq!(program.body.len(), 3);
    assert!(matches!(
        program.body[0],
        Statement::TSInterfaceDeclaration(_)
    ));
    assert!(matches!(
        program.body[1],
        Statement::TSTypeAliasDeclaration(_)
    ));
    let Statement::ExportNamedDeclaration(export) = &program.body[2] else {
        panic!("export")
    };
    let Some(Declaration::FunctionDeclaration(function)) = &export.declaration else {
        panic!("function")
    };
    assert!(function.type_parameters.is_some());
    assert!(function.return_type.is_some());
    assert!(function.params.items[0].type_annotation.is_some());
    assert_eq!(
        tree.source_type(),
        ProgramOptions::module(Lang::Ts).source_type()
    );
    let js = parse(&allocator, TS, Lang::Js, false);
    assert_eq!(js.hole(), Some(EmbedHole::Syntax));
    assert!(js.diagnostics().next().is_some());
}

#[test]
fn jsx_program_has_actual_element_and_expression_container_children() {
    let allocator = Allocator::default();
    let tree = parse(&allocator, JSX, Lang::Js, true);
    assert_eq!(tree.hole(), None);
    let program = tree.program().unwrap();
    assert_eq!(program.body.len(), 3);
    let Statement::ExportNamedDeclaration(export) = &program.body[2] else {
        panic!("export")
    };
    let Some(Declaration::VariableDeclaration(variable)) = &export.declaration else {
        panic!("variable")
    };
    let Some(Expression::JSXElement(element)) = &variable.declarations[0].init else {
        panic!("jsx")
    };
    assert_eq!(element.opening_element.attributes.len(), 1);
    assert_eq!(element.children.len(), 1);
    assert!(matches!(
        element.children[0],
        oxc_ast::ast::JSXChild::ExpressionContainer(_)
    ));
    assert_eq!(
        tree.source_type(),
        ProgramOptions {
            lang: Lang::Js,
            jsx: true,
            goal: ProgramGoal::Module
        }
        .source_type()
    );
    assert_eq!(
        parse(&allocator, JSX, Lang::Js, false).hole(),
        Some(EmbedHole::Syntax)
    );
}

#[test]
fn tsx_program_retains_typed_arrow_and_nested_jsx() {
    let allocator = Allocator::default();
    let tree = parse(&allocator, TSX, Lang::Ts, true);
    assert_eq!(tree.hole(), None);
    let program = tree.program().unwrap();
    assert_eq!(program.body.len(), 2);
    assert!(matches!(
        program.body[0],
        Statement::TSTypeAliasDeclaration(_)
    ));
    let Statement::ExportNamedDeclaration(export) = &program.body[1] else {
        panic!("export")
    };
    let Some(Declaration::VariableDeclaration(variable)) = &export.declaration else {
        panic!("variable")
    };
    let Some(Expression::ArrowFunctionExpression(arrow)) = &variable.declarations[0].init else {
        panic!("arrow")
    };
    assert!(arrow.params.items[0].type_annotation.is_some());
    let Statement::ExpressionStatement(statement) = &arrow.body.statements[0] else {
        panic!("body")
    };
    let Expression::ParenthesizedExpression(parentheses) = &statement.expression else {
        panic!("authored parentheses")
    };
    let Expression::JSXElement(element) = &parentheses.expression else {
        panic!("jsx")
    };
    assert_eq!(element.children.len(), 2);
    assert_eq!(
        parse(&allocator, TSX, Lang::Ts, false).hole(),
        Some(EmbedHole::Syntax)
    );
    assert_eq!(
        parse(&allocator, TSX, Lang::Js, true).hole(),
        Some(EmbedHole::Syntax)
    );
}

#[test]
fn explicit_goals_and_existing_program_dispatch_share_real_profile() {
    let allocator = Allocator::default();
    let text = "let state = 0; function update(value) { state += value; return state; } update(2);";
    let options = ProgramOptions {
        lang: Lang::Js,
        jsx: false,
        goal: ProgramGoal::Script,
    };
    let script = parse_program_once(&allocator, source(text), options);
    assert_eq!(script.hole(), None);
    assert!(script.source_type().is_script());
    assert_eq!(script.program().unwrap().source_type, options.source_type());
    let module = parse_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::Program,
                lang: Lang::Js,
            },
            source: source(text),
        },
    );
    assert_eq!(module.hole(), None);
    assert!(module.source_type().is_module());
    assert_eq!(
        module.program().unwrap().body.len(),
        script.program().unwrap().body.len()
    );
}

#[test]
fn decoded_program_source_and_entity_edit_boundaries_are_retained() {
    let allocator = Allocator::default();
    let authored = "const value = &quot;α&quot;; // keep";
    let prepared =
        prepare_attribute_value(&allocator, authored, Span::new(0, authored.len() as u32)).unwrap();
    let tree = parse_program_once(&allocator, prepared, ProgramOptions::module(Lang::Js));
    assert_eq!(tree.hole(), None);
    assert_eq!(
        tree.program().unwrap().source_text,
        "const value = \"α\"; // keep"
    );
    assert_eq!(tree.source().span(), Span::new(0, authored.len() as u32));
    assert_eq!(tree.comments().next().unwrap().text().unwrap(), "// keep");
    assert_eq!(
        tree.authored_span(oxc_span::Span::new(14, 15)),
        Ok(Span::new(14, 20))
    );
}

#[test]
fn malformed_and_every_utf8_cut_keep_source_profile_and_owned_observations() {
    let options = ProgramOptions {
        lang: Lang::Ts,
        jsx: true,
        goal: ProgramGoal::Script,
    };
    let text = "/* α */ const value: number = ;\r\n// tail β";
    for cut in (0..=text.len()).filter(|&cut| text.is_char_boundary(cut)) {
        let allocator = Allocator::default();
        let input = text.get(..cut).unwrap();
        let tree = parse_program_once(&allocator, source(input), options);
        assert_eq!(tree.source().text(), input);
        assert_eq!(tree.source_type(), options.source_type());
        for diagnostic in tree.diagnostics() {
            assert!(!diagnostic.message().is_empty());
            for label in diagnostic.labels() {
                let span = label.decoded_span().unwrap();
                assert!(input.is_char_boundary(span.start as usize));
                assert!(input.is_char_boundary(span.end as usize));
                assert!(span.end as usize <= input.len());
            }
        }
        if tree.hole().is_some() {
            assert!(tree.program().is_none());
        }
        assert!(core::mem::needs_drop::<NativeSyntax<'_>>());
        drop(tree);
    }
}

#[test]
fn pure_comment_expression_reproducer_is_replayed_as_a_real_program() {
    let allocator = Allocator::default();
    let input = "f<>/((\nd=//#__PURE__0";
    let tree = parse(&allocator, input, Lang::Ts, false);
    assert_eq!(tree.source().text(), input);
    assert_eq!(tree.hole(), Some(EmbedHole::Syntax));
    assert!(tree.diagnostics().next().is_some());
}

#[test]
fn pure_comment_recovery_and_rewind_keep_the_actual_annotation_owner() {
    use oxc_ast::ast::CommentContent;

    for input in [
        "/*#__PURE__*/:",
        "/* @__PURE__ */:;",
        "(/*#__PURE__*/\n/* other */\nf",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let tree = parse(&allocator, input, lang, false);
            assert_eq!(tree.source().text(), input);
            assert_eq!(tree.hole(), Some(EmbedHole::Syntax));
            assert!(tree.diagnostics().next().is_some());
            // Inspect the retained recovery observations without exposing a
            // successful AST view to consumers of a syntax hole.
            let recovered = tree.observation.as_ref().unwrap().comments();
            assert_eq!(recovered[0].content, CommentContent::PureNotApplied);
            if let Some(ordinary) = recovered.get(1) {
                assert_eq!(ordinary.content, CommentContent::None);
            }
            assert!(tree.program().is_none());
            drop(tree);
        }
    }
}
