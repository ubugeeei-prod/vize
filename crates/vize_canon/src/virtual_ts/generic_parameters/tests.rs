use super::GenericParameters;

#[test]
fn parser_owned_boundaries_preserve_full_variants_and_rows() {
    for (raw, names, defaults, aliases, fallback) in [
        (
            "const T extends 'a,b'",
            "T",
            "const T extends 'a,b' = any",
            "T extends 'a,b' = any",
            "'a,b'",
        ),
        (
            "T extends `a,${string}`, U = 'x,y'",
            "T, U",
            "T extends `a,${string}` = any, U = 'x,y'",
            "T extends `a,${string}` = any, U = 'x,y'",
            "`a,${string}`, unknown",
        ),
        (
            "const F extends (left: string, right: number) => [string, number], U",
            "F, U",
            "const F extends (left: string, right: number) => [string, number] = any, U = any",
            "F extends (left: string, right: number) => [string, number] = any, U = any",
            "(left: string, right: number) => [string, number], unknown",
        ),
        (
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "状態, 値",
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "状態 extends Record<'a,b', [string, number]> = { 'a,b': [string, number] }, 値 = 状態",
            "Record<'a,b', [string, number]>, unknown",
        ),
        (
            "T extends 'T', U extends T",
            "T, U",
            "T extends 'T' = any, U extends T = any",
            "T extends 'T' = any, U extends T = any",
            "'T', any",
        ),
        (
            "T, F extends <T>(value: T) => T",
            "T, F",
            "T = any, F extends <T>(value: T) => T = any",
            "T = any, F extends <T>(value: T) => T = any",
            "unknown, <T>(value: T) => T",
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let raw = raw.replace(", U", &vize_carton::cstr!(",{newline}U"));
            let plan = GenericParameters::parse(&raw);
            assert_eq!(plan.diagnostics, vec![], "{raw}");
            assert_eq!(plan.names, names, "{raw}");
            assert_eq!(plan.declaration, defaults, "{raw}");
            assert_eq!(plan.alias_declaration, aliases, "{raw}");
            assert_eq!(plan.fallback, fallback, "{raw}");
            for (rendered, rows) in plan.declaration_variants() {
                for (generated, original) in rows {
                    assert_eq!(
                        rendered.get(generated.clone()),
                        raw.get(original.clone()),
                        "{raw}: {rendered}"
                    );
                }
            }
        }
    }
}

#[test]
fn comments_and_authored_defaults_are_not_inferred_from_spelling() {
    let raw = "/* head , */ const /* modifier , */ T extends 'a=b,c' /* tail , */, // gap ,\n U = any // final ,";
    let plan = GenericParameters::parse(raw);
    assert_eq!(plan.diagnostics, vec![]);
    assert_eq!(plan.names, "T, U");
    assert_eq!(
        plan.declaration,
        "/* head , */ const /* modifier , */ T extends 'a=b,c' = any /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(
        plan.alias_declaration,
        "/* head , */ /* modifier , */ T extends 'a=b,c' = any /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(
        plan.alias_authored,
        "/* head , */ /* modifier , */ T extends 'a=b,c' /* tail , */, // gap ,\n U = any // final ,\n"
    );
    assert_eq!(plan.authored(), vize_carton::cstr!("{raw}\n"));
    assert_eq!(plan.fallback, "'a=b,c', unknown");
    assert!(plan.has_repeated_fallback_constraint());
    for (rendered, rows) in plan.declaration_variants() {
        for (generated, original) in rows {
            assert_eq!(rendered.get(generated.clone()), raw.get(original.clone()));
        }
    }
}

#[test]
fn malformed_parameters_keep_the_entire_authored_signature() {
    for raw in [
        "T extends",
        "T = 'a,b",
        "T extends (a: string, b: number => string",
        "T,, U",
        "const",
    ] {
        let plan = GenericParameters::parse(raw);
        assert!(!plan.diagnostics.is_empty(), "{raw}");
        assert_eq!(plan.raw(), raw);
        for (rendered, rows) in plan.declaration_variants() {
            assert_eq!(rendered, raw);
            assert_eq!(rows, &[(0..raw.len(), 0..raw.len())]);
        }
        for diagnostic in plan.diagnostics {
            assert!(diagnostic.start <= diagnostic.end);
            assert!(diagnostic.end as usize <= raw.len());
        }
    }
}
