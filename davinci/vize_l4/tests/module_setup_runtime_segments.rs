//! Authentic selected source segments append into the existing target writer.

use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l4::{
    module::setup::{
        SetupEmitErrorKind, emit_selected_setup, write_selected_setup_runtime_segments,
    },
    runtime::{Runtime, vocabulary},
    write::{NoLinks, Recorded, Writer},
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn original_js_ts_segments_preserve_existing_writer_source_links_and_helpers()
-> Result<(), &'static str> {
    for (source, expected, chunks, annotations) in [
        (
            "<!--前雪--><script setup>/*whole*/ let count=1; // tail🌸</script><template>{{count}}</template>",
            "/*whole*/ let count=1; // tail🌸",
            vec!["/*whole*/ let count=1; // tail🌸"],
            0,
        ),
        (
            "<!--前雪--><script setup lang='ts'>/*whole雪*/ let count:number=1; const label:string='🌸'; // tail</script><template>{{count}}</template>",
            "/*whole雪*/ let count=1; const label='🌸'; // tail",
            vec!["/*whole雪*/ let count", "=1; const label", "='🌸'; // tail"],
            2,
        ),
    ] {
        let arena = Allocator::default();
        let owner =
            core::hint::black_box(lower_selected_setup_sfc_native(&arena, source, options()));
        let view = owner.admitted().ok_or("normally owned envelope")?;
        let setup = view.setup();
        let body = setup.program().program().body.as_ptr();
        assert!(core::ptr::eq(setup.source().root_source(), source));
        assert!(core::ptr::eq(
            setup.syntax(),
            setup.owner().retained_setup().ok_or("stock owner")?
        ));
        assert_eq!(setup.type_annotations().count(), annotations);
        let prefix = "/*caller*/\n";
        let mut recorded = Writer::<Recorded>::with_capacity(1024);
        recorded.push_linked(prefix, Span::new(0, "<!--前雪-->".len() as u32));
        let buffer = recorded.as_str().as_ptr();
        let helper = vocabulary(Runtime::VueDom)
            .helper("toDisplayString")
            .ok_or("real helper")?;
        recorded.use_helper(helper);
        let helpers = recorded.helpers().in_use_order().to_vec();
        write_selected_setup_runtime_segments(setup, &mut recorded)
            .map_err(|_| "runtime segments")?;
        assert_eq!(recorded.as_str().as_ptr(), buffer);
        assert_eq!(recorded.helpers().in_use_order(), helpers.as_slice());
        recorded.push("\n/*after*/");
        let output = recorded.finish();
        assert_eq!(
            output.text.as_str(),
            format!("{prefix}{expected}\n/*after*/")
        );
        assert_eq!(output.links.links().len(), chunks.len() + 1);
        let first = output.links.links().first().ok_or("retained caller link")?;
        assert_eq!(first.generated, Span::new(0, prefix.len() as u32));
        assert_eq!(first.authored, Span::new(0, "<!--前雪-->".len() as u32));
        let mut cursor = prefix.len() as u32;
        for (link, chunk) in output.links.links().iter().skip(1).zip(chunks) {
            let authored = source.find(chunk).ok_or("independent authored segment")? as u32;
            assert_eq!(
                link.authored,
                Span::new(authored, authored + chunk.len() as u32)
            );
            assert_eq!(
                link.generated,
                Span::new(cursor, cursor + chunk.len() as u32)
            );
            assert!(link.segment && link.name.is_none());
            cursor += chunk.len() as u32;
        }
        let mut plain = Writer::<NoLinks>::default();
        plain.push(prefix);
        plain.use_helper(helper);
        write_selected_setup_runtime_segments(setup, &mut plain).map_err(|_| "NoLinks segments")?;
        plain.push("\n/*after*/");
        let plain = plain.finish();
        assert_eq!(plain.text, output.text);
        assert_eq!(plain.helpers, output.helpers);
        assert!(plain.into_document().links().is_empty());
        assert_eq!(setup.program().program().body.as_ptr(), body);
        assert_eq!(setup.syntax().comments().count(), 2);
    }
    Ok(())
}

#[test]
fn runtime_segment_provider_preserves_distinct_dom_wrapper_collision_preflight()
-> Result<(), &'static str> {
    for name in ["Object", "__value", "__v_raw"] {
        let arena = Allocator::default();
        let source = format!("<script setup>let {name}=1</script><template></template>");
        let owner = lower_selected_setup_sfc_native(&arena, &source, options());
        let view = owner.admitted().ok_or("genuine direct binding")?;
        let setup = view.setup();
        let error = emit_selected_setup::<Recorded>(setup)
            .err()
            .ok_or("DOM wrapper collision")?;
        assert_eq!(error.kind, SetupEmitErrorKind::GeneratedBindingCollision);
        let mut writer = Writer::<Recorded>::default();
        writer.push("/*target owns names*/");
        write_selected_setup_runtime_segments(setup, &mut writer)
            .map_err(|_| "pure original segments")?;
        assert_eq!(
            writer.as_str(),
            format!("/*target owns names*/let {name}=1")
        );
    }
    Ok(())
}

#[test]
fn wider_programs_never_mint_the_segment_provider_input() {
    for source in [
        "<script setup>import value from 'pkg';const count=1</script><template></template>",
        "<script setup>const count=[1]</script><template></template>",
        "<script setup lang='ts'>let count:1=1</script><template></template>",
        "<script setup>const count=make()</script><template></template>",
        "<script setup>let count=1</script><script>export default{}</script><template></template>",
    ] {
        let arena = Allocator::default();
        let owner = lower_selected_setup_sfc_native(&arena, source, options());
        assert!(owner.admitted().is_none(), "{source}");
        assert!(core::ptr::eq(
            owner.original().descriptor().source(),
            source
        ));
    }
}
