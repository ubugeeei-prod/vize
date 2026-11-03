use super::{Allocator, Parser, SourceType, finish};

#[test]
fn required_after_optional_retains_the_original_parser_error_without_a_file() {
    let arena = Allocator::default();
    for (profile, source) in [
        (
            SourceType::ts().with_module(true),
            "/* original */function f(value?:number,later:number){return later;}",
        ),
        (
            SourceType::tsx().with_module(true),
            "/* original 🌸 */\r\nfunction f(value?:number,later:number){return later;}const view=<></>;",
        ),
    ] {
        let observed = Parser::new(&arena, source, profile).parse_observed();
        assert!(!observed.panicked());
        assert!(!observed.is_flow_language());
        assert!(observed.admitted().is_none());
        assert_eq!(observed.comments().len(), 1);
        let diagnostics = observed.diagnostics();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        let original = diagnostics.as_ptr();
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code.scope.as_deref(), Some("TS"));
        assert_eq!(diagnostic.code.number.as_deref(), Some("1016"));
        assert_eq!(
            diagnostic.message.as_ref(),
            "A required parameter cannot follow an optional parameter."
        );
        assert_eq!(format!("{:?}", diagnostic.severity), "Error");
        assert!(diagnostic.help.is_none());
        assert!(diagnostic.note.is_none());
        assert!(diagnostic.url.is_none());
        assert_eq!(diagnostic.labels.len(), 1);
        let label = &diagnostic.labels[0];
        assert_eq!(
            label.offset() as usize,
            source.find("later:number").unwrap()
        );
        assert_eq!(label.len() as usize, "later:number".len());
        assert_eq!(
            finish(&arena, &observed).err(),
            Some("original Program admission")
        );
        assert_eq!(observed.diagnostics().as_ptr(), original);
        assert_eq!(observed.diagnostics().len(), 1);
        assert!(observed.admitted().is_none());
    }
}

#[test]
fn illegal_optional_default_and_rest_fields_remain_original_parser_refusals() {
    let arena = Allocator::default();
    for source in [
        "function f(value?:number=1){return value;}",
        "function f(...value?:number[]){return value;}",
    ] {
        let observed =
            Parser::new(&arena, source, SourceType::ts().with_module(true)).parse_observed();
        assert!(observed.admitted().is_none(), "{source}");
        assert!(observed.diagnostics().has_errors());
        let diagnostics = observed.diagnostics().clone();
        assert_eq!(
            finish(&arena, &observed).err(),
            Some("original Program admission")
        );
        assert_eq!(observed.diagnostics(), &diagnostics);
    }
}
