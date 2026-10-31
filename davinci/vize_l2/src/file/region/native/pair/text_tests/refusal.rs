//! Late policy/refusal and unwind retain genuine prefix custody, not output.

use super::*;

#[test]
fn each_branch_still_refuses_non_bare_unowned_or_mixed_original_bodies() {
    for body in [
        "{{1}}",
        "{{true}}",
        "{{null}}",
        "{{'literal'}}",
        "{{message+message}}",
        "{{message()}}",
        "{{message.length}}",
        "{{missing}}",
        "{{[message]}}",
        "{{({message})}}",
        "{{m&#101;ssage}}",
        "{{message /* kept */}}",
        "{{message as string}}",
        "{{}}",
        "{{message +}}",
        "prefix{{message}}",
        "{{message}}suffix",
        "{{message}}{{message}}",
        "<i>{{message}}</i>",
    ] {
        for first in [true, false] {
            let arena = Allocator::default();
            let (before, after) = if first {
                (body, "{{message}}")
            } else {
                ("{{message}}", body)
            };
            let source = alloc::format!(
                "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>{before}</p><b v-else>{after}</b></template>"
            );
            let mut owner = owner(&arena, &source);
            {
                let mut walk = owner.begin().unwrap();
                let mut tail = walk.selected().children();
                let original = tail.next().unwrap();
                assert!(walk.child_with_tail(original, &mut tail).is_err(), "{body}");
                assert_eq!(walk.cursor, 0);
                assert!(walk.complete().is_err());
            }
            let output = owner.finish();
            assert!(output.view().is_err() && output.setup().is_err(), "{body}");
            let (rows, conditionals) = if let Some(file) = output.file() {
                assert!(!file.is_complete());
                (file.native_interpolations(), file.native_conditionals())
            } else {
                let rejected = output.rejected_file().unwrap();
                (
                    rejected.native_interpolations(),
                    rejected.native_conditionals(),
                )
            };
            let [conditional] = conditionals else {
                panic!("retained original pair")
            };
            assert!(matches!(
                conditional.state(),
                NativeFileConditionalState::Refused(_)
            ));
            assert!(conditional.original.is_none());
            assert!(conditional.else_input().is_some());
            let rejected_construct = match body {
                "prefix{{message}}"
                | "{{message}}suffix"
                | "{{message}}{{message}}"
                | "<i>{{message}}</i>" => "{{message}}",
                original => original,
            };
            let expected_constructs = if first {
                [rejected_construct, "{{message}}"]
            } else {
                ["{{message}}", rejected_construct]
            };
            for (index, row) in rows.iter().enumerate() {
                assert!(core::ptr::eq(
                    row.input().operand().syntax().source().authored_root(),
                    source.as_str()
                ));
                let construct = row.input().operand().full_span().slice(&source);
                assert_eq!(construct, *expected_constructs.get(index).unwrap());
            }
        }
    }
}

#[test]
fn late_second_policy_refusal_keeps_the_admitted_first_prefix_and_both_actual_inputs() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>{{left}}</p><b v-else>{{right+right}}</b></template>";
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
    let (rows, conditionals) = if let Some(file) = output.file() {
        assert!(!file.is_complete());
        let [first, last] = file.native_interpolations() else {
            panic!("two inputs")
        };
        let State::Admitted(node) = first.state() else {
            panic!("genuine prefix")
        };
        assert!(core::ptr::eq(
            first,
            file.native_interpolation(node).unwrap()
        ));
        assert!(matches!(
            last.state(),
            State::Refused(Kind::UnsupportedChild)
        ));
        (file.native_interpolations(), file.native_conditionals())
    } else {
        let rejected = output.rejected_file().unwrap();
        (
            rejected.native_interpolations(),
            rejected.native_conditionals(),
        )
    };
    let [first, last] = rows else {
        panic!("whole two original input owners")
    };
    assert!(matches!(first.state(), State::Admitted(_)));
    assert_eq!(last.state(), State::Refused(Kind::UnsupportedChild));
    assert_eq!(
        first.input().operand().full_span().slice(source),
        "{{left}}"
    );
    assert_eq!(
        last.input().operand().full_span().slice(source),
        "{{right+right}}"
    );
    assert!(!core::ptr::eq(
        first.input().operand().syntax().expression().unwrap(),
        last.input().operand().syntax().expression().unwrap()
    ));
    let [conditional] = conditionals else {
        panic!("one original pair")
    };
    assert_eq!(
        conditional.state(),
        NativeFileConditionalState::Refused(Kind::UnsupportedChild)
    );
    assert!(conditional.original.is_none());
    assert!(conditional.else_input().unwrap().body_node().is_none());
}

#[test]
fn caught_late_pair_unwind_keeps_admitted_queryable_rows_but_denies_complete_selected_view() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true;let left='text';let right=2</script><template><p v-if='ready'>{{left}}</p><b v-else>{{right}}</b></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let mut tail = walk.selected().children();
        let first = tail.next().unwrap();
        // Real close has completed the canonical If; outer admission has not.
        super::super::tests::set_fault(5);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                walk.child_with_tail(first.reborrow(), &mut tail)
            }))
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
    assert!(output.retained_setup().is_some());
    // File row queries intentionally keep admitted prefixes on incomplete Files.
    let file = output
        .file()
        .expect("normally closed canonical artifact remains retained");
    assert!(!file.is_complete() && file.template_interruption().is_some());
    let [first, last] = file.native_interpolations() else {
        panic!("two genuine prefixes")
    };
    for row in [first, last] {
        let State::Admitted(node) = row.state() else {
            panic!("real admitted prefix")
        };
        assert!(core::ptr::eq(row, file.native_interpolation(node).unwrap()));
        let resolution = file.expression(node).unwrap();
        assert_eq!(
            resolution.scope(),
            Some(file.units().first().unwrap().scope)
        );
        assert!(core::ptr::eq(
            resolution.table().unwrap().expression().ast,
            row.input().operand().syntax().expression().unwrap()
        ));
    }
    let [conditional] = file.native_conditionals() else {
        panic!("pending original pair")
    };
    assert_eq!(conditional.state(), NativeFileConditionalState::Pending);
    assert!(conditional.original.is_none());
    let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
        panic!("real closed If")
    };
    assert!(!conditional.accepts(original));
    assert!(conditional.else_input().unwrap().body_node().is_some());
}
