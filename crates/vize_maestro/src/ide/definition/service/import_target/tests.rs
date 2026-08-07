#[test]
fn the_importing_statement_is_matched_by_offset_and_binding() {
    let content = "import { a } from \"./a\";\nimport { useCounter, type User } from \"../composables/useCounter\";\n";
    let offset = content.find("useCounter").unwrap() + 2;
    assert_eq!(
        super::importing_specifier(content, offset, "useCounter"),
        Some((
            "../composables/useCounter".to_owned(),
            "useCounter".to_owned()
        )),
    );
    // Same statement, different word: `type User` binds too.
    let offset = content.find("User").unwrap();
    assert_eq!(
        super::importing_specifier(content, offset, "User"),
        Some(("../composables/useCounter".to_owned(), "User".to_owned())),
    );
    // Outside any import statement: no match.
    assert_eq!(
        super::importing_specifier(content, content.len() - 1, "useCounter"),
        None
    );
}

#[test]
fn renamed_default_and_namespace_imports_bind() {
    // The alias is local; the target declares the source name.
    assert_eq!(
        super::bound_source_name("import { long as short } from", "short").as_deref(),
        Some("long"),
    );
    assert_eq!(
        super::bound_source_name("import { long as short } from", "long"),
        None
    );
    assert_eq!(
        super::bound_source_name("import { type User as U } from", "U").as_deref(),
        Some("User"),
    );
    assert_eq!(
        super::bound_source_name("import Default, { x } from", "Default").as_deref(),
        Some("Default"),
    );
    assert_eq!(
        super::bound_source_name("import * as ns from", "ns").as_deref(),
        Some("ns")
    );
}

#[test]
fn tag_imports_resolve_through_aliases_anywhere_in_the_file() {
    let content = "import { Widget as LocalWidget } from \"@/comps\";\n";
    assert_eq!(
        super::bound_import(content, "LocalWidget"),
        Some(("@/comps".to_owned(), "Widget".to_owned())),
    );
    assert_eq!(super::bound_import(content, "Widget"), None);
}

#[test]
fn reexports_cover_named_renames_and_stars() {
    let barrel =
        "export { default as UiButton } from \"./UiButton.vue\";\nexport * from \"./tokens\";\n";
    // `default` has no locatable declaration name, so the hop keeps the
    // requested name.
    assert_eq!(
        super::reexport_specifier(barrel, "UiButton"),
        Some(("./UiButton.vue".to_owned(), "UiButton".to_owned())),
    );
    assert_eq!(
        super::reexport_specifier(barrel, "anything"),
        Some(("./tokens".to_owned(), "anything".to_owned())),
    );
    // A renaming barrel hop continues under the source name.
    let renaming = "export { Widget as LocalWidget } from \"./Widget\";\n";
    assert_eq!(
        super::reexport_specifier(renaming, "LocalWidget"),
        Some(("./Widget".to_owned(), "Widget".to_owned())),
    );
}
