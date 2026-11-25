use super::{Combinator, HasArgument, Operator, Selector, Simple, Value, parse};

#[test]
fn parses_the_supported_subset() {
    for source in [
        "TSAsExpression",
        "*",
        "LogicalExpression[operator='??'] > MemberExpression.left:matches([object.name='ts'], [object.property.name=\"ts\"])",
        "VElement[name='slot'] > VText[value=/\\S/i]",
        "A ~ B + C D, E",
        "BinaryExpression[left.value>=1]:not(:has(Identifier)):first-child",
        "ArrayExpression > :nth-child(2):nth-last-child(1)",
        ":expression:statement:declaration:function:pattern",
        "[name=foo][value!=1.5][x<2][y<=3][z>4]",
        "VElement:has(> VStartTag > VAttribute[key.name='flex-wrap']) > VElement",
        "A:has( >B ~ C, + D E, F > G, ~ H)",
        "A:not(:has(> B))",
    ] {
        assert!(parse(source).is_some(), "{source}");
    }
    for source in [
        "",
        "A >",
        "A:nth-of-type(1)",
        "A:has(>)",
        "A:has(> > B)",
        "A:has(> B,)",
        "A:matches(> B)",
        "A:not(> B)",
        "> A",
        "A[x=type(string)]",
        "A[x='unterminated]",
        "A, ",
        "A:matches(B",
    ] {
        assert!(parse(source).is_none(), "{source}");
    }
}

#[test]
fn builds_relations_left_to_right() {
    let parsed = parse("A[b='c'] > D.e");
    let Some([Selector::Relation(left, Combinator::Child, right)]) = parsed.as_deref() else {
        unreachable!("one child relation: {parsed:?}");
    };
    assert!(
        matches!(left.as_ref(), Selector::Compound(parts) if matches!(parts.as_slice(),
            [Simple::Type(_), Simple::Attribute(path, Some((Operator::Equal, Value::Literal(value))))]
                if path.len() == 1 && value == "c"))
    );
    assert!(matches!(right.as_ref(), Selector::Compound(parts) if parts.len() == 2));
}

#[test]
fn anchors_relative_has_arguments_at_the_subject() {
    let parsed = parse("A:has(> B ~ C D, E > F)");
    let Some([Selector::Compound(parts)]) = parsed.as_deref() else {
        unreachable!("one compound: {parsed:?}");
    };
    let [Simple::Type(_), Simple::Has(arguments)] = parts.as_slice() else {
        unreachable!("a type and :has(): {parts:?}");
    };
    let [
        HasArgument::Relative(steps),
        HasArgument::Descendant(descendant),
    ] = arguments.as_slice()
    else {
        unreachable!("a relative and a descendant argument: {arguments:?}");
    };
    let combinators = steps
        .iter()
        .map(|(combinator, _)| *combinator)
        .collect::<Vec<_>>();
    assert_eq!(
        combinators,
        [
            Combinator::Child,
            Combinator::Sibling,
            Combinator::Descendant
        ]
    );
    assert!(steps.iter().all(|(_, parts)| parts.len() == 1));
    assert!(matches!(
        descendant,
        Selector::Relation(_, Combinator::Child, _)
    ));
}
