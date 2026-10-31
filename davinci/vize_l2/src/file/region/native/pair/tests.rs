extern crate std;

use super::*;
use crate::file::NativeFileConditionalState;
use crate::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use crate::op::Op;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::SurfaceParseOptions,
};

std::thread_local! { static FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
pub(super) fn inject(point: u8) {
    FAULT.with(|fault| {
        if fault.get() == point {
            fault.set(0);
            panic!("interrupted genuine native If/Else custody at {point}");
        }
    });
}

fn owner<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateOwner<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("whole owner"));
    if owner.selected().setup().is_some() {
        owner.parse_setup_program().unwrap();
    }
    owner
}

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let mut owner = owner(arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let mut children = walk.selected().children();
        while let Some(child) = children.next() {
            walk.child_with_tail(child, &mut children).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

#[test]
fn original_pair_mints_one_if_two_real_regions_and_the_same_boolean_file_read() {
    for (source, expected) in [
        (
            "<script setup>let ready=true</script><template><p v-if='ready'>yes</p><b v-else>no</b></template>",
            true,
        ),
        (
            "<script setup>const ready=false</script><template><p v-if='ready'/><b v-else/></template>",
            false,
        ),
        (
            "\r\n<script setup lang='ts'>var prêt:boolean=true</script><template><p v-if='prêt'>雪🌸</p><b v-else>戻🌸</b></template>\r\n",
            true,
        ),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let file = output.file().unwrap();
        assert!(file.is_complete() && output.view().is_ok());
        assert_eq!(file.scopes().len(), 2);
        let [unit] = file.units() else {
            panic!("actual setup")
        };
        let [record] = file.native_conditionals() else {
            panic!("one actual condition")
        };
        let NativeFileConditionalState::Admitted(node) = record.state() else {
            panic!("closed pair")
        };
        let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
            panic!("actual If root")
        };
        assert!(record.accepts(original));
        let [first, last] = original.branches.as_slice() else {
            panic!("two original branches")
        };
        let [Op::Element(first_body)] = first.region.ops.as_slice() else {
            panic!("first actual element")
        };
        let [Op::Element(last_body)] = last.region.ops.as_slice() else {
            panic!("second actual element")
        };
        assert!(last.condition.is_none());
        let Some(crate::expr::ExprRef::Js(condition)) = first.condition else {
            panic!("original condition")
        };
        assert!(record.input().matches_expression(condition));
        assert_eq!(record.scope(), unit.scope);
        let resolution = file.expression(node).unwrap();
        assert_eq!(resolution.scope(), Some(unit.scope));
        let [read] = resolution.table().unwrap().occurrences() else {
            panic!("one read")
        };
        assert_eq!(read.usage, crate::resolution::Usage::Read);
        assert_eq!(record.binding(), Some(read.binding));
        assert_eq!(
            file.binding(read.binding)
                .unwrap()
                .declaration()
                .unwrap()
                .boolean_initializer(),
            Some(expected)
        );
        let otherwise = record.else_input().unwrap();
        assert_eq!(otherwise.operand().name_span().slice(source), "v-else");
        assert_eq!(otherwise.operand().element_span(), last.span);
        assert_eq!(first.span.end, last.span.start);
        assert_eq!(original.span, Span::new(first.span.start, last.span.end));
        assert_eq!(first_body.span, first.span);
        assert_eq!(last_body.span, last.span);
        assert!(first_body.attributes.is_empty() && first_body.bindings.is_empty());
        assert!(last_body.attributes.is_empty() && last_body.bindings.is_empty());
        let mut last_id = None;
        file.artifact()
            .visit_nodes(&mut |id, actual| {
                if let crate::walk::NodeRef::Op(Op::Element(element)) = actual
                    && core::ptr::eq(element.as_ref(), last_body.as_ref())
                {
                    last_id = Some(id);
                }
            })
            .unwrap();
        let last_id = last_id.unwrap();
        assert_eq!(otherwise.body_node(), Some(last_id));
        assert!(otherwise.accepts_body(last_id, last_body));
        assert!(!otherwise.accepts_body(last_id, first_body));
        let selected = output.selected();
        let second = selected.children().nth(1).unwrap().into_element().unwrap();
        assert!(
            otherwise
                .operand()
                .admitted_for(selected, second.attributes().next().unwrap())
                .is_some()
        );
        assert!(core::ptr::eq(
            otherwise.operand().source_block().root_source(),
            source
        ));
        assert!(file.native_interpolations().is_empty());
    }
}

#[test]
fn pair_tail_preserves_ordinary_siblings_and_existing_single_if_text_dispatch() {
    let arena = Allocator::default();
    let mut owner = owner(&arena, "<template><p>one</p><b>two</b></template>");
    {
        let mut walk = owner.begin().unwrap();
        let mut children = walk.selected().children();
        let first = children.next().unwrap();
        walk.child_with_tail(first, &mut children).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(walk.cursor, 1);
        let last = children.next().unwrap();
        walk.child_with_tail(last, &mut children).unwrap();
        walk.complete().unwrap();
    }
    let output = owner.finish();
    let file = output.file().unwrap();
    let [Op::Element(_), Op::Element(_)] = file.artifact().root().ops.as_slice() else {
        panic!("unchanged siblings")
    };
    assert!(file.native_conditionals().is_empty() && file.is_complete());
    let text = completed(
        &arena,
        "<script setup>let ready=true</script><template><div v-if='ready'>{{ready}}</div></template>",
    );
    assert!(text.view().is_ok());
    let [record] = text.file().unwrap().native_conditionals() else {
        panic!("single If")
    };
    assert!(record.else_input().is_none());
}

#[test]
fn paired_text_nested_mixed_headers_and_nonadjacent_roots_stay_lower_refusals() {
    for template in [
        "<p v-if='ready'>{{ready}}</p><b v-else/>",
        "<p v-if='ready'/><b v-else>{{ready}}</b>",
        "<p v-if='ready'/><b v-else><i/></b>",
        "<p v-if='ready'/><b v-else>a b</b>",
        "<p v-if='ready'/><b v-else>&amp;</b>",
        "<p v-if='ready'/><b v-else><!--kept--></b>",
        "<p v-if='ready' title='x'/><b v-else/>",
        "<p v-if='ready'/><b v-else title='x'/>",
        "<p v-if='ready'/><b v-else='ready'/>",
        "<p v-if='ready'/><b v-else.foo/>",
        "<p v-if='ready'/><b v-else-if='ready'/>",
        "<p v-if='ready'/><b/>",
        "<p v-if='ready'/> <b v-else/>",
        "<p v-if='ready'/><!--kept--><b v-else/>",
        "<p v-if='ready'/><b v-else/><i/>",
    ] {
        let arena = Allocator::default();
        let source =
            alloc::format!("<script setup>let ready=true</script><template>{template}</template>");
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let mut children = walk.selected().children();
            let first = children.next().unwrap();
            assert_eq!(
                walk.child_with_tail(first, &mut children).unwrap_err().kind,
                Kind::UnsupportedChild,
                "{template}"
            );
            assert_eq!(walk.cursor, 0);
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err(), "{template}");
        let records = if let Some(file) = output.file() {
            assert!(!file.is_complete());
            file.native_conditionals()
        } else {
            output.rejected_file().unwrap().native_conditionals()
        };
        let [record] = records else {
            panic!("whole original conditional")
        };
        assert_eq!(
            record.state(),
            NativeFileConditionalState::Refused(Kind::UnsupportedChild)
        );
        assert!(record.original.is_none());
        assert!(core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            source.as_str()
        ));
    }
}

#[test]
fn both_original_headers_are_parked_before_the_actual_boolean_initializer_policy() {
    for initializer in ["1", "null", "'true'", "2n"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>const ready={initializer}</script><template><p v-if='ready'/><b v-else/></template>"
        );
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let mut children = walk.selected().children();
            let first = children.next().unwrap();
            assert_eq!(
                walk.child_with_tail(first, &mut children).unwrap_err().kind,
                Kind::UnsupportedChild
            );
            assert_eq!(walk.cursor, 0);
            let [record] = walk.root.facts.native_conditionals() else {
                panic!("actual If header")
            };
            assert!(record.binding().is_none() && record.original.is_none());
            let otherwise = record.else_input().unwrap();
            assert!(otherwise.body_node().is_none());
            assert_eq!(otherwise.operand().name_span().slice(&source), "v-else");
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        let file = output.file().unwrap();
        assert!(output.view().is_err() && !file.is_complete());
        assert_eq!(file.artifact().node_count(), 0);
    }
}

#[test]
fn foreign_or_reordered_original_tail_never_advances_the_root_cursor() {
    let arena = Allocator::default();
    let source =
        "<script setup>let ready=true</script><template><p v-if='ready'/><b v-else/></template>";
    let foreign = owner(&arena, source);
    let mut own = owner(&arena, source);
    {
        let mut walk = own.begin().unwrap();
        let first = walk.selected().children().next().unwrap();
        let mut tail = foreign.selected().children();
        tail.next().unwrap();
        assert_eq!(
            walk.child_with_tail(first, &mut tail).unwrap_err().kind,
            Kind::InvalidEvent
        );
        assert_eq!(walk.cursor, 0);
        assert!(walk.complete().is_err());
    }
    assert!(own.finish().view().is_err());
    let mut own = owner(&arena, source);
    {
        let mut walk = own.begin().unwrap();
        let mut tail = walk.selected().children();
        let wrong = tail.nth(1).unwrap();
        assert_eq!(
            walk.child_with_tail(wrong, &mut tail).unwrap_err().kind,
            Kind::InvalidEvent
        );
        assert_eq!(walk.cursor, 0);
        assert!(walk.complete().is_err());
    }
    assert!(own.finish().view().is_err());
}

#[test]
fn equal_pair_ids_and_spans_do_not_exchange_if_or_else_allocation_receipts() {
    let arena = Allocator::default();
    let source =
        "<script setup>let ready=true</script><template><p v-if='ready'/><b v-else/></template>";
    let own = completed(&arena, source);
    let foreign = completed(&arena, source);
    let [record] = own.file().unwrap().native_conditionals() else {
        panic!("own")
    };
    let [other] = foreign.file().unwrap().native_conditionals() else {
        panic!("foreign")
    };
    assert_eq!(record.state(), other.state());
    let [Op::If(other_if)] = foreign.file().unwrap().artifact().root().ops.as_slice() else {
        panic!("foreign If")
    };
    assert!(!record.accepts(other_if));
    let [_, last] = other_if.branches.as_slice() else {
        panic!("foreign branch")
    };
    let [Op::Element(body)] = last.region.ops.as_slice() else {
        panic!("foreign element")
    };
    assert!(
        !record
            .else_input()
            .unwrap()
            .accepts_body(other.else_input().unwrap().body_node().unwrap(), body)
    );
}

#[test]
fn caught_pair_unwind_retains_both_original_headers_and_no_completed_pair_receipt() {
    for point in 1..=5 {
        let arena = Allocator::default();
        let source = "<script setup>let ready=true</script><template><p v-if='ready'>yes</p><b v-else>no</b></template>";
        let mut owner = owner(&arena, source);
        {
            let mut walk = owner.begin().unwrap();
            let mut children = walk.selected().children();
            let first = children.next().unwrap();
            FAULT.with(|fault| fault.set(point));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || walk.child_with_tail(first.reborrow(), &mut children)
                ))
                .is_err()
            );
            assert_eq!(walk.cursor, 0);
            assert_eq!(walk.root.facts.scopes.len(), 2);
            let [record] = walk.root.facts.native_conditionals() else {
                panic!("parked If")
            };
            assert_eq!(record.state(), NativeFileConditionalState::Pending);
            assert!(record.original.is_none());
            assert_eq!(
                record
                    .else_input()
                    .unwrap()
                    .operand()
                    .name_span()
                    .slice(source),
                "v-else"
            );
            assert_eq!(
                record.else_input().unwrap().body_node().is_some(),
                point >= 4
            );
            assert_eq!(
                walk.child_with_tail(first, &mut children).unwrap_err().kind,
                Kind::Interrupted
            );
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err());
        let records = if let Some(file) = output.file() {
            assert!(!file.is_complete());
            file.native_conditionals()
        } else {
            output.rejected_file().unwrap().native_conditionals()
        };
        let [record] = records else {
            panic!("retained pair owners")
        };
        assert_eq!(record.state(), NativeFileConditionalState::Pending);
        let selected = output.selected();
        let second = selected.children().nth(1).unwrap().into_element().unwrap();
        assert!(
            record
                .else_input()
                .unwrap()
                .operand()
                .admitted_for(selected, second.attributes().next().unwrap())
                .is_some()
        );
    }
}
