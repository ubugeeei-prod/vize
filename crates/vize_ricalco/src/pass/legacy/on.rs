//! Vue 2 v-on modifier sugar: strip `.native`, rewrite numeric keyCodes.

use vize_carton::{Allocator, Vec};
use vize_disegno::op::BindingOp;

/// Rewrite `ui.on` modifiers in place. `.native` is removed; the Vue 2
/// built-in keyCode table maps onto Vue 3 key names (`13` → `enter`).
pub(super) fn rewrite<'a>(allocator: &'a Allocator, bindings: &mut Vec<'a, BindingOp<'a>>) {
    for binding in bindings.iter_mut() {
        let BindingOp::On(on) = binding else {
            continue;
        };
        if on.modifiers.is_empty() {
            continue;
        }
        let mut rest = Vec::new_in(&allocator);
        for modifier in &on.modifiers {
            if *modifier == "native" {
                continue;
            }
            rest.push(keycode_to_key_name(modifier).unwrap_or(*modifier));
        }
        on.modifiers = rest;
    }
}

/// Mirrors `@vue/compiler-dom`'s removed `keyCodes` table (and the
/// shipped `desugar_v2_v_on_modifiers`).
fn keycode_to_key_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "8" => "delete",
        "9" => "tab",
        "13" => "enter",
        "27" => "esc",
        "32" => "space",
        "37" => "left",
        "38" => "up",
        "39" => "right",
        "40" => "down",
        "46" => "delete",
        _ => return None,
    })
}
