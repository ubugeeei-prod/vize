//! The two-part profile never legalizes whitespace, entities or wider syntax.

use super::*;

#[test]
fn third_same_kind_nested_comment_whitespace_entity_and_non_bare_parts_stay_refused() {
    for (body, construct) in [
        ("prefix{{message}}suffix", "{{message}}"),
        ("{{message}}suffix{{message}}", "{{message}}"),
        ("{{message}}{{message}}", "{{message}}"),
        ("prefix<!--kept-->", "{{message}}"),
        ("<!--kept-->{{message}}", "{{message}}"),
        ("prefix<i/>", "{{message}}"),
        ("prefix {{message}}", "{{message}}"),
        ("{{message}} suffix", "{{message}}"),
        ("pre\t{{message}}", "{{message}}"),
        ("{{message}}\r\nsuffix", "{{message}}"),
        ("pre&amp;{{message}}", "{{message}}"),
        ("{{message}}&amp;", "{{message}}"),
        ("prefix{{message+message}}", "{{message+message}}"),
        ("{{message()}}suffix", "{{message()}}"),
        ("prefix{{missing}}", "{{missing}}"),
        ("{{1}}suffix", "{{1}}"),
        ("prefix{{m&#101;ssage}}", "{{m&#101;ssage}}"),
        ("{{message /* kept */}}suffix", "{{message /* kept */}}"),
    ] {
        for first in [true, false] {
            let arena = Allocator::default();
            let (left, right) = if first {
                (body, "{{message}}")
            } else {
                ("{{message}}", body)
            };
            let source = alloc::format!(
                "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>{left}</p><b v-else>{right}</b></template>"
            );
            let mut owner = owner(&arena, &source);
            {
                let mut walk = owner.begin().unwrap();
                let mut tail = walk.selected().children();
                let first = tail.next().unwrap();
                assert!(walk.child_with_tail(first, &mut tail).is_err(), "{body}");
                assert_eq!(walk.cursor, 0);
                assert!(walk.complete().is_err());
            }
            let output = owner.finish();
            assert!(output.view().is_err() && output.setup().is_err(), "{body}");
            let (records, conditionals) = if let Some(file) = output.file() {
                assert!(!file.is_complete());
                (file.native_interpolations(), file.native_conditionals())
            } else {
                let rejected = output.rejected_file().unwrap();
                (
                    rejected.native_interpolations(),
                    rejected.native_conditionals(),
                )
            };
            let expected = if first {
                [construct, "{{message}}"]
            } else {
                ["{{message}}", construct]
            };
            for (index, row) in records.iter().enumerate() {
                assert_eq!(
                    row.input().operand().full_span().slice(&source),
                    *expected.get(index).unwrap()
                );
                assert!(core::ptr::eq(
                    row.input().operand().syntax().source().authored_root(),
                    source.as_str()
                ));
            }
            let [conditional] = conditionals else {
                panic!("both original headers")
            };
            assert!(conditional.else_input().is_some() && conditional.original.is_none());
            assert!(matches!(
                conditional.state(),
                NativeFileConditionalState::Refused(_)
            ));
        }
    }
}

#[test]
fn late_second_and_third_parts_retain_real_admitted_prefixes_without_completing_the_pair() {
    for (source, second_refusal) in [
        (
            "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>prefix{{left}}</p><b v-else>suffix{{right+right}}</b></template>",
            true,
        ),
        (
            "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>prefix{{left}}suffix</p><b v-else>{{right}}</b></template>",
            false,
        ),
    ] {
        let arena = Allocator::default();
        let mut owner = owner(&arena, source);
        {
            let mut walk = owner.begin().unwrap();
            let mut tail = walk.selected().children();
            let first = tail.next().unwrap();
            assert_eq!(
                walk.child_with_tail(first, &mut tail).unwrap_err().kind,
                Kind::UnsupportedChild
            );
            assert_eq!(walk.cursor, 0);
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err() && output.setup().is_err());
        let (records, conditionals) = if let Some(file) = output.file() {
            assert!(!file.is_complete());
            let first = file.native_interpolations().first().unwrap();
            let State::Admitted(node) = first.state() else {
                panic!("actual prefix")
            };
            assert!(core::ptr::eq(
                first,
                file.native_interpolation(node).unwrap()
            ));
            (file.native_interpolations(), file.native_conditionals())
        } else {
            let rejected = output.rejected_file().unwrap();
            (
                rejected.native_interpolations(),
                rejected.native_conditionals(),
            )
        };
        let first = if second_refusal {
            let [first, second] = records else {
                panic!("two actual source inputs")
            };
            assert_eq!(second.state(), State::Refused(Kind::UnsupportedChild));
            assert_eq!(
                second.input().operand().full_span().slice(source),
                "{{right+right}}"
            );
            first
        } else {
            let [first] = records else {
                panic!("third child refused before its mint")
            };
            first
        };
        assert!(matches!(first.state(), State::Admitted(_)));
        assert_eq!(
            first.input().operand().full_span().slice(source),
            "{{left}}"
        );
        assert!(core::ptr::eq(
            first.input().operand().syntax().source().authored_root(),
            source
        ));
        let [conditional] = conditionals else {
            panic!("retained If/Else")
        };
        assert_eq!(
            conditional.state(),
            NativeFileConditionalState::Refused(Kind::UnsupportedChild)
        );
        assert!(conditional.original.is_none() && conditional.else_input().is_some());
    }
}

#[test]
fn caught_composite_pair_unwind_keeps_queryable_prefixes_and_denies_selected_output() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>prefix{{left}}</p><b v-else>{{right}}suffix</b></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let mut tail = walk.selected().children();
        let first = tail.next().unwrap();
        super::super::tests::set_fault(5);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || walk.child_with_tail(first.reborrow(), &mut tail)
            ))
            .is_err()
        );
        assert_eq!(walk.cursor, 0);
        assert_eq!(
            walk.child_with_tail(first.reborrow(), &mut tail)
                .unwrap_err()
                .kind,
            Kind::Interrupted
        );
        assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
    }
    let output = owner.finish();
    assert!(output.view().is_err() && output.setup().is_err());
    let file = output
        .file()
        .expect("real canonical If closed before outer admission");
    assert!(!file.is_complete() && file.template_interruption().is_some());
    let [first, last] = file.native_interpolations() else {
        panic!("two real prefixes")
    };
    for record in [first, last] {
        let State::Admitted(node) = record.state() else {
            panic!("completed original read")
        };
        assert!(core::ptr::eq(
            record,
            file.native_interpolation(node).unwrap()
        ));
        let row = file.expression(node).unwrap();
        assert!(core::ptr::eq(
            row.table().unwrap().expression().ast,
            record.input().operand().syntax().expression().unwrap()
        ));
    }
    let [conditional] = file.native_conditionals() else {
        panic!("pending outer pair")
    };
    assert_eq!(conditional.state(), NativeFileConditionalState::Pending);
    assert!(conditional.original.is_none());
    let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual closed If")
    };
    assert!(!conditional.accepts(original));
    assert_eq!(
        conditional.else_input().unwrap().body_node(),
        Some(node(
            file,
            original
                .branches
                .last()
                .unwrap()
                .region
                .ops
                .first()
                .unwrap()
        ))
    );
}
