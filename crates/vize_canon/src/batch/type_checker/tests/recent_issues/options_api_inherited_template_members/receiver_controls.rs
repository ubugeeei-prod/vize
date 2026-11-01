//! Negative callable receiver controls are separate from the original five errors.
use super::*;

pub(super) fn shared_with_receiver_methods() -> String {
    let overloads = "function overloaded(this: { currentDevice: string }, value: string): string;\nfunction overloaded(this: { currentDevice: string }, value: number): number;\nfunction overloaded(this: { currentDevice: string }, value: string | number): string | number { return value; }\n";
    FILES[1].1.replace("export const shared", &cstr!("{overloads}export const shared"))
        .replace("methods: {", "methods: {\n    overloaded,\n    identity<T>(this: { currentDevice: string }, value: T): T { return value; },")
        .replace("greet(name: string)", "greet(this: { currentDevice: string }, name: string)").into()
}

fn portable_negative(message: &str) -> String {
    const MISSING: &str = "Property 'missingInherited' does not exist on type '";
    if let Some(rest) = message.strip_prefix(MISSING)
        && rest.starts_with("CreateComponentPublicInstance")
        && rest.ends_with("'.")
    {
        return cstr!("{MISSING}<authored instance>'.");
    }
    const THIS: &str = "The 'this' context of type '";
    const VOID: &str = "' is not assignable to method's 'this' of type 'void'.";
    if let Some(rest) = message.strip_prefix(THIS)
        && rest.starts_with("CreateComponentPublicInstance")
        && rest.ends_with(VOID)
    {
        return cstr!("{THIS}<authored instance>{VOID}");
    }
    String::from(message)
}

#[test]
fn absent_call_generic_argument_and_wrong_receiver_keep_their_complete_errors() {
    let Some(native) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "native receiver negatives are required"
        );
        return;
    };
    let shared = shared_with_receiver_methods();
    let mut records = Vec::new();
    for (label, expression, shared, code, message) in [
        (
            "missing-callee",
            "missingInherited()",
            shared.clone(),
            2339,
            "Property 'missingInherited' does not exist on type '<authored instance>'.",
        ),
        (
            "generic-argument",
            "identity<number>('bad')",
            shared.clone(),
            2345,
            "Argument of type 'string' is not assignable to parameter of type 'number'.",
        ),
        (
            "wrong-receiver",
            "greet('x')",
            shared
                .replace(
                    "greet(this: { currentDevice: string }, name: string)",
                    "greet(this: void, name: string)",
                )
                .into(),
            2684,
            "The 'this' context of type '<authored instance>' is not assignable to method's 'this' of type 'void'.",
        ),
    ] {
        let source = cstr!(
            "<template>\n  <p>{{{{ {expression} }}}}</p>\n</template>\n<script lang=\"ts\">\nimport {{ defineComponent }} from 'vue'\nimport {{ shared }} from './shared'\nexport default defineComponent({{ mixins: [shared] }})\n</script>\n"
        );
        let files = [
            ("src/shared.ts", shared.as_str()),
            ("src/Negative.vue", source.as_str()),
            FILES[5],
        ];
        let root = create_project_case(label, &files);
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            &root,
            BatchTypeCheckerOptions::default(),
            Some(&native),
        )
        .expect("negative receiver checker");
        checker.enable_options_api();
        checker.scan_project().expect("negative receiver scan");
        let result = checker.check_project().expect("negative receiver check");
        retain_complete_result(&checker, &root, &native, &result, &files, label)
            .expect("retain whole negative source and ordered result before verdict");
        let actual = serde_json::json!({"exitCode":result.exit_code,"success":result.success,
            "diagnostics":result.diagnostics.iter().map(|d|serde_json::json!({
                "file":d.file,"line":d.line,"column":d.column,"code":d.code,"severity":d.severity,
                "message":portable_negative(&d.message),"blockType":d.block_type.as_ref().map(|b|cstr!("{b:?}"))})).collect::<Vec<_>>()});
        let expected = serde_json::json!({"exitCode":1,"success":false,"diagnostics":[{
            "file":root.join("src/Negative.vue"),"line":1,"column":if code==2345 { 25 } else { 8 },
            "code":code,"severity":1,"message":message,"blockType":"Template"}]});
        records.push((label, actual, expected));
        drop(checker);
        std::fs::remove_dir_all(&root).expect("remove completed negative receiver fixture");
    }
    for (label, actual, expected) in records {
        assert_eq!(actual, expected, "whole negative receiver vector: {label}");
    }
}
