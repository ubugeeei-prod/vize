use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{Namespace, ReferenceTarget};
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l2::resolution::{SyntaxKind as Syntax, Usage};
use vize_l3::jsx::{JsxDecisionKind as Kind, JsxIssueKind as Issue, build_jsx_decisions};

fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
    profile: SourceType,
) -> Result<JsxFile<'a>, &'static str> {
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let original = Parser::new(arena, source, profile).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, original, block, 0).map_err(|_| "original producer")?;
    producer.walk().map_err(|_| "sole walk")?;
    producer.finish().map_err(|_| "completed File")
}

#[test]
fn scalar_attributes_keep_actual_values_and_read_binding_after_owner_movement()
-> Result<(), &'static str> {
    let source = "function render(message) { return <div id={message} class={'one\\t two'} title={'A\\u0026amp;B'} disabled={true} data-count={0x10} aria-hidden={false} data-null={null}/>; }";
    for profile in [SourceType::jsx(), SourceType::tsx().with_module(true)] {
        let arena = Allocator::default();
        let owner = lower(&arena, source, profile)?;
        let body = owner
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr();
        let mut moved = vec![build_jsx_decisions(owner).map_err(|_| "view")?];
        moved.reserve(256);
        let analysis = moved.pop().ok_or("moved")?;
        assert_eq!(
            analysis
                .owner()
                .observation()
                .admitted()
                .ok_or("original")?
                .program()
                .body
                .as_ptr(),
            body
        );
        assert!(core::ptr::eq(
            analysis.owner().file().artifact().source(),
            source
        ));
        let mut names = Vec::new();
        for row in analysis.decisions() {
            assert!(core::ptr::eq(row.node().owner(), analysis.owner()));
            assert_eq!(
                analysis.decision_for(row.node()).ok_or("own query")?.kind(),
                row.kind()
            );
            if let Kind::ExpressionAttribute { name } = row.kind() {
                names.push(name);
                let mut children = row.node().children();
                assert_eq!(
                    children.next().and_then(|node| node.kind()),
                    Some(Syntax::AttributeName(name))
                );
                let value = children.next().ok_or("actual container")?;
                assert!(children.next().is_none());
                assert_eq!(value.kind(), Some(Syntax::Container));
                assert_eq!(
                    analysis.decision_for(value).ok_or("own value")?.kind(),
                    Kind::ExpressionContainer
                );
                assert!(row.node().same_owner(value));
            }
        }
        assert_eq!(
            names,
            [
                "id",
                "class",
                "title",
                "disabled",
                "data-count",
                "aria-hidden",
                "data-null"
            ]
        );
        let literals: Vec<_> = analysis
            .decisions()
            .filter_map(|row| match row.kind() {
                kind @ (Kind::String(_) | Kind::Number(_) | Kind::Boolean(_) | Kind::Null) => {
                    Some(kind)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            literals,
            [
                Kind::String("one\t two"),
                Kind::String("A&amp;B"),
                Kind::Boolean(true),
                Kind::Number(16.0_f64.to_bits()),
                Kind::Boolean(false),
                Kind::Null
            ]
        );
        let read = analysis
            .decisions()
            .find(|row| matches!(row.kind(), Kind::Read { .. }))
            .ok_or("read")?;
        let Kind::Read { name, binding } = read.kind() else {
            return Err("read role");
        };
        let [reference] = read.node().references().ok_or("references")? else {
            return Err("one read");
        };
        assert_eq!(name, "message");
        assert_eq!(reference.usage, Usage::Read);
        assert_eq!(reference.target, ReferenceTarget::Resolved(binding));
        assert_eq!(
            analysis
                .owner()
                .file()
                .lookup(reference.scope, name, Namespace::Value)
                .ok_or("same File binding")?
                .id(),
            binding
        );
        assert_ne!(reference.scope, analysis.owner().file().units()[0].scope);
    }
    Ok(())
}

#[test]
fn exact_former_attribute_refusals_and_escaped_shadow_reads_have_original_custody()
-> Result<(), &'static str> {
    for (source, name, spelling) in [
        ("const x = 1; const view = <div id={x}/>;", "x", "x"),
        ("function render(x) { return <div id={x}/>; }", "x", "x"),
        (
            "const value = 7; function render(value) { return <div title={value}/>; }",
            "value",
            "value",
        ),
        (
            "function render(\\u6d88\\u606f) { return <div title={\\u6d88\\u606f}/>; }",
            "消息",
            "\\u6d88\\u606f",
        ),
    ] {
        let arena = Allocator::default();
        let owner = lower(&arena, source, SourceType::jsx())?;
        let analysis = build_jsx_decisions(owner).map_err(|_| "owned view")?;
        assert_eq!(
            analysis
                .decisions()
                .filter(|row| matches!(row.kind(), Kind::ExpressionAttribute { .. }))
                .count(),
            1
        );
        let read = analysis
            .decisions()
            .find(|row| matches!(row.kind(), Kind::Read { .. }))
            .ok_or("read")?;
        let Kind::Read {
            name: actual,
            binding,
        } = read.kind()
        else {
            return Err("read role");
        };
        assert_eq!(actual, name);
        assert_eq!(read.node().source(), Some(spelling));
        let [reference] = read.node().references().ok_or("reference")? else {
            return Err("one read");
        };
        assert_eq!(reference.target, ReferenceTarget::Resolved(binding));
        assert_eq!(
            analysis
                .owner()
                .file()
                .lookup(reference.scope, name, Namespace::Value)
                .ok_or("genuine scope")?
                .id(),
            binding
        );
        if name == "value" {
            assert_ne!(
                analysis
                    .owner()
                    .file()
                    .lookup(
                        analysis.owner().file().units()[0].scope,
                        name,
                        Namespace::Value,
                    )
                    .ok_or("original outer binding")?
                    .id(),
                binding,
            );
        }
        let foreign = build_jsx_decisions(lower(&arena, source, SourceType::jsx())?)
            .map_err(|_| "second owner")?;
        assert!(foreign.decision_for(read.node()).is_none());
    }
    Ok(())
}

#[test]
fn complex_or_non_whitelisted_attributes_refuse_the_complete_view() -> Result<(), &'static str> {
    for (attribute, expected) in [
        ("id={x + 1}", Issue::Attribute),
        ("id={-1}", Issue::Attribute),
        ("id={(x)}", Issue::Attribute),
        ("id={x.value}", Issue::Attribute),
        ("id={x()}", Issue::Attribute),
        ("id={{value: x}}", Issue::Attribute),
        ("id={[x]}", Issue::Attribute),
        ("id={`value`}", Issue::Attribute),
        ("style={x}", Issue::Attribute),
        ("onClick={x}", Issue::Attribute),
        ("v-show={x}", Issue::DirectiveOrSlot),
        ("v-slots={x}", Issue::DirectiveOrSlot),
        ("{...x}", Issue::SpreadAttribute),
    ] {
        let arena = Allocator::default();
        let source = format!("function render(x) {{ return <div {attribute}/>; }}");
        let owner = lower(&arena, &source, SourceType::jsx())?;
        let body = owner
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr();
        let rejected = build_jsx_decisions(owner).err().ok_or("whole refusal")?;
        assert_eq!(
            rejected
                .owner()
                .observation()
                .admitted()
                .ok_or("original")?
                .program()
                .body
                .as_ptr(),
            body
        );
        assert!(rejected.owner().file().is_complete());
        assert!(
            rejected.issues().iter().any(|issue| issue.kind == expected),
            "{attribute}"
        );
    }
    Ok(())
}

#[test]
fn unbound_or_empty_attribute_values_never_reach_upper_admission() {
    let arena = Allocator::default();
    for source in [
        "function render() { return <div title={missing}/>; }",
        "function render() { return <div title={}/>; }",
        "function render() { return <div title={/*empty*/}/>; }",
    ] {
        assert!(lower(&arena, source, SourceType::jsx()).is_err());
    }
}
