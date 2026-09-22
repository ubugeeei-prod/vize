//! The virtual MoonBit projection with span links (charter #14).
//!
//! The template is lowered through S1 → S2 like every Vue template; each
//! S2 expression position is then re-read as `ExprRef::Foreign` of
//! dialect `moonbit` (the P6-4a stand-in for a lowering that constructs
//! `Foreign` itself, which P6-4b owns) and emitted by
//! [`crate::dialect::MoonBitDialect`] into **one** virtual `.mbt` file:
//!
//! 1. the script block, verbatim, as the binding environment;
//! 2. `fn __vize_template()`, the template's control structure as MoonBit
//!    control flow — `v-if` chains become `if`/`else if`/`else` (so a
//!    condition must be `Bool`), `v-for` becomes `for … in …` (so the
//!    alias is typed by the iterated collection and scoped to its
//!    region), and each remaining position becomes a typed demand;
//! 3. the demand helpers the template used, and only those.
//!
//! | Position               | Projected as                        | Demand                    |
//! | ---------------------- | ----------------------------------- | ------------------------- |
//! | `{{ e }}`              | `__vize_show(e)`                    | `e : Show`                |
//! | `v-if` / `v-else-if`   | `if e { … } else if …`              | `e : Bool`                |
//! | `v-show`               | `__vize_cond(e)`                    | `e : Bool`                |
//! | `v-for="v in e"`       | `for v in e { … }`                  | `e` iterable              |
//! | `v-for="(v, k) in e"`  | `for k, v in e { … }`               | `e` iterable by two       |
//! | `:a="e"`               | `__vize_bind(e)`                    | well-typed                |
//! | `:[e]` / `@[e]`        | `__vize_name(e)`                    | `e : String`              |
//! | `@a="path"`            | `__vize_on(path)`                   | `path : () -> Unit`       |
//! | `@a="statement"`       | `__vize_on(fn() { statement })`     | statement : `Unit`        |
//!
//! Every expression and the script are copied byte for byte, so each
//! [`SpanLink`] maps a generated range to its authored range one to one.
//! Positions outside this table (`v-model`, slots' scope, custom
//! directives, …) are reported as [`Unsupported`], never dropped silently.

mod emit;

use vize_s0::{Allocator, Span, String, append};
use vize_s1_to_s2::{LegacyCaps, lower_source_block_with_caps};
use vize_s2::expr::ForeignExpr;

use crate::sfc::MoonBitSfc;
use emit::{Emitter, generated_offset, interpolation_parts};

/// The package every projection is checked as.
pub const PACKAGE: &str = "vize/sfc";

/// What a link's generated range carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The script block, the binding environment.
    Script,
    /// The template expression at this index of [`Projection::positions`].
    Expression(usize),
}

/// One generated range and the authored range it was copied from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanLink {
    /// Byte range in [`Projection::text`].
    pub generated: Span,
    /// File-absolute byte range in the SFC.
    pub source: Span,
    /// What the range carries.
    pub role: Role,
}

/// How a template position was projected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionKind {
    /// `{{ e }}`.
    Interpolation,
    /// A `v-if` / `v-else-if` condition.
    Condition,
    /// A `v-show` value.
    Show,
    /// The iterated collection of a `v-for`.
    IterSource,
    /// The value alias of a `v-for`.
    IterValue,
    /// The second alias of a `v-for` (the index of an array).
    IterKey,
    /// A `v-bind` value.
    Bind,
    /// A dynamic argument (`:[e]`, `@[e]`).
    Argument,
    /// A handler reference (`@click="save"`).
    Handler,
    /// An inline handler statement (`@click="n.val += 1"`).
    HandlerStatement,
}

/// One template expression position, re-read as a foreign expression.
#[derive(Debug, Clone, Copy)]
pub struct Position<'a> {
    /// How the position was projected.
    pub kind: PositionKind,
    /// The payload: dialect `moonbit`, the authored text and span.
    pub expr: &'a ForeignExpr<'a>,
    /// Generated range of the whole projected statement (or loop/branch
    /// header): a diagnostic inside it but outside every link belongs to
    /// this position.
    pub statement: Span,
}

/// A template position the P6-4a projection does not cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsupported {
    /// The S2 op mnemonic (`ui.model`, `ui.slot-content`, …).
    pub what: &'static str,
    /// The op's authored range.
    pub span: Span,
}

/// The virtual MoonBit file for one SFC.
#[derive(Debug)]
pub struct Projection<'a> {
    /// The virtual file's name (`<sfc file name>.mbt`).
    pub file_name: String,
    /// The virtual MoonBit source.
    pub text: String,
    /// Span links, in generated order.
    pub links: Vec<SpanLink>,
    /// Template expression positions, in generated order.
    pub positions: Vec<Position<'a>>,
    /// Positions outside the projected subset.
    pub unsupported: Vec<Unsupported>,
}

/// The demand helpers, in emission order: (name, definition).
const HELPERS: [(&str, &str); 5] = [
    (
        "__vize_show",
        "fn[T : Show] __vize_show(value : T) -> Unit {\n  ignore(value.to_string())\n}\n",
    ),
    (
        "__vize_cond",
        "fn __vize_cond(value : Bool) -> Unit {\n  ignore(value)\n}\n",
    ),
    (
        "__vize_bind",
        "fn[T] __vize_bind(value : T) -> Unit {\n  ignore(value)\n}\n",
    ),
    (
        "__vize_name",
        "fn __vize_name(name : String) -> Unit {\n  ignore(name)\n}\n",
    ),
    (
        "__vize_on",
        "fn __vize_on(handler : () -> Unit) -> Unit {\n  ignore(handler)\n}\n",
    ),
];

/// Project `sfc` into its virtual MoonBit file.
#[must_use]
pub fn project<'a>(
    allocator: &'a Allocator,
    sfc: &MoonBitSfc<'a>,
    file_name: &str,
) -> Projection<'a> {
    let (tree, errors) = vize_s1::parse(allocator, sfc.template.source());
    let lowered =
        lower_source_block_with_caps(allocator, &tree, &errors, sfc.template, LegacyCaps::VUE3);
    let mut parts = Vec::new();
    interpolation_parts(&tree.children, sfc.template, &mut parts);
    let mut emitter = Emitter {
        allocator,
        parts,
        projection: Projection {
            file_name: vize_s0::cstr!("{file_name}.mbt"),
            text: String::default(),
            links: Vec::new(),
            positions: Vec::new(),
            unsupported: Vec::new(),
        },
        used: [false; HELPERS.len()],
        depth: 1,
    };
    let out = &mut emitter.projection;
    append!(
        out.text,
        "// vize: virtual MoonBit projection of {file_name} (generated)\n"
    );
    let script = sfc.script;
    let start = generated_offset(&out.text);
    out.text.push_str(script.source());
    out.links.push(SpanLink {
        generated: Span::new(start, generated_offset(&out.text)),
        source: script.span(),
        role: Role::Script,
    });
    out.text
        .push_str("\n///|\nfn __vize_template() -> Unit {\n");
    emitter.region(&lowered.root.ops);
    emitter.projection.text.push_str("}\n");
    for (used, (_, helper)) in emitter.used.iter().zip(HELPERS) {
        if *used {
            emitter.projection.text.push_str("\n///|\n");
            emitter.projection.text.push_str(helper);
        }
    }
    emitter.projection
}
