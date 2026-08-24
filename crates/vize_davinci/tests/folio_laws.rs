//! Folio trait laws on hand-written croquis folio texts.
//!
//! The mode-explicit contract under test (see `crates/vize_davinci/src/folio.rs`
//! and `davinci-road/plan/folio-format.md`):
//!
//! - `Full`: `print(parse(t)) == t` for canonical text, `parse(print(v)) == v`
//!   for (normalized) values; non-canonical input is normalized by the first
//!   print.
//! - `Display`: spans and default markers elided, no round-trip law.

use vize_davinci::assert_folio_snapshot;
use vize_davinci::folio::croquis::CroquisFolio;
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::{String, cstr};

/// A canonical text exercising every section of the croquis folio.
const CANONICAL: &str = "\
[vir]
script_setup=true
scopes=3
bindings=4

[surface.props]
title!:string
subtitle?=

[surface.emits]
change

[surface.models]
modelValue:string

[surface.expose]
{ focus }
@10:20

[surface.slots]
{ default(props: { item: string }): any }

[macros]
@defineProps<{ title: string }> @5:40
@defineEmits @50:70

[reactivity]
count=ref

[extern]
./helpers {formatDate,parseDate}
vue^

[types]
Props^i@1:30
Rowt@31:60

[bindings]
c:formatDate,parseDate
st:count

[scopes]
~0 mod @0:100 [count,formatDate]
~1 setup @0:100 [parseDate] < ~0
!0 mounted @40:80 < ~1

[errors]
rows=Const@197:237

";

#[test]
fn full_print_is_identity_on_canonical_text() {
    let folio = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
}

#[test]
fn parse_print_is_structural_identity() {
    let folio = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    let printed = folio.print_to_string(FolioMode::Full);
    let reparsed = CroquisFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed, folio);
}

#[test]
fn non_canonical_input_is_normalized_by_the_first_print() {
    // Sections out of order, unsorted names, non-sequential scope ids,
    // extra blank lines: all normalized by print, none an error.
    let scrambled = "\
[vir]
bindings=4
scopes=3
script_setup=true


[bindings]
c:parseDate,formatDate
st:count

[scopes]
~4 mod @0:100 [formatDate,count]
~7 setup @0:100 [parseDate] < ~4
!2 mounted @40:80 < ~7

[surface.props]
title!:string
subtitle?=

[macros]
@defineProps<{ title: string }> @5:40
@defineEmits @50:70

[surface.emits]
change

[surface.models]
modelValue:string

[surface.expose]
{ focus }
@10:20

[surface.slots]
{ default(props: { item: string }): any }

[reactivity]
count=ref

[types]
Props^i@1:30
Rowt@31:60

[extern]
./helpers {formatDate,parseDate}
vue^

[errors]
rows=Const@197:237
";
    let folio = CroquisFolio::parse(scrambled).expect("non-canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    // And the canonical parse is the same value (normalization happened at
    // parse time, not only in print).
    let canonical = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(folio, canonical);
}

#[test]
fn display_mode_elides_spans_and_defaults() {
    let folio = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    let expected = "\
[vir]
script_setup=true
scopes=3
bindings=4

[surface.props]
title!:string
subtitle?

[surface.emits]
change

[surface.models]
modelValue:string

[surface.expose]
{ focus }

[surface.slots]
{ default(props: { item: string }): any }

[macros]
@defineProps<{ title: string }>
@defineEmits

[reactivity]
count=ref

[extern]
./helpers {formatDate,parseDate}
vue^

[types]
Props^i
Rowt

[bindings]
c:formatDate,parseDate
st:count

[scopes]
~0 mod [count,formatDate]
~1 setup [parseDate] < ~0
!0 mounted < ~1

[errors]
rows=Const

";
    assert_eq!(folio.print_to_string(FolioMode::Display).as_str(), expected);
}

#[test]
fn multi_line_macro_type_args_round_trip() {
    let text = "\
[vir]
script_setup=true
scopes=0
bindings=0

[macros]
@defineEmits<<{
  change: [value: string];

  update: [id: number, value: string];
}>> @14:98

";
    let folio = CroquisFolio::parse(text).expect("multi-line macro parses");
    assert_eq!(folio.macros.len(), 1);
    assert_eq!(
        folio.macros[0]
            .type_args
            .as_ref()
            .expect("type args")
            .as_str(),
        "<{\n  change: [value: string];\n\n  update: [id: number, value: string];\n}>"
    );
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), text);
}

#[test]
fn empty_dump_prints_header_only() {
    let text = "\
[vir]
script_setup=false
scopes=0
bindings=0

";
    let folio = CroquisFolio::parse(text).expect("header-only text parses");
    assert_eq!(folio, CroquisFolio::default());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), text);
}

fn parse_err(input: &str) -> FolioError {
    CroquisFolio::parse(input).expect_err("input must not parse")
}

#[test]
fn parse_errors_carry_line_numbers() {
    assert_eq!(
        parse_err("x\n"),
        FolioError::new(1, cstr!("content before the [vir] header"))
    );
    assert_eq!(
        parse_err("[scopes]\n"),
        FolioError::new(1, cstr!("first section must be [vir]"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\nscopes=0\nbindings=0\n\n[macros]\n\n[macros]\n"),
        FolioError::new(8, cstr!("duplicate section [macros]"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\n"),
        FolioError::new(0, cstr!("incomplete [vir] header"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=yes\n"),
        FolioError::new(2, cstr!("script_setup must be true or false"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\nscopes=0\nbindings=0\n\n[scopes]\n~0 mod @x:y\n"),
        FolioError::new(7, cstr!("malformed scope span"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\nscopes=1\nbindings=0\n\n[scopes]\n~0 mod @0:1 < ~9\n"),
        FolioError::new(0, cstr!("unresolved scope reference ~9"))
    );
    assert_eq!(
        parse_err(
            "[vir]\nscript_setup=true\nscopes=2\nbindings=0\n\n[scopes]\n~0 mod @0:1\n~0 fn @0:1\n"
        ),
        FolioError::new(0, cstr!("duplicate scope id ~0"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\nscopes=0\nbindings=0\n\n[macros]\n@defineProps<{\n"),
        FolioError::new(7, cstr!("macro line is missing a span"))
    );
    assert_eq!(
        parse_err("[vir]\nscript_setup=true\nscopes=0\nbindings=0\n\n[surface.props]\nnodefault\n"),
        FolioError::new(7, cstr!("prop line is missing a !/? marker"))
    );
}

#[test]
fn hand_built_normalized_value_round_trips() {
    let mut folio = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    // Perturb into a non-normalized equivalent, then normalize by hand.
    folio.bindings[0].names.reverse();
    folio.scopes[1].bindings.reverse();
    folio.normalize();

    let printed = folio.print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), CANONICAL);
    let reparsed = CroquisFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed, folio);
}

#[test]
fn folio_snapshot_macro_uses_the_normalized_full_printer() {
    let folio = CroquisFolio::parse(CANONICAL).expect("canonical text parses");
    assert_folio_snapshot!(folio);
}

#[test]
fn folio_error_displays_line_information() {
    let error = FolioError::new(3, String::from("boom"));
    let mut rendered = String::default();
    use core::fmt::Write as _;
    write!(rendered, "{error}").expect("write to string");
    assert_eq!(rendered.as_str(), "folio parse error at line 3: boom");
}
