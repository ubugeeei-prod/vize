extern crate std;

use super::*;
use oxc_ast::ast::{ClassElement, Expression, Program, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub(crate) fn aliases<'a>(a: &'a Allocator, text: &'a str) -> NativeForAliases<'a> {
    let parsed = Parser::new(a, text, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let program = a.alloc(parsed.program);
    let Statement::ExpressionStatement(statement) = &program.body[0] else {
        panic!("actual arrow statement")
    };
    let Expression::ArrowFunctionExpression(arrow) = &statement.expression else {
        panic!("actual arrow root")
    };
    let coordinates = a.alloc(
        JsCoordinates::checked(text, text, Span::new(0, text.len() as u32), 0, &[]).unwrap(),
    );
    let mut bindings = arrow.params.items.iter().map(|parameter| {
        JsBinding::from_retained_in(
            a,
            parameter,
            text,
            Span::new(0, text.len() as u32),
            coordinates,
        )
        .unwrap()
    });
    let value = bindings.next().unwrap();
    NativeForAliases::checked(value, bindings.next(), bindings.next()).unwrap()
}

#[test]
fn real_formals_share_exact_context_and_refuse_sparse_reversed_or_different_blocks() {
    let a = Allocator::default();
    let first = aliases(&a, "(x,key,index)=>{}");
    let roots = first.iter().collect::<alloc::vec::Vec<_>>();
    assert_eq!(
        roots
            .iter()
            .map(|binding| binding.source())
            .collect::<alloc::vec::Vec<_>>(),
        ["x", "key", "index"]
    );
    assert_eq!(
        NativeForAliases::checked(roots[0], None, Some(roots[2])).unwrap_err(),
        AliasError::MissingKey
    );
    assert_eq!(
        NativeForAliases::checked(roots[1], Some(roots[0]), None).unwrap_err(),
        AliasError::InvalidOrder
    );
    let second = aliases(&a, "(x,key,index)=>{}");
    assert_eq!(
        NativeForAliases::checked(roots[0], second.iter().nth(1), None).unwrap_err(),
        AliasError::DifferentBlock
    );
    let actual = crate::op::ForBinding {
        source: ExprRef::parse_js_in(&a, "[]", Span::new(0, 2)),
        value: BindingRef::Js(roots[0]),
        key: Some(BindingRef::Js(roots[1])),
        index: Some(BindingRef::Js(roots[2])),
    };
    assert!(actual.value.as_expression().is_none());
    assert!(core::ptr::eq(
        actual.value.as_js().unwrap().parameter(),
        roots[0].parameter()
    ));
    assert!(!core::mem::needs_drop::<Program<'_>>());
    assert_eq!(core::mem::size_of::<BindingRef<'_>>(), 16);
    assert_eq!(core::mem::size_of::<crate::op::ForBinding<'_>>(), 64);
}

#[test]
fn actual_class_parameter_properties_cannot_become_native_for_aliases() {
    let a = Allocator::default();
    let text = "class C{constructor(public x:number){}}";
    let parsed = Parser::new(&a, text, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let Statement::ClassDeclaration(class) = &parsed.program.body[0] else {
        panic!("actual class")
    };
    let ClassElement::MethodDefinition(method) = &class.body.body[0] else {
        panic!("actual constructor")
    };
    let coordinates =
        JsCoordinates::checked(text, text, Span::new(0, text.len() as u32), 0, &[]).unwrap();
    let binding = JsBinding::from_retained_in(
        &a,
        &method.value.params.items[0],
        text,
        Span::new(0, text.len() as u32),
        &coordinates,
    )
    .unwrap();
    assert_eq!(
        NativeForAliases::checked(binding, None, None).unwrap_err(),
        AliasError::InvalidContext
    );
}

#[test]
fn checked_artifact_retains_real_binding_and_dump_refusal_preserves_authored_span() {
    use crate::artifact::{Artifact, ArtifactError, ArtifactParts};
    use crate::dump::{NativeDumpError, Page};
    use crate::op::{ForBinding, ForOp, Op, Region};
    use crate::scope::{ScopeFacts, ScopeTag};
    use vize_l0::{Box, Vec, id::NodeId, side_table::SideTable};

    let a = Allocator::default();
    let file = "(x)=>{[]}";
    let native = aliases(&a, file);
    let value = native.iter().next().unwrap();
    let root = value.parameter();
    let make_parts = |source| {
        let binding = ForBinding {
            source: ExprRef::parse_js_in(&a, "[]", Span::new(6, 8)),
            value: BindingRef::Js(value),
            key: None,
            index: None,
        };
        let mut ops = Vec::new_in(&&a);
        ops.push(Op::For(Box::new_in(
            ForOp {
                binding,
                region: Region {
                    ops: Vec::new_in(&&a),
                },
                span: Span::new(0, file.len() as u32),
            },
            &&a,
        )));
        let mut scopes = SideTable::new();
        scopes.insert(
            NodeId::FIRST,
            ScopeFacts {
                tag: ScopeTag::from_index(0),
                bindings: alloc::vec::Vec::new(),
            },
        );
        ArtifactParts {
            source,
            root: Region { ops },
            scopes,
            provenance: alloc::vec::Vec::new(),
        }
    };
    let artifact = Artifact::try_new(make_parts(file)).unwrap();
    let mut seen = 0;
    artifact
        .visit_nodes(&mut |id, node| {
            assert_eq!(id, NodeId::FIRST);
            node.for_each_binding(&mut |binding| {
                assert!(binding.as_expression().is_none());
                assert!(core::ptr::eq(binding.as_js().unwrap().parameter(), root));
                seen += 1;
            });
        })
        .unwrap();
    assert_eq!(seen, 1);
    let error = Page::of(&artifact.root().ops).unwrap_err();
    assert_eq!(
        error,
        NativeDumpError::JsBindingUnsupported {
            span: Span::new(1, 2)
        }
    );
    assert_eq!(
        vize_l0::cstr!("{error}"),
        "native binding dump unsupported at bytes 1..2"
    );
    assert_eq!(
        Artifact::try_new(make_parts("(y)=>{[]}"))
            .unwrap_err()
            .error,
        ArtifactError::MismatchedBindingSource {
            node: NodeId::FIRST,
            span: Span::new(1, 2)
        }
    );
}
