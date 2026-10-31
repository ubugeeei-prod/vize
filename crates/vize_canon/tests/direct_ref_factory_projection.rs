//! The same external observer captures both generation modes on base and head.
#![expect(clippy::unwrap_used, reason = "assertion-driven projection observer")]

use serde_json::{Value, json};
use std::{fs, io::Write, path::Path};
use vize_canon::{
    batch::{ImportRewriter, generate_vue_document_virtual_ts},
    virtual_ts::{ProjectionFeatures, SHARED_PREAMBLE_DTS, VirtualTsOptions},
};
use vize_carton::cstr;

#[test]
fn dual_generic_refs_preserve_import_hygiene_and_both_projection_modes() {
    let fixture = std::env::var_os("VIZE_DIRECT_FACTORY_FIXTURE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/_fixtures/differential/typechecker/direct-ref-factory")
        });
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    let inputs = [
        ("Resolver.vue.txt", "src/Resolver.vue"),
        ("Parent.vue.txt", "src/Parent.vue"),
        ("names.ts.txt", "src/names.ts"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ]
    .map(|(pin, file)| {
        let source = fs::read_to_string(fixture.join(pin)).unwrap();
        fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
        fs::write(root.join(file), source.as_bytes()).unwrap();
        json!({"file":file,"wholeUtf8":source})
    });
    let mut packets = Vec::new();
    for hoisted in [false, true] {
        for input in &inputs[..2] {
            let file = input["file"].as_str().unwrap();
            let source = fs::read_to_string(root.join(file)).unwrap();
            let generated = generate_vue_document_virtual_ts(
                &root.join(file),
                &source,
                &VirtualTsOptions::default(),
                &ImportRewriter::new(),
                hoisted,
            )
            .unwrap();
            let map = &generated.mapping;
            let imports = &generated.import_source_map;
            let link = |link: &vize_canon::virtual_ts::VizeSemanticLink| json!({"source":link.source_range,"target":link.target_range,"kind":cstr!("{:?}",link.kind)});
            let features = ProjectionFeatures::NAMED.map(|(flag, name)| json!({
                "name":name,
                "toAuthored":(0..=generated.pre_rewrite_code.len()).map(|n| map.to_authored_for(n,flag)).collect::<Vec<_>>(),
                "toGenerated":(0..=source.len()).map(|n| map.to_generated_for(n,flag)).collect::<Vec<_>>()
            }));
            packets.push(json!({
                "file":file,"hoisted":hoisted,"source":source,
                "code":generated.code,"preRewrite":generated.pre_rewrite_code,
                "sourceType":cstr!("{:?}",generated.source_type),"virtualSuffix":generated.virtual_suffix,
                "authoredBase":map.authored_base(),
                "rows":map.rows().map(|row| json!({"generated":row.span.gen_range,"authored":row.span.src_range,"subSpans":row.span.sub_spans.iter().map(|span| json!({"generated":span.gen_range,"authored":span.src_range})).collect::<Vec<_>>(),"features":row.meta.features.bits(),"kind":cstr!("{:?}",row.meta.kind)})).collect::<Vec<_>>(),
                "semanticLinks":map.semantic_links().iter().map(link).collect::<Vec<_>>(),
                "propDefaultKeyLinks":map.prop_default_key_links().iter().map(link).collect::<Vec<_>>(),
                "features":features,
                "toAuthored":(0..=generated.pre_rewrite_code.len()).map(|n| map.to_authored(n)).collect::<Vec<_>>(),
                "toGenerated":(0..=source.len()).map(|n| map.to_generated(n)).collect::<Vec<_>>(),
                "importMapDebug":cstr!("{:?}",imports),
                "importForward":(0..=generated.code.len()).map(|n| imports.get_original_offset(n as u32)).collect::<Vec<_>>(),
                "importReverse":(0..=generated.pre_rewrite_code.len()).map(|n| imports.get_virtual_offset(n as u32)).collect::<Vec<_>>()
            }));
        }
    }
    let packet = json!({"schema":"vize.direct-ref-factory-projection-v1","inputs":inputs,"sharedPreamble":SHARED_PREAMBLE_DTS,"files":packets});
    if let Some(destination) = std::env::var_os("VIZE_DIRECT_FACTORY_CAPTURE") {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .unwrap()
            .write_all(&serde_json::to_vec(&packet).unwrap())
            .unwrap();
    }
    let rows = packet["files"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["file"], inputs[index % 2]["file"]);
        assert_eq!(row["source"], inputs[index % 2]["wholeUtf8"]);
        assert_eq!(row["features"].as_array().unwrap().len(), 8);
        let code = row["code"].as_str().unwrap();
        assert_eq!(
            row["importForward"].as_array().unwrap().len(),
            code.len() + 1
        );
        assert!(!row["rows"].as_array().unwrap().is_empty());
    }
    // Two separately specialized refs exercise the factory, while the DOM
    // ref and the same-name authored imports remain in the actual source.
    assert_eq!(rows[1]["source"], inputs[1]["wholeUtf8"]);
    assert_eq!(rows[3]["source"], inputs[1]["wholeUtf8"]);
}
