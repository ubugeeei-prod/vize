//! The semantic lowering carries the chosen language before projection.

use vize_dialect_moonbit::{dialect::MoonBitDialect, sfc::split};
use vize_l0::Allocator;
use vize_l1_to_l2::{ForeignDialect, LegacyCaps, lower_source_block_with_foreign_expressions};
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, DynamicName, Op};

fn binding_expressions<'a>(bindings: &[BindingOp<'a>], out: &mut Vec<ExprRef<'a>>) {
    for binding in bindings {
        let (name, value) = match binding {
            BindingOp::Bind(bind) => (bind.name, bind.value),
            BindingOp::On(on) => (on.name, on.handler),
            BindingOp::SlotContent(slot) => (slot.name, slot.params),
            BindingOp::VueShow(show) => (None, Some(show.value)),
            _ => continue,
        };
        if let Some(DynamicName::Dynamic(expr)) = name {
            out.push(expr);
        }
        if let Some(expr) = value {
            out.push(expr);
        }
    }
}

fn expressions<'a>(ops: &[Op<'a>], out: &mut Vec<ExprRef<'a>>) {
    for op in ops {
        match op {
            Op::Element(element) => {
                binding_expressions(&element.bindings, out);
                expressions(&element.children.ops, out);
            }
            Op::Component(component) => {
                binding_expressions(&component.bindings, out);
                expressions(&component.children.ops, out);
            }
            Op::Interpolation(interpolation) => out.push(interpolation.expression),
            Op::For(each) => {
                out.push(each.binding.source);
                out.push(each.binding.value);
                out.extend(each.binding.key);
                out.extend(each.binding.index);
                expressions(&each.region.ops, out);
            }
            Op::If(chain) => {
                for branch in &chain.branches {
                    out.extend(branch.condition);
                    expressions(&branch.region.ops, out);
                }
            }
            Op::Slot(slot) => {
                if let DynamicName::Dynamic(name) = slot.name {
                    out.push(name);
                }
                binding_expressions(&slot.bindings, out);
                expressions(&slot.fallback.ops, out);
            }
            Op::Text(_) | Op::Comment(_) => {}
        }
    }
}

#[test]
fn every_expression_is_foreign_and_scopes_keep_exact_authored_names() {
    let source = r#"<script setup lang="moonbit">let x = 1</script><template><section><button :[name]="x" @[event]="x.val += 1" v-show="show.val">{{ if show.val { x.val } else { 0 } }}</button><p v-for="(item, i) in items">{{ item }}</p><Comp v-slot="props" :title><slot :name /></Comp></section></template>"#;
    let allocator = Allocator::new();
    let sfc = split(source).unwrap();
    let (tree, errors) = vize_l1::parse(&allocator, sfc.template.source());
    let dialect = ForeignDialect::new::<MoonBitDialect>("moonbit").unwrap();
    let lowered = lower_source_block_with_foreign_expressions(
        &allocator,
        &tree,
        &errors,
        sfc.template,
        LegacyCaps::VUE3,
        dialect,
    );
    assert_eq!(lowered.diagnostics.len(), 0);
    let mut found = Vec::new();
    expressions(&lowered.root.ops, &mut found);
    assert_eq!(
        found.iter().map(|expr| expr.source()).collect::<Vec<_>>(),
        [
            "name",
            "x",
            "event",
            "x.val += 1",
            "show.val",
            "if show.val { x.val } else { 0 }",
            "items",
            "item",
            "i",
            "item",
            "props",
            "title",
            "name",
        ]
    );
    for expr in found {
        let ExprRef::Foreign(foreign) = expr else {
            panic!("wrong dialect: {expr:?}");
        };
        assert_eq!(foreign.dialect, "moonbit");
        assert_eq!(
            source.get(foreign.span.start as usize..foreign.span.end as usize),
            Some(foreign.source)
        );
    }
    let mut scopes = lowered
        .scopes
        .iter()
        .map(|(node, facts)| {
            (
                node.index(),
                facts
                    .bindings
                    .iter()
                    .map(|binding| binding.name.as_str())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    scopes.sort_by_key(|(node, _)| *node);
    assert_eq!(scopes, [(6, vec!["item", "i"]), (10, vec!["props"])]);
}

#[test]
fn registry_ids_are_checked_once_before_lowering() {
    for name in ["", "MoonBit", "moon-bit", "moonbit1"] {
        assert!(ForeignDialect::new::<MoonBitDialect>(name).is_none());
    }
    assert!(ForeignDialect::new::<MoonBitDialect>("moonbit").is_some());
}
