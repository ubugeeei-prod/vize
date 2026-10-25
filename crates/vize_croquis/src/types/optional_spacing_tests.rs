//! Property optionality comes from the question token, including surrounding trivia.

use oxc_allocator::Allocator;
use oxc_ast::ast::{PropertyKey, Statement, TSSignature, TSType};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, cstr};

use super::TypeResolver;

#[test]
fn borrowed_ascii_members_match_the_actual_parser_with_lf_and_crlf() {
    let members = "spaced? : string;\ntabbed?\t: number;\ncommented? /* question */ : boolean;\ncompact?: string;\nrequired : number;\nreadonly read? : string;\n\"scope.name\"? : string;\ncafé? : number;\n\"why?:\"? /* trivia */ : boolean;";
    for newline in ["\n", "\r\n"] {
        let members = members.replace('\n', newline);
        let source = cstr!("type Props = {{{members}}}");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(!parsed.panicked);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let [Statement::TSTypeAliasDeclaration(alias)] = parsed.program.body.as_slice() else {
            panic!("fixture must be the actual parser's type alias");
        };
        let TSType::TSTypeLiteral(literal) = &alias.type_annotation else {
            panic!("fixture must be a type literal");
        };
        let expected = literal
            .members
            .iter()
            .map(|member| {
                let TSSignature::TSPropertySignature(property) = member else {
                    panic!("property fixture");
                };
                let name = match &property.key {
                    PropertyKey::StaticIdentifier(name) => name.name.as_str(),
                    PropertyKey::StringLiteral(name) => name.value.as_str(),
                    _ => panic!("static property fixture"),
                };
                (
                    CompactString::new(name),
                    property.optional,
                    property.type_annotation.as_ref().map(|annotation| {
                        CompactString::new(annotation.type_annotation.span().source_text(&source))
                    }),
                )
            })
            .collect::<Vec<_>>();
        let resolver = TypeResolver::new();
        let actual = resolver.extract_properties(cstr!("{{{members}}}").as_str());
        assert_eq!(
            actual
                .iter()
                .map(|prop| (prop.name.clone(), prop.optional, prop.prop_type.clone()))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            resolver.literal_member_parse_calls(),
            4,
            "only readonly/quoted/Unicode members use the existing AST fallback"
        );
    }
}

#[test]
fn script_macro_required_flags_keep_ascii_spacing_optional_without_reparsing() {
    let source = "defineProps<{ spaced? : string; tabbed?\t: number; commented? /* trivia */ : boolean; compact?: string; required : number }>()";
    let parsed = crate::script_parser::parse_script_setup(source);
    assert_eq!(
        parsed
            .macros
            .props()
            .iter()
            .map(|prop| (prop.name.as_str(), prop.required))
            .collect::<Vec<_>>(),
        [
            ("spaced", false),
            ("tabbed", false),
            ("commented", false),
            ("compact", false),
            ("required", true)
        ]
    );
    assert_eq!(parsed.types.literal_member_parse_calls(), 0);
}
