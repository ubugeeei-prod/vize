//! Genuine selected root receipts reach the sole selected L4 entry.
//! The genuine lower Stack and actually merged selected entry retain all providers.

use serde_json::{Value, json};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceChild, SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::NativeTemplateOwner;
use vize_l3::decision::native::build_native_dom_file_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit_template,
    write::{NoLinks, Recorded},
};

#[test]
fn whole_selected_condensed_root_modules_maps_and_links_match_pinned_packets()
-> Result<(), &'static str> {
    let packet: Value = serde_json::from_str(include_str!(
        "fixtures/native-selected-root-text-vue-3.5.35.json"
    ))
    .map_err(|_| "complete source packet")?;
    assert_eq!(packet["vueVersion"], "3.5.35");
    let cases = packet["cases"].as_array().ok_or("complete cases")?;
    assert_eq!(cases.len(), 24);
    let filename = packet["options"]["filename"].as_str().ok_or("filename")?;
    let capture = std::env::var("VIZE_L4_SELECTED_ROOT_TEXT_CAPTURE")
        .ok()
        .or_else(|| {
            if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
                std::env::var("GITHUB_WORKSPACE").ok().map(|workspace| {
                    let profile = std::env::var("NEXTEST_PROFILE")
                        .ok()
                        .filter(|profile| matches!(profile.as_str(), "pr" | "full"))
                        .unwrap_or_else(|| "pr".to_owned());
                    format!(
                        "{workspace}/target/nextest/{profile}/native-selected-root-text-dom.json"
                    )
                })
            } else {
                None
            }
        });
    let mut captured = Vec::new();
    for fixture in cases {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().ok_or("whole authored source")?;
        let descriptor = Vue.observe_descriptor(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let selected = NativeTemplateComponent::parse_in(
            &arena,
            descriptor.admitted().map_err(|_| "descriptor admission")?,
        )
        .map_err(|_| "once original Component")?
        .ok_or("selected template")?;
        let block = selected.component().block();
        assert_eq!(
            json!([block.span().start, block.span().end]),
            fixture["templateSpan"]
        );
        assert_eq!(
            block.source(),
            fixture["template"].as_str().ok_or("template")?
        );
        assert_eq!(
            selected.children().len(),
            fixture["rootSlots"].as_array().ok_or("slots")?.len()
        );
        let mut original = NativeTemplateOwner::new(selected).map_err(|_| "genuine File owner")?;
        {
            let mut walk = original.begin().map_err(|_| "original root begin")?;
            let selected = walk.selected();
            for child in selected.children() {
                if matches!(child.surface(), SurfaceChild::Text(_)) {
                    let ordinal = child.ordinal();
                    let receipt = selected
                        .prepare_condensed_root_text(child)
                        .map_err(|_| "genuine original root receipt")?;
                    let text = fixture["rootText"]
                        .as_array()
                        .ok_or("text facts")?
                        .iter()
                        .find(|text| text["ordinal"] == ordinal)
                        .ok_or("same original ordinal")?;
                    assert_eq!(json!(receipt.raw_text()), text["raw"]);
                    assert_eq!(json!(receipt.content()), text["content"]);
                    assert_eq!(
                        json!([receipt.span().start, receipt.span().end]),
                        text["span"]
                    );
                    let minted = walk.root_text(&receipt).map_err(|_| "actual cursor text")?;
                    assert_eq!(minted.is_some(), receipt.content().is_some());
                } else {
                    walk.child(child).map_err(|_| "actual nontext")?;
                }
            }
            walk.complete().map_err(|_| "complete original root")?;
        }
        let output = core::hint::black_box(original.finish());
        let analysis = build_native_dom_file_decisions(output.view().map_err(|_| "completion")?)
            .map_err(|_| "genuine selected L3")?;
        assert!(core::ptr::eq(analysis.owner(), &output));
        assert!(core::ptr::eq(
            analysis.file(),
            output.file().ok_or("sole File")?
        ));
        assert!(core::ptr::eq(analysis.artifact().source(), source));
        assert!(analysis.dom().ok_or("DOM facts")?.unsupported().is_empty());
        let recorded = assemble_template(
            emit_template::<Recorded>(&analysis).map_err(|_| "recorded selected target")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "recorded whole module")?;
        let plain = assemble_template(
            emit_template::<NoLinks>(&analysis).map_err(|_| "plain selected target")?,
            vocabulary(Runtime::VueDom),
        )
        .map_err(|_| "plain whole module")?;
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        assert!(plain.into_document().links().is_empty());
        let document = recorded.into_document();
        assert_eq!(
            document.as_str(),
            fixture["referenceCode"]
                .as_str()
                .ok_or("complete official module")?,
            "{}",
            fixture["id"]
        );
        let map: Value = serde_json::from_str(&document.source_map(filename, source))
            .map_err(|_| "complete native map")?;
        let links = document
            .links()
            .iter()
            .map(|link| {
                json!({
                    "generated":[link.generated.start,link.generated.end],
                    "authored":[link.authored.start,link.authored.end],
                    "name":link.name.as_ref().map(|name| name.as_str()),
                    "segment":link.segment,
                })
            })
            .collect::<Vec<_>>();
        let native = json!({"code":document.as_str(),"map":map,"links":links,
            "nodeCount":analysis.artifact().node_count()});
        if let Some(frozen) = fixture.get("native") {
            assert_eq!(&native, frozen, "{}: complete native packet", fixture["id"]);
        } else if capture.is_none() {
            return Err("missing genuine hosted native capture");
        }
        captured.push(json!({"id":fixture["id"],"source":source,"template":fixture["template"],
            "code":native["code"],"map":native["map"],"links":native["links"],"nodeCount":native["nodeCount"]}));
    }
    if let Some(path) = capture {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent).map_err(|_| "native capture directory")?;
        }
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&captured).map_err(|_| "capture encoding")?,
        )
        .map_err(|_| "capture write")?;
    }
    Ok(())
}
