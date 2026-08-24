//! Exact-oracle rejection suite for the disegno folio parser (TS-13:
//! whole-message equality, no partial matching).
//!
//! Every message here is part of the format contract. The common prefix
//! `P` puts the body's first line at line 5.

use vize_davinci::folio::{Folio, FolioError};
use vize_disegno::folio::DisegnoFolio;
use vize_s0::String;

/// Valid page prefix: `[disegno]` header, `ops=0`, blank, `[disegno.ops]`.
const P: &str = "[disegno]\nops=0\n\n[disegno.ops]\n";

/// `(input, expected 1-based line, expected message)`.
const REJECTIONS: &[(&str, usize, &str)] = &[
    // -- sections and header ------------------------------------------------
    (
        "x\n[disegno]\nops=0\n",
        1,
        "content before the [disegno] header",
    ),
    ("[disegno.ops]\n", 1, "first section must be [disegno]"),
    ("[frame]\n", 1, "first section must be [disegno]"),
    (
        "[disegno]\nops=0\n[disegno]\n",
        3,
        "duplicate section [disegno]",
    ),
    ("[disegno]\nops=0\n[frame]\n", 3, "unknown section [frame]"),
    ("", 0, "missing [disegno] header"),
    ("\n\n", 0, "missing [disegno] header"),
    ("[disegno]\nops\n", 2, "field line is missing `=`"),
    ("[disegno]\ncount=1\n", 2, "unknown field `count`"),
    ("[disegno]\nops=0\nops=1\n", 3, "duplicate field `ops`"),
    ("[disegno]\nops=x7\n", 2, "invalid integer `x7`"),
    ("[disegno]\n", 0, "missing field `ops`"),
];

/// Ops-section rejections; every input is `P` plus the listed body.
const OPS_REJECTIONS: &[(&str, usize, &str)] = &[
    // -- indentation and tree structure -------------------------------------
    (" ui.text \"x\" @1:2\n", 5, "odd indentation"),
    ("  ui.text \"x\" @1:2\n", 5, "over-indented line"),
    ("ui.portal @1:2\n", 5, "unknown op `ui.portal`"),
    ("attr x @1:2\n", 5, "`attr` outside an element"),
    (
        "ui.element d @1:2\n  vue.directive \"f\" @1:2\n  attr x @1:2\n",
        7,
        "attribute after a binding",
    ),
    (
        "ui.element d @1:2\n  ui.text \"x\" @1:2\n  attr x @1:2\n",
        7,
        "attribute after a child",
    ),
    (
        "ui.model read=js(\"m\" @1:2) write=js(\"m\" @1:2) @1:2\n",
        5,
        "binding outside an element",
    ),
    (
        "ui.element d @1:2\n  ui.text \"x\" @1:2\n  vue.directive \"f\" @1:2\n",
        7,
        "binding after a child",
    ),
    (
        "ui.element d @1:2\n  ui.model read=js(\"m\" @1:2) write=js(\"m\" @1:2) @1:2\n    ui.text \"x\" @1:2\n",
        7,
        "expected `attr` under `ui.model`",
    ),
    (
        "ui.element d @1:2\n  ui.model read=js(\"m\" @1:2) write=js(\"m\" @1:2) @1:2\n    branch @1:2\n",
        7,
        "`branch` outside `ui.if`",
    ),
    ("branch @1:2\n", 5, "`branch` outside `ui.if`"),
    (
        "ui.if @1:2\n  ui.text \"x\" @1:2\n",
        6,
        "expected `branch` under `ui.if`",
    ),
    (
        "ui.if @1:2\n  attr x @1:2\n",
        6,
        "expected `branch` under `ui.if`",
    ),
    (
        "ui.if @1:2\n  ui.model read=js(\"m\" @1:2) write=js(\"m\" @1:2) @1:2\n",
        6,
        "expected `branch` under `ui.if`",
    ),
    // -- the ui.bind / ui.on lines (P2-9 series 5) ---------------------------
    (
        "ui.bind name=\"a\" value=js(\"v\" @1:2) @1:2\n",
        5,
        "binding outside an element",
    ),
    (
        "ui.on name=\"click\" handler=js(\"go()\" @1:2) @1:2\n",
        5,
        "binding outside an element",
    ),
    (
        "ui.element d @1:2\n  ui.bind name=top @1:2\n",
        6,
        "expected quoted string or an expression payload",
    ),
    (
        "ui.element d @1:2\n  ui.bind mods=\"a,,b\" value=js(\"v\" @1:2) @1:2\n",
        6,
        "invalid modifier list",
    ),
    (
        "ui.element d @1:2\n  ui.on name=\"click\" handler=x @1:2\n",
        6,
        "expected an expression payload",
    ),
    (
        "ui.element d @1:2\n  ui.bind name=\"a\" value=js(\"v\" @1:2)\n",
        6,
        "missing span",
    ),
    (
        "ui.slot name=\"s\" @1:2\n  ui.text \"x\" @1:2\n  ui.bind name=\"a\" value=js(\"v\" @1:2) @1:2\n",
        7,
        "binding after a child",
    ),
    (
        "ui.slot name=\"s\" @1:2\n  ui.on name=\"e\" @1:2\n  attr a @1:2\n",
        7,
        "attribute after a binding",
    ),
    // -- line grammar --------------------------------------------------------
    ("ui.if\n", 5, "missing span"),
    ("ui.text \"x\"\n", 5, "missing span"),
    ("ui.if @1:x\n", 5, "invalid span `@1:x`"),
    ("ui.if 1:2\n", 5, "invalid span `1:2`"),
    ("ui.text \"x\"y @1:2\n", 5, "trailing content `y @1:2`"),
    ("ui.text x @1:2\n", 5, "expected quoted string"),
    ("ui.text \"x @1:2\n", 5, "unterminated quoted string"),
    ("ui.text \"x\\q\" @1:2\n", 5, "invalid escape `\\q`"),
    (
        "ui.interpolation @1:2\n",
        5,
        "expected an expression payload",
    ),
    (
        "ui.for value=js(\"v\" @1:2) @1:2\n",
        5,
        "expected `source=`",
    ),
    (
        "ui.for source=js(\"s\" @1:2) @1:2\n",
        5,
        "expected `value=`",
    ),
    (
        "ui.element d @1:2\n  ui.model read=js(\"m\" @1:2) @1:2\n",
        6,
        "expected `write=`",
    ),
    (
        "ui.element d @1:2\n  ui.model write=js(\"m\" @1:2) @1:2\n",
        6,
        "expected `read=`",
    ),
    ("ui.slot header @1:2\n", 5, "expected `name=`"),
    (
        "ui.slot name=header @1:2\n",
        5,
        "expected quoted string or an expression payload",
    ),
    ("ui.element\n", 5, "missing tag"),
    ("ui.component\n", 5, "missing component name"),
    ("attr\n", 5, "missing attribute name"),
    (
        "ui.element circle ns=math @1:2\n",
        5,
        "invalid namespace `math`",
    ),
    (
        "ui.element d @1:2\n  vue.directive \"f\" mods=\"a,,b\" @1:2\n",
        6,
        "invalid modifier list",
    ),
    (
        "ui.element d @1:2\n  vue.directive \"f\" arg=top @1:2\n",
        6,
        "expected quoted string or an expression payload",
    ),
    (
        "ui.element d @1:2\n  attr a=b @1:2\n",
        6,
        "expected quoted string",
    ),
    (
        "ui.if @1:2\n  branch x @1:2\n",
        6,
        "expected an expression payload",
    ),
    (
        "ui.element d @1:2\n  vue.sync name=js(\"foo\" @1:2) value=js(\"bar\" @1:2) @1:2\n",
        6,
        "expected quoted string",
    ),
    (
        "ui.element d @1:2\n  vue.once extra @1:2\n",
        6,
        "invalid span `extra @1:2`",
    ),
    (
        "ui.element d @1:2\n  vue.memo @1:2\n",
        6,
        "expected `value=`",
    ),
    // -- expression payload tokens -------------------------------------------
    ("ui.interpolation js(x) @1:2\n", 5, "expected quoted string"),
    ("ui.interpolation js(\"x\") @1:2\n", 5, "missing span"),
    (
        "ui.interpolation js(\"x\" @1:2\n",
        5,
        "unterminated expression payload",
    ),
    (
        "ui.interpolation js(\"x\" 1:2) @1:2\n",
        5,
        "invalid span `1:2`",
    ),
    (
        "ui.interpolation opaque(mystery \"x\" @1:2) @1:2\n",
        5,
        "unknown opaque reason `mystery`",
    ),
    ("ui.interpolation opaque(\n", 5, "expected an opaque reason"),
    ("ui.interpolation foreign(\n", 5, "expected a dialect id"),
    (
        "ui.interpolation foreign( \"x\" @1:2) @1:2\n",
        5,
        "expected a dialect id",
    ),
    (
        "ui.if @1:2\n  branch js(\"c\" @1:2)x @1:2\n",
        6,
        "trailing content `x @1:2`",
    ),
];

fn assert_rejected(input: &str, line: usize, message: &str) {
    let error = DisegnoFolio::parse(input).expect_err("input must be rejected");
    assert_eq!(
        error,
        FolioError::new(line, String::from(message)),
        "input: {input:?}"
    );
}

#[test]
fn header_and_section_rejections_carry_exact_messages() {
    for (input, line, message) in REJECTIONS {
        assert_rejected(input, *line, message);
    }
}

#[test]
fn ops_section_rejections_carry_exact_messages() {
    for (body, line, message) in OPS_REJECTIONS {
        let mut input = String::from(P);
        input.push_str(body);
        assert_rejected(input.as_str(), *line, message);
    }
}
