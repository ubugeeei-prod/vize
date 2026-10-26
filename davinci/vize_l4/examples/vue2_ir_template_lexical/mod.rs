//! An external consumer of the public retained-owner/shared-IR APIs.
//! Original source and explicit caller-owned binding identities are preserved.

use alloc::vec::Vec;
use vize_l2::resolution::{BindingId, BindingLookup};
use vize_l4::expr::vue::{Access, Binding};

const NAMES: &[&str] = &[
    "a",
    "b",
    "日本",
    "Math",
    "_vm",
    "_vm1",
    "render",
    "render1",
    "staticRenderFns",
    "staticRenderFns1",
];
struct Lookup;
impl BindingLookup for Lookup {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        NAMES
            .iter()
            .position(|known| *known == name)
            .map(|index| BindingId::new(index as u32))
    }
}

pub(super) fn capture_rows() -> Vec<serde_json::Value> {
    let expressions = [
        ("_vm", false),
        (r"_\u0076m", false),
        ("({_vm})", false),
        ("new _vm(a)", true),
        ("render", false),
        (r"r\u0065nder", false),
        ("staticRenderFns", false),
        (r"\u0073taticRenderFns", false),
        (
            "[_vm,_vm1,render,render1,staticRenderFns,staticRenderFns1]",
            false,
        ),
        ("({_vm:a,render:b,staticRenderFns:日本})", false),
        ("'_vm render staticRenderFns'", false),
    ];
    let mut rows = Vec::new();
    for role in ["global", "local", "context", "mixed"] {
        let bindings = NAMES
            .iter()
            .enumerate()
            .map(|(index, name)| Binding {
                id: BindingId::new(index as u32),
                access: if *name == "Math" {
                    Access::Global
                } else if index < 4 || role == "context" {
                    Access::Context
                } else if role == "local" {
                    Access::Local
                } else if role == "mixed"
                    && matches!(*name, "_vm1" | "render1" | "staticRenderFns1")
                {
                    Access::Context
                } else {
                    Access::Global
                },
            })
            .collect::<Vec<_>>();
        for (expression, constructor) in expressions {
            let source = vize_l0::cstr!("<div>{{{{ {expression} }}}}</div>");
            let mut row = super::capture(&source, &Lookup, &bindings);
            row["role"] = role.into();
            row["expression"] = expression.into();
            row["constructor"] = constructor.into();
            rows.push(row);
        }
    }
    rows
}
