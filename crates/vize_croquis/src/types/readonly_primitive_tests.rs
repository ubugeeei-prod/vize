//! A borrowed fast path must match full-member parser truth, including refusals.
use super::TypeResolver;
use oxc_allocator::Allocator;
use oxc_ast::ast::{PropertyKey, Statement, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, cstr};
type Packet = (CompactString, bool, Option<CompactString>);
fn parser_truth(member: &str) -> Option<Packet> {
    let source = cstr!("type Props = {{ {member} }}");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let [Statement::TSTypeAliasDeclaration(alias)] = parsed.program.body.as_slice() else {
        return None;
    };
    let TSType::TSTypeLiteral(literal) = &alias.type_annotation else {
        return None;
    };
    let [TSSignature::TSPropertySignature(property)] = literal.members.as_slice() else {
        return None;
    };
    let name = match &property.key {
        PropertyKey::StaticIdentifier(value) if !property.computed => value.name.as_str(),
        PropertyKey::StringLiteral(value) => value.value.as_str(),
        _ => return None,
    };
    Some((
        CompactString::new(name),
        property.optional,
        property.type_annotation.as_ref().map(|annotation| {
            CompactString::new(annotation.type_annotation.span().source_text(&source))
        }),
    ))
}
fn assert_member(member: &str, expected_calls: u32) {
    let resolver = TypeResolver::new();
    let actual = resolver
        .extract_properties(&cstr!("{{ {member} }}"))
        .into_iter()
        .map(|prop| (prop.name, prop.optional, prop.prop_type))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        parser_truth(member).into_iter().collect::<Vec<_>>(),
        "{member}"
    );
    assert_eq!(
        resolver.literal_member_parse_calls(),
        expected_calls,
        "{member}"
    );
}
#[test]
fn primitive_keywords_trivia_and_question_tokens_match_oxc_without_member_parses() {
    for primitive in [
        "number",
        "string",
        "boolean",
        "bigint",
        "symbol",
        "object",
        "unknown",
        "never",
        "any",
        "undefined",
        "null",
        "void",
    ] {
        for prefix in [
            "readonly p?",
            "readonly p? ",
            "readonly\tp? \t",
            "readonly/* boundary */p? /* optional */",
            "readonly readonly?",
            "readonly readonlyName",
            "readonlyThing?",
            "readonly?",
        ] {
            for newline in ["\n", "\r\n"] {
                assert_member(
                    &cstr!("\n{prefix}: {primitive}\n").replace('\n', newline),
                    0,
                );
            }
        }
    }
}
#[test]
fn complex_malformed_cooked_and_unicode_members_keep_full_parser_authority() {
    for member in [
        "readonly\u{00a0}p?: number",
        "readonly\u{0085}p?: number",
        "readonly p?\u{0085}: number",
        "readonly p?: \u{0085}number",
        "readonly p?: number | string",
        "readonly p?: { label: string }",
        "readonly p?: (value: string) => number",
        "readonly p??: number",
        "readonly p?: Missing<",
        "readonly p?: number = invalid",
        "readonly readonly p?: string",
        "readonlyThing p?: number",
        r#"readonly "scope.name"?: string"#,
        r#"readonly ["\u006cabel"]? : string"#,
        "readonly café?: number",
        "readonly [key]: number",
        "readonly [sideEffect()]: number",
        r#"readonly ["a" + "b"]: string"#,
    ] {
        assert_member(member, 1);
    }
}
#[test]
fn readonly_primitive_matrix_has_complete_optional_metadata_and_zero_exotic_work() {
    for count in [32, 128, 512] {
        let mut members = CompactString::new("{");
        for index in 0..count {
            vize_carton::append!(members, "readonly p_{index}?: number;");
        }
        members.push('}');
        let resolver = TypeResolver::new();
        let props = resolver.extract_properties(&members);
        assert_eq!(props.len(), count);
        for (index, prop) in props.into_iter().enumerate() {
            assert_eq!(prop.name, cstr!("p_{index}"));
            assert_eq!(prop.prop_type.as_deref(), Some("number"));
            assert!(prop.optional);
        }
        assert_eq!(resolver.literal_member_parse_calls(), 0);
    }
}
