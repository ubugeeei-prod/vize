//! Compare both complete target decisions and their existing L4 writers.
#![expect(
    clippy::unwrap_used,
    reason = "fixture owners and oracle comparisons fail on unexpected admission"
)]
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorOptions, ScriptRole},
    },
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::{SelectedTemplateComponent, SurfaceParseOptions},
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l3::decision::{
    build_client_server_decisions,
    ssr::{TemplateSsrAnalysis, build_template_ssr_decisions},
    template::{TemplateDomAnalysis, build_template_dom_decisions},
};
use vize_l4::{
    targets::{dom, ssr},
    write::{EmitDocument, NoLinks, Recorded},
};

mod profile;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn original<'a>(arena: &'a Allocator, source: &'a str) -> Option<NativeTemplateFile<'a>> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().ok()?;
    let selected = SelectedTemplateComponent::parse_in(arena, admitted).ok()??;
    let mut owner = NativeTemplateOwner::new(selected).ok()?;
    for script in [admitted.ordinary(), admitted.setup()]
        .into_iter()
        .flatten()
    {
        let block = script.block();
        let syntax = parse_program_once(
            arena,
            EmbedSource::authored(source, block.span()).ok()?,
            ProgramOptions::module(script.lang()),
        );
        let program = syntax.admitted_program()?;
        match script.role() {
            ScriptRole::Ordinary => owner.ordinary_program(program),
            ScriptRole::Setup => owner.setup_program(program),
        }
        .ok()?;
    }
    if let Ok(mut walk) = owner.begin() {
        let selected = walk.selected();
        if selected
            .children()
            .into_iter()
            .all(|child| walk.child(child).is_ok())
        {
            let _ = walk.complete();
        }
    }
    Some(owner.finish())
}

fn dom_product(input: &TemplateDomAnalysis<'_, '_>) -> Result<EmitDocument, dom::DomError> {
    dom::emit_template::<Recorded>(input).map(|writer| writer.finish().into_document())
}
fn ssr_product(input: &TemplateSsrAnalysis<'_, '_>) -> Result<EmitDocument, ssr::SsrError> {
    ssr::emit_template::<Recorded>(input).map(|writer| writer.finish().into_document())
}

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    missing: usize,
    refused: usize,
    complete: usize,
    dom: usize,
    ssr: usize,
    code: usize,
    links: usize,
}

fn compare(owner: &NativeTemplateFile<'_>, counts: &mut Counts) {
    let actual = build_client_server_decisions(owner);
    let view = match owner.view() {
        Ok(view) => view,
        Err(issue) => {
            assert_eq!(actual.err(), Some(issue));
            counts.refused += 1;
            return;
        }
    };
    counts.complete += 1;
    let actual = actual.unwrap();
    let old_dom = build_template_dom_decisions(view);
    let old_ssr = build_template_ssr_decisions(owner.view().unwrap());
    assert_eq!(old_dom.as_ref().err().copied(), actual.dom().err());
    assert_eq!(old_ssr.as_ref().err().copied(), actual.ssr().err());
    if let (Ok(old), Ok(new)) = (&old_dom, actual.dom()) {
        assert!(core::ptr::eq(old.owner(), new.owner()) && core::ptr::eq(old.l2ir(), new.l2ir()));
        assert_eq!(old.tables(), new.tables());
        assert!(core::ptr::eq(
            old.original_attributes().unwrap().l2ir(),
            new.original_attributes().unwrap().l2ir()
        ));
        assert_eq!(
            old.original_attributes().map(|v| v.len()),
            new.original_attributes().map(|v| v.len())
        );
        assert_eq!(dom_product(old), dom_product(new));
        let expected =
            dom::emit_template::<NoLinks>(old).map(|writer| writer.finish().into_document());
        assert_eq!(
            expected,
            dom::emit_template::<NoLinks>(new).map(|writer| writer.finish().into_document())
        );
        if let Ok(product) = dom_product(new) {
            counts.dom += 1;
            counts.code += product.as_str().len();
            counts.links += product.links().len();
        }
    }
    if let (Ok(old), Ok(new)) = (&old_ssr, actual.ssr()) {
        assert!(core::ptr::eq(old.owner(), new.owner()) && core::ptr::eq(old.l2ir(), new.l2ir()));
        assert_eq!(
            old.original_attributes().map(|v| v.len()),
            Some(new.original_attributes().len())
        );
        assert_eq!(
            vize_l0::cstr!("{:?}", old.ssr().unwrap()),
            vize_l0::cstr!("{:?}", new.facts())
        );
        assert!(core::ptr::eq(
            old.original_attributes().unwrap().l2ir(),
            new.original_attributes().l2ir()
        ));
        let actual_product =
            ssr::emit_render_analysis::<Recorded>(new).map(|w| w.finish().into_document());
        assert_eq!(ssr_product(old), actual_product);
        let expected =
            ssr::emit_template::<NoLinks>(old).map(|writer| writer.finish().into_document());
        assert_eq!(
            expected,
            ssr::emit_render_analysis::<NoLinks>(new).map(|writer| writer.finish().into_document())
        );
        if let Ok(product) = actual_product {
            counts.ssr += 1;
            counts.code += product.as_str().len();
            counts.links += product.links().len();
        }
    }
}

#[test]
fn original_attributes_target_specific_bindings_and_styled_refusals() {
    let mut counts = Counts::default();
    for source in [
        "<template><div/></template>",
        "<template><main id='雪'><b title='a&amp;b'>hello</b><i/></main></template>",
        "<template><main @click='run'><i :class='name'>{{ value }}</i></main></template>",
        "<template><b v-if='show'>ok</b><i v-else>no</i></template>",
        "<template><span class='x'/></template><style>.x{}</style>",
        "<template><i v-for='x in xs'>{{ x }}</i></template>",
        "<template><i/><b/></template>",
    ] {
        let arena = Allocator::default();
        let owner = original(&arena, source).unwrap();
        compare(&owner, &mut counts);
    }
    assert!(counts.dom > 0 && counts.ssr > 0 && counts.links > 0);
}

#[test]
#[ignore = "NUL-delimited original Vue files"]
fn tracked_originals() {
    let manifest = std::fs::read(std::env::var_os("VIZE_CLIENT_SERVER_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Counts::default();
    let mut invalid = 0;
    for entry in manifest.split(|byte| *byte == 0).filter(|p| !p.is_empty()) {
        let path = root.join(std::str::from_utf8(entry).unwrap());
        let bytes = std::fs::read(&path).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        counts.files += 1;
        let arena = Allocator::default();
        let selected =
            vize_l1_to_l2::native_file::lower_selected_sfc_native(&arena, source, options());
        if let Some(owner) = selected.template() {
            compare(owner, &mut counts);
            if std::env::var_os("VIZE_CLIENT_SERVER_TIMINGS").is_some() && owner.view().is_ok() {
                profile::measure(path.to_str().unwrap(), owner);
            }
        } else {
            counts.missing += 1;
        }
    }
    eprintln!("client_server {counts:?} invalid_utf8={invalid}");
    assert!(counts.files > 0 && counts.files == counts.complete + counts.refused + counts.missing);
    if std::env::var_os("VIZE_CLIENT_SERVER_REQUIRE_OUTPUT").is_some() {
        assert!(counts.dom > 0 && counts.ssr > 0 && counts.links > 0);
    }
}
