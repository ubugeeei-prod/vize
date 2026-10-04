use super::*;
use crate::decision::dom::{DomUnsupported, LiteralExpressions};

#[test]
fn actual_constant_scope_uses_vnodes_and_rejects_a_counterfeit_mutable_access_kind() {
    struct WrongMode<'v, 'o, 'a>(&'v NativeSelectedSetup<'o, 'a>);
    impl<'o, 'a> FileReads<'o, 'a> for WrongMode<'_, 'o, 'a> {
        const RECORD: bool = true;
        const NATIVE_SETUP: bool = true;
        const ORIGINAL_FOR: bool = true;
        fn classify(
            &self,
            occurrence: &Occurrence<'a>,
            binding: BindingRef<'o, 'a>,
        ) -> Option<VueReadKind> {
            (occurrence.usage == Usage::Read && self.0.binding(binding).is_ok())
                .then_some(VueReadKind::SetupLet)
        }
    }
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='count in count'>{{count}}</i></template>";
    let observation = lower_selected_setup_sfc_native(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = observation.admitted().unwrap();
    let setup = view.setup();
    let file = setup.file();
    let [root @ Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("original");
    };
    let valid = crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
    let dom = valid.dom().unwrap();
    assert!(dom.unsupported().is_empty());
    let row = dom.file_for_head(original.id().node()).unwrap();
    assert_eq!(
        row.collection_read().unwrap().kind(),
        VueReadKind::SetupConst
    );
    assert!(setup.binding(row.collection()).is_ok());
    let alias = row.head().value().unwrap();
    assert_ne!(alias.id(), row.collection().id());
    assert!(setup.binding(alias).is_err());
    let foreign_observation = lower_selected_setup_sfc_native(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let foreign_view = foreign_observation.admitted().unwrap();
    let foreign_reads = WrongMode(foreign_view.setup());
    let mut foreign = DomBuilder::new(&LiteralExpressions, 1, Some(file), &foreign_reads);
    foreign.enter(original.id().node(), root, None).unwrap();
    assert_eq!(
        foreign.facts.unsupported[0].reason,
        DomUnsupported::ForCollectionRuntime
    );
    assert!(!foreign.frames[0].1.node.block_eligible);
    let wrong = WrongMode(setup);
    let mut builder = DomBuilder::new(&LiteralExpressions, 1, Some(file), &wrong);
    builder.enter(original.id().node(), root, None).unwrap();
    assert_eq!(
        builder.facts.unsupported[0].reason,
        DomUnsupported::ForCollectionRuntime
    );
    assert!(!builder.frames[0].1.node.block_eligible);
    assert!(
        builder
            .facts
            .file_for_heads
            .get(original.id().node())
            .unwrap()
            .accepts_original(original)
    );
}

#[test]
fn constant_enter_unwind_keeps_actual_collection_program_params_and_completed_file() {
    let arena = Allocator::default();
    let source = "<script setup>const count=2</script><template><i v-for='item in count'>fixed</i></template>";
    let observation = lower_selected_setup_sfc_native(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = observation.admitted().unwrap();
    let setup = view.setup();
    let file = setup.file();
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("original");
    };
    let resolution = file.for_head_for(original).unwrap().resolution().unwrap();
    let program = setup.program().program().body.as_ptr();
    let collection = resolution.input().collection() as *const _;
    let parameter = resolution.value_declaration().parameter() as *const _;
    let calls = Cell::new(0);
    let reads = InterruptedReads {
        setup,
        calls: &calls,
    };
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::decision::build::build_with(
            file.artifact(),
            crate::decision::policy::TargetPolicy::Dom,
            &LiteralExpressions,
            Some(file),
            &reads,
        )
    }));
    assert!(caught.is_err());
    assert_eq!(calls.get(), 1);
    assert!(file.is_complete());
    assert_eq!(program, setup.program().program().body.as_ptr());
    assert_eq!(collection, resolution.input().collection() as *const _);
    assert_eq!(
        parameter,
        resolution.value_declaration().parameter() as *const _
    );
    let valid = crate::decision::native::build_native_selected_setup_dom_decisions(setup).unwrap();
    assert!(valid.dom().unwrap().unsupported().is_empty());
}
