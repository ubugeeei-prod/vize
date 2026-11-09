use super::*;

#[test]
fn boolean_proof_is_the_original_initializer_not_a_name_annotation_or_primitive_family() {
    for initializer in ["1", "null", "'true'", "2n"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>const ready={initializer}</script><template><div v-if='ready'/></template>"
        );
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let child = walk.selected().children().next().unwrap();
            assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        let file = output.file().unwrap();
        let mut bindings = file.bindings();
        let binding = bindings.next().unwrap();
        assert!(bindings.next().is_none());
        assert!(binding.declaration().unwrap().initializer.is_primitive());
        assert_eq!(binding.declaration().unwrap().boolean_initializer(), None);
        assert_eq!(file.artifact().node_count(), 0);
        assert!(!file.is_complete());
    }
}

#[test]
fn unsupported_condition_spelling_and_body_shapes_remain_whole_lower_refusals() {
    for template in [
        "<div v-if='ready && ready'/>",
        "<div v-if='(ready)'/>",
        "<div v-if='ready /* kept */'/>",
        "<div v-if=' ready '/>",
        "<div v-if='r&#101;ady'/>",
        "<div v-if='\\u0072eady'/>",
        "<div v-if='ready'><b/></div>",
        "<div v-if='ready'>a b</div>",
        "<div v-if='ready' title='value'/>",
        "<div v-if='ready' @click='ready'/>",
        "<div v-if='ready' v-if='ready'/>",
        "<div v-if='ready'/><p/>",
        "<div><p v-if='ready'/></div>",
        "<div v-if='ready' v-for='item in ready'/>",
        "<div v-else-if='ready'/>",
        "<div v-if='missing'/>",
    ] {
        let arena = Allocator::default();
        let source =
            alloc::format!("<script setup>let ready=true</script><template>{template}</template>");
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let mut refused = false;
            for child in walk.selected().children() {
                if walk.child(child).is_err() {
                    refused = true;
                    break;
                }
            }
            assert!(refused, "{template}");
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err(), "{template}");
        if let Some(file) = output.file() {
            assert!(!file.is_complete());
        }
        if let Some(file) = output.rejected_file() {
            assert!(core::ptr::eq(file.source(), source.as_str()));
            assert!(
                file.native_conditionals()
                    .iter()
                    .all(|row| !matches!(row.state(), NativeFileConditionalState::Admitted(_)))
            );
        }
    }
}
