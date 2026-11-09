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
fn unsupported_alias_shapes_and_reserved_prefixes_keep_their_earliest_original_refusal() {
    use vize_l1::embed::syntax::NativeForRefusal;

    for (aliases, syntax_refusal) in [
        ("([item],key)", Some(NativeForRefusal::AliasShape)),
        ("({item},key)", Some(NativeForRefusal::AliasShape)),
        ("(item=1,key)", Some(NativeForRefusal::AliasShape)),
        ("(item,key,third)", Some(NativeForRefusal::AliasCount(3))),
        ("(item,_key)", None),
        ("(item,$key)", None),
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
        let operand = match (rejected, syntax_refusal) {
            ([vize_l2::file::RejectedFileFor::Syntax(input)], Some(expected)) => {
                assert_eq!(input.kind, expected, "{aliases}");
                assert_eq!(input.span, input.operand().value_span());
                assert_eq!(input.operand().syntax().native_refusal(), Some(expected));
                assert!(input.operand().syntax().aliases().unwrap().is_ok());
                assert!(input.operand().syntax().collection().unwrap().is_ok());
                input.operand()
            }
            ([vize_l2::file::RejectedFileFor::Resolution { input, error }], None) => {
                assert_eq!(error.part, vize_l1::embed::syntax::ForHeadPart::Aliases);
                assert_eq!(
                    error.kind,
                    vize_l2::resolution::ForResolutionErrorKind::ReservedAlias
                );
                assert_eq!(input.aliases().len(), 2);
                input.operand()
            }
            _ => panic!("exact normally retained original alias refusal: {aliases}: {rejected:?}"),
        };
        assert_eq!(operand.raw_value(), alloc::format!("{aliases} in count"));
        assert_eq!(operand.value_span().slice(&source), operand.raw_value());
        assert!(core::ptr::eq(
            operand.syntax().source().authored_root(),
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
