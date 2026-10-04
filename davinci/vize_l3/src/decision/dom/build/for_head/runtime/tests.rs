extern crate std;
use super::*;
use core::cell::Cell;
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::BindingRef,
    lang::js::NativeSelectedSetup,
    resolution::{Occurrence, Usage},
};

struct InterruptedReads<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    calls: &'view Cell<u32>,
}
impl<'owner, 'arena> FileReads<'owner, 'arena> for InterruptedReads<'_, 'owner, 'arena> {
    const RECORD: bool = true;
    const NATIVE_SETUP: bool = true;
    const ORIGINAL_FOR: bool = true;
    fn classify(
        &self,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        assert_eq!(occurrence.usage, Usage::Read);
        assert!(self.setup.binding(binding).is_ok());
        self.calls.set(self.calls.get() + 1);
        panic!("stop during authentic original For Enter classification")
    }
}

#[test]
fn caught_enter_unwind_keeps_normal_program_collection_params_and_alias_scope() {
    let arena = Allocator::default();
    let source = "<script setup>let count=2</script><template><i v-for='count in count'>fixed</i></template>";
    let observation = lower_selected_setup_sfc_native(&arena, source, Default::default());
    let view = observation.admitted().unwrap();
    let setup = view.setup();
    let file = setup.file();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let head = file.for_head_for(original).unwrap();
    let resolution = head.resolution().unwrap();
    let collection = resolution.input().collection() as *const _;
    let parameter = resolution.value_declaration().parameter() as *const _;
    let program = setup.program().program().body.as_ptr();
    let calls = Cell::new(0);
    let reads = InterruptedReads {
        setup,
        calls: &calls,
    };
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::decision::build::build_with(
            file.artifact(),
            crate::decision::policy::TargetPolicy::Dom,
            &crate::decision::dom::LiteralExpressions,
            Some(file),
            &reads,
        )
    }));
    assert!(caught.is_err());
    assert_eq!(calls.get(), 1);
    assert!(file.is_complete());
    assert!(head.accepts(original));
    assert_eq!(program, setup.program().program().body.as_ptr());
    assert_eq!(collection, resolution.input().collection() as *const _);
    assert_eq!(
        parameter,
        resolution.value_declaration().parameter() as *const _
    );
    assert!(head.scope().is_some());
    assert_ne!(head.scope(), head.enclosing_scope());
    let alias = head.value().unwrap();
    let collection = file.binding(resolution.collection().binding).unwrap();
    assert_eq!(
        alias.template_declaration().unwrap().declaration().scope(),
        head.scope().unwrap()
    );
    assert_ne!(alias.id(), collection.id());
    assert!(setup.binding(alias).is_err());
    assert!(setup.binding(collection).is_ok());
    let complete =
        crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
    let dom = complete.dom().unwrap();
    assert!(dom.unsupported().is_empty());
    assert!(dom.node(original.id().node()).unwrap().block_eligible);
    assert!(core::ptr::eq(
        dom.file_for_head(original.id().node())
            .unwrap()
            .resolution(),
        resolution
    ));
}
