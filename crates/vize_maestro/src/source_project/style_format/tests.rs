use super::*;
use crate::{source_project::SourceSnapshotCache, utils::position_to_offset};
use tower_lsp::lsp_types::Url;
use vize_glyph::FormatOptions;
use vize_l0::{
    SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}
#[test]
fn authentic_format_keeps_css_bindings_and_untouched_bytes_and_rechecks_revision() {
    let text = "\u{feff}<template>🦀</template>\r\n<style scoped lang='scss'>$color:red; .a{color:$color}</style>\r\n<!--尻--><style>.keep{}</style>";
    let uri = Url::parse("file:///Theme.vue").unwrap();
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    docs.open(uri.clone(), text.into(), 7, "vue".into());
    let first = cache.capture(&docs, &uri).unwrap();
    let arena = Allocator::default();
    let options_fmt = FormatOptions::default();
    let formatter = GlyphFormatter::new(&options_fmt, &arena);
    let descriptor = Vue.observe_descriptor(&arena, first.source(), options());
    let approved = first
        .verified_style_format(&docs, &descriptor, 0, &formatter)
        .unwrap();
    let edit = approved.publish(&docs).unwrap().unwrap();
    assert_eq!(edit.text_document.uri, uri);
    assert_eq!(edit.text_document.version, Some(7));
    let [OneOf::Left(edit)] = edit.edits.as_slice() else {
        panic!("one style edit")
    };
    let (from, to) = {
        let doc = docs.get(&uri).unwrap();
        let positions = doc.position_text();
        (
            position_to_offset(positions, edit.range.start).unwrap(),
            position_to_offset(positions, edit.range.end).unwrap(),
        )
    };
    let style = descriptor.admitted().unwrap().styles().next().unwrap();
    let selected = style.original_block().authored_span();
    assert_eq!((from, to), (selected.start as usize, selected.end as usize));
    let mut after = text.to_owned();
    after.replace_range(from..to, &edit.new_text);
    assert_eq!(
        after,
        [
            "\u{feff}<template>🦀</template>\r\n",
            edit.new_text.as_str(),
            "\r\n<!--尻--><style>.keep{}</style>"
        ]
        .concat()
    );
    let observed = Vue.observe_descriptor(&arena, &after, options());
    let next = observed.admitted().unwrap().styles().next().unwrap();
    let old = ScssProducts::from_style(style).unwrap();
    let new = ScssProducts::from_style(next).unwrap();
    assert_eq!(old.css().unwrap(), new.css().unwrap());
    assert_eq!(old.classes(), new.classes());
    assert!(matches!(
        first.verified_style_format(&docs, &observed, 0, &formatter),
        Err(StyleFormatError::ForeignSource)
    ));
    docs.open(uri.clone(), after.clone().into(), 8, "vue".into());
    assert!(matches!(
        approved.publish(&docs),
        Err(StyleFormatError::Host(SnapshotRefusal::Superseded))
    ));
    let fresh = cache.capture(&docs, &uri).unwrap();
    let refreshed = Vue.observe_descriptor(&arena, fresh.source(), options());
    assert!(
        fresh
            .verified_style_format(&docs, &refreshed, 0, &formatter)
            .unwrap()
            .publish(&docs)
            .unwrap()
            .is_none()
    );
}

#[test]
fn checker_rejects_changed_outputs_and_same_css_with_a_different_definition() {
    let read = |s| ScssProducts::read(SourceRoot::new(s).unwrap().whole_block());
    let before = read("$a:red;$b:red;.x{color:$a}");
    let changed = read("$a:red;$b:red;.x{color:blue}");
    assert!(matches!(
        before.check_equivalent(&changed),
        Err(ScssMismatch::Rules)
    ));
    let moved = read("$a:red;$b:red;.x{color:$b}");
    assert_eq!(before.css().unwrap(), moved.css().unwrap());
    assert!(matches!(
        before.check_equivalent(&moved),
        Err(ScssMismatch::Uses)
    ));
    let extra = read("$a:red;$b:red;$c:red;.x{color:$a}");
    assert_eq!(before.css().unwrap(), extra.css().unwrap());
    assert!(matches!(
        before.check_equivalent(&extra),
        Err(ScssMismatch::Definitions)
    ));
    let bad = read(".x{color:$missing}");
    assert!(matches!(
        before.check_equivalent(&bad),
        Err(ScssMismatch::Replacement(_))
    ));
}
