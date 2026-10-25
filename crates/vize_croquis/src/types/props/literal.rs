//! Demand the parser only for member names outside the ASCII fast path.
//! Cooked string and numeric keys follow JavaScript property-name semantics;
//! computed expressions are never evaluated or converted into invented names.

use oxc_allocator::Allocator;
use oxc_ast::ast::{PropertyKey, Statement, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::number::ToJsString;
use vize_carton::{CompactString, cstr};

use super::TypeProperty;

pub(super) fn parse(segment: &str) -> Option<TypeProperty> {
    let source = cstr!("type __VizeMember = {{ {segment} }}");
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
        PropertyKey::StaticIdentifier(identifier) if !property.computed => {
            CompactString::new(identifier.name.as_str())
        }
        PropertyKey::StringLiteral(value) => CompactString::new(value.value.as_str()),
        PropertyKey::NumericLiteral(value) => CompactString::new(value.value.to_js_string()),
        _ => return None,
    };
    Some(TypeProperty {
        name,
        prop_type: property.type_annotation.as_ref().map(|annotation| {
            CompactString::new(annotation.type_annotation.span().source_text(&source))
        }),
        optional: property.optional,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn cooked_names_and_exact_type_text_are_retained() {
        for (source, name, optional, expected_type) in [
            (r#""scope.name"?: string"#, "scope.name", true, "string"),
            (
                r#"['a:b']?: { value: "x:y" }"#,
                "a:b",
                true,
                r#"{ value: "x:y" }"#,
            ),
            (r#""\u006cabel"?: string"#, "label", true, "string"),
            (r#"['quote\'key']?: number"#, "quote'key", true, "number"),
            (r#"""?: boolean"#, "", true, "boolean"),
            ("café?: number", "café", true, "number"),
            ("readonly 状態: string", "状態", false, "string"),
            ("0x10?: string", "16", true, "string"),
        ] {
            let prop = parse(source).expect(source);
            assert_eq!(prop.name.as_str(), name);
            assert_eq!(prop.optional, optional);
            assert_eq!(prop.prop_type.as_deref(), Some(expected_type));
        }
    }

    #[test]
    fn dynamic_keys_malformed_members_and_extra_declarations_are_refused() {
        for source in [
            "[sideEffect()]: string",
            "[key]: string",
            "[`prefix${key}`]: string",
            "[Symbol.iterator]: string",
            "[name in Keys]: string",
            "[key: string]: string",
            "'unterminated: number",
            "'a': number; 'b': boolean",
            "['a' + 'b']: string",
        ] {
            assert!(parse(source).is_none(), "{source}");
        }
    }
}
