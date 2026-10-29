use proc_macro2::TokenStream;
use syn::Expr;

fn expand_body(body: &str) -> String {
    let body: TokenStream = body.parse().expect("token stream");
    let mut pieces = Vec::new();
    let mut printer = super::Printer::new();
    printer.render(&mut pieces, body).expect("render");
    printer.flush(&mut pieces);
    let buffer: Expr = syn::parse_str("&mut ts").expect("buffer");
    crate::emit_ts::quote_pieces(buffer, pieces).to_string()
}

fn assert_static(body: &str, expected: &str) {
    let generated = expand_body(body);
    let literal = format!("{expected:?}");
    assert!(
        generated.contains(&literal),
        "missing {expected:?}\n{generated}"
    );
    assert_eq!(generated.matches("reserve_ts").count(), 1, "{generated}");
    assert!(
        generated.contains(&format!("__len : usize = {}", expected.len())),
        "reserve length is not {}\n{generated}",
        expected.len()
    );
    assert!(!generated.contains("format !"), "{generated}");
    assert!(!generated.contains(":: std"), "{generated}");
    assert!(!generated.contains(":: alloc"), "{generated}");
}

#[test]
fn dollars_glue_and_empty_braces_are_one_static_copy() {
    assert_static("type $Slots = {};", "type $Slots = {};\n");
}

#[test]
fn a_literal_ts_string_is_folded_into_the_static_reserve() {
    assert_static(
        "export type Emits = { #(\"  \") #TsString(\"click\"): any[]; };",
        "export type Emits = {\n  \"click\": any[];\n};\n",
    );
}

#[test]
fn a_raw_hole_keeps_single_quoted_indexes() {
    assert_static(
        "type $Slots = Awaited<ReturnType<typeof __setup>> #(\"['__vize_typed_slots']\");",
        "type $Slots = Awaited<ReturnType<typeof __setup>>['__vize_typed_slots'];\n",
    );
}

#[test]
fn arrows_and_logical_and_keep_typescript_spaces() {
    assert_static(
        "type Handler = (value: string) => void;",
        "type Handler = (value: string) => void;\n",
    );
    assert_static("const ok = a && b;", "const ok = a && b;\n");
}

#[test]
fn a_repeat_reserves_once_and_clones_the_sequence() {
    let generated = expand_body("#(name in names => #name,)*");
    assert_eq!(generated.matches("reserve_ts").count(), 1, "{generated}");
    assert!(generated.contains(". clone ()"), "{generated}");
    assert_eq!(generated.matches("for name in").count(), 2, "{generated}");
    assert!(generated.contains("hole_len"), "{generated}");
    assert!(!generated.contains(":: std"), "{generated}");
    assert!(!generated.contains("Vec"), "{generated}");
}

#[test]
fn a_statement_splice_keeps_tight_braces_and_its_own_reserve() {
    let generated = expand_body("{ #{ let _kept = 1; } #(\"x\") }");
    assert!(generated.contains("\"{\""), "{generated}");
    assert!(generated.contains("\"x}\""), "{generated}");
    assert!(generated.contains("_kept"), "{generated}");
    assert_eq!(generated.matches("reserve_ts").count(), 2, "{generated}");
}

#[test]
fn a_repeat_statement_runs_inside_the_iteration() {
    let generated = expand_body("#(item in items => #{ let _seen = item; } #item)*");
    assert_eq!(generated.matches("for item in").count(), 1, "{generated}");
    assert!(generated.contains("_seen"), "{generated}");
    assert!(generated.contains("reserve_ts"), "{generated}");
    assert!(!generated.contains(". clone ()"), "{generated}");
}

#[test]
fn a_dynamic_ts_string_stays_a_hole() {
    let generated = expand_body("#TsString(name)");
    assert!(generated.contains("hole_len"), "{generated}");
    assert!(generated.contains("TsString"), "{generated}");
    assert_eq!(generated.matches("reserve_ts").count(), 1, "{generated}");
}
