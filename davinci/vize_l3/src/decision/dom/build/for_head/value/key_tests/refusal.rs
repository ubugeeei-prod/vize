use super::*;

#[test]
fn key_heads_keep_static_empty_and_value_only_whole_for_shape_refusals() {
    // The first two whole inputs are the existing immutable mutable/static and
    // const/value counterexamples. Neither is promoted by the key-only route.
    for source in [
        "<script setup>let count=2</script><template><i v-for='(item,key) in count'>fixed</i></template>",
        "<script setup>const count=2</script><template><i v-for='(item, index) in count'>{{item}}</i></template>",
        "<script setup>let count=2</script><template><i v-for='(item,key) in count'></i></template>",
        "<script setup>const count=2</script><template><i v-for='(item,key) in count'>fixed</i></template>",
        "<script setup>let count=2</script><template><i v-for='(item,key) in count'>{{item}}</i></template>",
    ] {
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, options());
        let view = observation.admitted().unwrap();
        let file = view.setup().file();
        let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            panic!("one whole original For");
        };
        let analysis = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
        let dom = analysis.dom().unwrap();
        let [rejected] = dom.unsupported() else {
            panic!("one sticky original ForShape: {source}");
        };
        assert_eq!(rejected.reason, DomUnsupported::ForShape, "{source}");
        assert_eq!(rejected.node, original.id().node());
        assert_eq!(rejected.span, original.span);
        assert!(!dom.node(original.id().node()).unwrap().block_eligible);
        assert!(
            dom.file_for_head(original.id().node())
                .unwrap()
                .accepts_original(original)
        );
    }
}

#[test]
fn key_only_completion_does_not_admit_compound_mixed_grouped_nested_or_attributed_bodies() {
    for template in [
        "<i v-for='(item,key) in count'>{{key + 1}}</i>",
        "<i v-for='(item,key) in count'>{{(key)}}</i>",
        "<i v-for='(item,key) in count'>fixed{{key}}</i>",
        "<i v-for='(item,key) in count'>{{key}}{{key}}</i>",
        "<i v-for='(item,key) in count'><b>{{key}}</b></i>",
        "<i v-for='(item,key) in count'><b v-for='(child,inner) in item'>{{inner}}</b></i>",
        "<i title='fixed' v-for='(item,key) in count'>{{key}}</i>",
        "<i v-for='(item,key) in count'>{{count}}</i>",
        "<i v-for='(item,key) in count'>{{key}}</i><b/>",
        "<b><i v-for='(item,key) in count'>{{key}}</i></b>",
    ] {
        let arena = Allocator::default();
        let source =
            alloc::format!("<script setup>let count=2</script><template>{template}</template>");
        let observation = lower_selected_setup_sfc_native(&arena, &source, options());
        let view = observation.admitted().unwrap();
        let analysis = build_native_selected_setup_dom_decisions(view.setup()).unwrap();
        assert!(
            !analysis.dom().unwrap().unsupported().is_empty(),
            "{template}"
        );
    }
}

#[test]
fn unsupported_alias_shapes_and_reserved_prefixes_keep_their_original_l2_refusal() {
    for (aliases, reserved) in [
        ("([item],key)", false),
        ("({item},key)", false),
        ("(item=1,key)", false),
        ("(item,key,third)", false),
        ("(item,_key)", true),
        ("(item,$key)", true),
    ] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>let count=2</script><template><i v-for='{aliases} in count'>{{{{key}}}}</i></template>"
        );
        let observation = lower_selected_setup_sfc_native(&arena, &source, options());
        assert!(observation.admitted().is_none(), "{aliases}");
        let owner = observation.original().template().unwrap();
        let rejected = owner
            .file()
            .map(|file| file.rejected_for_heads())
            .or_else(|| owner.rejected_file().map(|file| file.rejected_for_heads()))
            .unwrap();
        let [vize_l2::file::RejectedFileFor::Resolution { input, error }] = rejected else {
            panic!("normally retained original alias refusal: {aliases}");
        };
        assert_eq!(error.part, vize_l1::embed::syntax::ForHeadPart::Aliases);
        assert_eq!(
            error.kind,
            if reserved {
                vize_l2::resolution::ForResolutionErrorKind::ReservedAlias
            } else {
                vize_l2::resolution::ForResolutionErrorKind::UnsupportedAlias
            }
        );
        assert_eq!(
            input.operand().raw_value(),
            alloc::format!("{aliases} in count")
        );
        assert!(core::ptr::eq(
            input.operand().syntax().source().authored_root(),
            source.as_str()
        ));
        assert!(owner.retained_setup().is_some());
    }
}

#[test]
fn key_consumers_do_not_widen_original_collection_or_entity_policies() {
    for source in [
        "<script setup>const count=[1,2]</script><template><i v-for='(item,key) in count'>{{key}}</i></template>",
        "<script setup>const count={a:1}</script><template><i v-for='(item,key) in count'>{{key}}</i></template>",
        "<script setup>let count=2</script><template><i v-for='(item,k&#101;y) in count'>{{key}}</i></template>",
    ] {
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, options());
        assert!(observation.admitted().is_none(), "{source}");
        let owner = observation.original().template().unwrap();
        assert!(owner.retained_setup().is_some());
        assert!(core::ptr::eq(
            owner.selected().component().block().root_source(),
            source
        ));
    }
}
