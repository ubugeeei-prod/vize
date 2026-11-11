use super::{Combinator, Operator, Selector, Simple, Value, parse};

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
    ] {
        assert!(parse(source).is_some(), "{source}");
    }
    for source in [
        "",
        "A >",
        "A:nth-of-type(1)",
        "A:has(> B)",
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
