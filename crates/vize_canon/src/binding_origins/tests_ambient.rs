use super::{BindingOrigin, tests::observe};
use oxc_span::SourceType;

#[test]
fn annotated_runtime_custom_functions_cannot_borrow_provider_identity() {
    for text in [
        "const useTemplateRef: typeof import('vue').useTemplateRef = <T = unknown>(key: string): Readonly<{ value: T | null }> => ({value:null});",
        "function custom<T = unknown>(key: string): Readonly<{value:T|null}> {return {value:null};} const useTemplateRef: typeof import('vue').useTemplateRef = custom;",
        "declare const actual: typeof import('vue').useTemplateRef; let useTemplateRef: typeof actual = actual;",
        "declare const actual: typeof import('vue').useTemplateRef; const useTemplateRef: typeof actual = actual as typeof actual;",
    ] {
        let (origin, resolutions) = observe(text, SourceType::ts(), true);
        assert!(
            matches!(origin, BindingOrigin::Custom(_)),
            "{text}: {origin:?}"
        );
        assert_eq!(resolutions, 0);
    }
}

#[test]
fn uninitialized_runtime_annotations_are_not_ambient_provider_declarations() {
    for declaration in ["let", "var"] {
        let text = format!("{declaration} useTemplateRef: typeof import('vue').useTemplateRef;");
        let (origin, resolutions) = observe(&text, SourceType::ts(), true);
        assert!(matches!(origin, BindingOrigin::Custom(_)));
        assert_eq!(resolutions, 0);
    }
}

#[test]
fn actual_ambient_declarations_and_immutable_value_aliases_keep_provider_identity() {
    for (text, source_type, global_script) in [
        (
            "declare const useTemplateRef: typeof import('vue').useTemplateRef;",
            SourceType::ts(),
            true,
        ),
        (
            "const useTemplateRef: typeof import('vue').useTemplateRef;",
            SourceType::d_ts(),
            true,
        ),
        (
            "export {}; declare global {var useTemplateRef: typeof import('vue').useTemplateRef;}",
            SourceType::ts(),
            false,
        ),
        (
            "declare const actual: typeof import('vue').useTemplateRef; const useTemplateRef: typeof actual = actual;",
            SourceType::ts(),
            true,
        ),
    ] {
        let (origin, resolutions) = observe(text, source_type, global_script);
        assert_eq!(origin, BindingOrigin::Vue, "{text}");
        assert_eq!(resolutions, 1);
    }
}

#[test]
fn merged_overload_declarations_preserve_custom_symbol_identity() {
    let text = "declare function useTemplateRef(key: string): {value:string}; declare function useTemplateRef(key: number): {value:number};";
    let (origin, resolutions) = observe(text, SourceType::d_ts(), true);
    assert!(matches!(origin, BindingOrigin::Custom(_)));
    assert_eq!(resolutions, 0);
}
