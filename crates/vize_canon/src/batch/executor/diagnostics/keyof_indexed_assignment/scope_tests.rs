use std::path::Path;

use super::matches_at;

fn matches_assignment(source: &str) -> bool {
    let offset = source.find("target[key").unwrap() as u32;
    matches_at(source, Path::new("scope.ts"), offset)
}

#[test]
fn a_shadowing_parameter_does_not_inherit_an_outer_cast() {
    let source = r#"type A = { text: string; count: number };
declare const target: A;
declare const key: string;
const value = null as unknown as A[keyof A];
function assign(value: boolean) {
  target[key as keyof A] = value;
}
"#;
    assert!(!matches_assignment(source));
}

#[test]
fn an_out_of_scope_cast_does_not_describe_a_different_binding() {
    let source = r#"type A = { text: string; count: number };
declare const target: A;
declare const key: string;
let value = true;
{ const value = null as unknown as A[keyof A]; }
target[key as keyof A] = value;
"#;
    assert!(!matches_assignment(source));
}

#[test]
fn reassignment_invalidates_an_initializer_cast() {
    let source = r#"type A = { text: string; count: number };
declare const target: A;
declare const key: string;
let value: unknown = null as unknown as A[keyof A];
value = true;
target[key as keyof A] = value;
"#;
    assert!(!matches_assignment(source));
}

#[test]
fn writes_and_widened_annotations_cannot_prove_an_alias_type() {
    for declaration in [
        "const value: unknown = null as unknown as A[keyof A];",
        "let value = null as unknown as A[keyof A]; value = true;",
        "const value = null as unknown as A[keyof A]; value = true;",
        "let value = null as unknown as A[keyof A]; value += 1;",
        "let value = null as unknown as A[keyof A]; ({ value } = { value: true });",
        "let value = null as unknown as A[keyof A]; function mutate() { value = true; }",
        "let value = null as unknown as A[keyof A]; for (value of [true]) {}",
    ] {
        let source = format!(
            "type A = {{ text: string; count: number }}; declare const target: A; \
             declare const key: string; {declaration} target[key as keyof A] = value;"
        );
        assert!(!matches_assignment(&source), "{declaration}");
    }
}

#[test]
fn immutable_aliases_and_inline_casts_remain_supported() {
    for declaration in ["const", "let", "var"] {
        let source = format!(
            "type A = {{ text: string; count: number }}; declare const target: A; \
             declare const key: string; {declaration} value = null as unknown as A[keyof A]; \
             target[key as keyof A] = (value);"
        );
        assert!(matches_assignment(&source), "{declaration}");
    }
    assert!(matches_assignment(
        "type A = { text: string; count: number }; declare const target: A; \
         declare const key: string; target[key as keyof A] = null as unknown as A[keyof A];"
    ));
}

#[test]
fn shadowing_types_do_not_compare_equal_by_spelling() {
    for nested in [
        "function assign<A>() { target[key as keyof A] = value; }",
        "{ type A = { flag: boolean }; target[key as keyof A] = value; }",
    ] {
        let source = format!(
            "type A = {{ text: string; count: number }}; declare const target: A; \
             declare const key: string; const value = null as unknown as A[keyof A]; {nested}"
        );
        assert!(!matches_assignment(&source), "{nested}");
    }
    assert!(!matches_assignment(
        "type Box<T> = { value: T }; function outer<T>() { \
         const value = null as unknown as Box<T>[keyof Box<T>]; \
         function inner<T>(target: Box<T>, key: string) { target[key as keyof Box<T>] = value; }}"
    ));
}

#[test]
fn nested_scopes_keep_unshadowed_bindings_and_types() {
    assert!(matches_assignment(
        "type A = { text: string; count: number }; declare const target: A; \
         declare const key: string; const value = null as unknown as A[keyof A]; \
         { const unused = 1; function assign() { target[key as keyof A] = value; }}"
    ));
}

#[test]
fn unresolved_types_and_forward_aliases_fail_closed() {
    for source in [
        "declare const target: Missing; declare const key: string; \
         const value = null as unknown as Missing[keyof Missing]; target[key as keyof Missing] = value;",
        "type A = { text: string; count: number }; declare const target: A; \
         declare const key: string; target[key as keyof A] = value; \
         const value = null as unknown as A[keyof A];",
        "type A = ; target[key as keyof A] = null as A[keyof A];",
    ] {
        assert!(!matches_assignment(source), "{source}");
    }
}

#[test]
fn nested_rhs_errors_and_non_assignment_operators_are_not_suppressed() {
    let source = "type A = { text: string; count: number }; declare const target: A; \
                  declare const key: string; target[key as keyof A] = (() => { \
                  const wrong: string = 42; return wrong; })() as A[keyof A];";
    let index = super::AssignmentIndex::new(source, Path::new("scope.ts"));
    assert!(matches_assignment(source));
    for needle in ["wrong: string", "key as", "42", "return wrong"] {
        assert!(
            !index.matches_at(source.find(needle).unwrap() as u32),
            "{needle}"
        );
    }
    assert!(!matches_assignment(&source.replace(
        "target[key as keyof A] =",
        "target[key as keyof A] +="
    )));
}

#[test]
fn one_index_retains_all_assignment_locations() {
    let source = "type A = { text: string; count: number }; declare const target: A; \
                  declare const key: string; const value = null as unknown as A[keyof A]; \
                  target[key as keyof A] = value; target[key as keyof A] = value;";
    let index = super::AssignmentIndex::new(source, Path::new("scope.ts"));
    let offsets: Vec<_> = source
        .match_indices("target[key")
        .map(|(offset, _)| offset as u32)
        .collect();
    assert_eq!(index.offsets, offsets);
    for _ in 0..100 {
        for offset in &offsets {
            assert!(index.matches_at(*offset));
        }
        assert!(!index.matches_at(u32::MAX));
    }
    assert!(
        super::AssignmentIndex::new("const value = 1;", Path::new("scope.ts"))
            .offsets
            .is_empty()
    );
}
