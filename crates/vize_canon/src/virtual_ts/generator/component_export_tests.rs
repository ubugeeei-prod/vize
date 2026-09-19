use super::strip_synthetic_any_defaults;

#[test]
fn synthetic_any_defaults_are_stripped_without_touching_constraints() {
    assert_eq!(
        strip_synthetic_any_defaults("T extends { id: string; } = any").as_str(),
        "T extends { id: string; }"
    );
    assert_eq!(strip_synthetic_any_defaults("T = any").as_str(), "T");
    assert_eq!(
        strip_synthetic_any_defaults("T extends (value: string) => void = any").as_str(),
        "T extends (value: string) => void"
    );
    assert_eq!(
        strip_synthetic_any_defaults("A extends Record<string, any> = any, B = A").as_str(),
        "A extends Record<string, any>, B = A"
    );
    assert_eq!(
        strip_synthetic_any_defaults("A = any, B extends A = any").as_str(),
        "A, B extends A"
    );
    assert_eq!(
        strip_synthetic_any_defaults("const T extends Tab").as_str(),
        "const T extends Tab"
    );
    assert_eq!(
        strip_synthetic_any_defaults("T = string").as_str(),
        "T = string"
    );
}
