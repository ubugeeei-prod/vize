use super::*;
use core::cell::Cell;
use vize_l0::String;

#[test]
fn original_same_buffer_foreign_descriptor_has_no_template_name_membership() {
    let source = "<template>same</template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let foreign = Vue.observe_descriptor(&arena, source, options());
    let view = owner.admitted().unwrap().template().unwrap();
    let other = foreign.admitted().unwrap().template().unwrap();
    let names = view.frame_names().unwrap();
    assert_eq!(view.block(), other.block());
    assert_eq!(names.container_index(), other.container_index());
    assert!(names.accepts(&view));
    assert!(!names.accepts(&other));
    assert!(other.frame_names().unwrap().accepts(&other));
}

#[test]
fn original_equal_owned_source_contents_never_replace_descriptor_identity() {
    let source = String::from("<template>same</template>");
    let equal = String::from(source.as_str());
    assert_ne!(source.as_ptr(), equal.as_ptr());
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, &source, options());
    let foreign = Vue.observe_descriptor(&arena, &equal, options());
    let view = owner.admitted().unwrap().template().unwrap();
    let other = foreign.admitted().unwrap().template().unwrap();
    assert_eq!(view.block(), other.block());
    let names = view.frame_names().unwrap();
    assert_eq!(names.opening(), other.frame_names().unwrap().opening());
    assert!(!names.accepts(&other));
    assert_eq!(names.source_block().source().as_ptr(), source.as_ptr());
}

#[test]
fn a_foreign_original_row_with_equal_index_and_buffer_cannot_forge_this_view() {
    let source = "<style>.x{}</style><script>const x=1</script><template>x</template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let foreign = Vue.observe_descriptor(&arena, source, options());
    let genuine = owner.admitted().unwrap().template().unwrap();
    let other = foreign.admitted().unwrap().template().unwrap();
    assert_eq!(genuine.container_index(), 2);
    assert_eq!(other.container_index(), 2);
    let forged = TemplateView {
        owner: &owner,
        selected: other.selected,
    };
    assert_eq!(
        forged.frame_names().unwrap_err(),
        NativeTemplateFrameNameRefusal::SourceMismatch
    );
    assert!(!genuine.frame_names().unwrap().accepts(&forged));
}

#[test]
fn original_owner_move_reborrow_and_query_order_do_not_allocate_or_change_names() {
    let source = "<template><p>x</p></template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let moved = (owner,);
    let before = arena.allocated_bytes();
    let expected = (Span::new(1, 9), Span::new(20, 28));
    for _ in 0..64 {
        let view = moved.0.admitted().unwrap().template().unwrap();
        let first = view.frame_names().unwrap();
        let second = moved
            .0
            .admitted()
            .unwrap()
            .template()
            .unwrap()
            .frame_names()
            .unwrap();
        assert_eq!((first.opening(), second.closing()), expected);
        assert_eq!((second.opening(), first.closing()), expected);
        assert!(first.accepts(&second.template()));
    }
    assert_eq!(arena.allocated_bytes(), before);
}

struct TrackedSource<'c> {
    value: String,
    drops: &'c Cell<u32>,
}

impl Drop for TrackedSource<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn original_source_drops_normally_only_after_descriptor_and_frame_scope_end() {
    let drops = Cell::new(0);
    let source = TrackedSource {
        value: String::from("<template>x</template>"),
        drops: &drops,
    };
    let arena = Allocator::default();
    {
        let owner = Vue.observe_descriptor(&arena, &source.value, options());
        let view = owner.admitted().unwrap().template().unwrap();
        let names = view.frame_names().unwrap();
        assert_eq!(
            names.source_block().source().as_ptr(),
            source.value.as_ptr()
        );
        assert_eq!(drops.get(), 0);
        assert_eq!(names.closing(), Span::new(13, 21));
    }
    assert_eq!(drops.get(), 0);
    drop(source);
    assert_eq!(drops.get(), 1);
}

#[test]
fn corrupted_private_geometry_refuses_without_inferring_a_new_name() {
    let source = "<template>x</template>";
    let arena = Allocator::default();
    let mut owner = Vue.observe_descriptor(&arena, source, options());
    owner
        .template
        .as_mut()
        .unwrap()
        .names
        .as_mut()
        .unwrap()
        .closing = Span::new(0, 8);
    let view = owner.admitted().unwrap().template().unwrap();
    assert_eq!(
        view.frame_names().unwrap_err(),
        NativeTemplateFrameNameRefusal::SourceMismatch
    );
}
