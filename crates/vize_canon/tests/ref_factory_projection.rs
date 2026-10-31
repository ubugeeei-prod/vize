//! Identical external observer for exact-base/head ref-factory qualification.
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_canon::{batch::VirtualProject, virtual_ts::SHARED_PREAMBLE_DTS};
use vize_carton::cstr;

#[test]
fn complete_ref_factory_projection_packets() {
    let fixture = std::env::var_os("VIZE_REF_FACTORY_FIXTURE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/_fixtures/differential/typechecker/shared-ref-factory")
        });
    let temporary = tempfile::tempdir().unwrap();
    let root = std::env::var_os("VIZE_REF_FACTORY_PROJECT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temporary.path().join("project"));
    assert!(root.is_absolute() && !root.exists());
    fs::create_dir_all(root.join("src")).unwrap();
    if let Some(provider) = std::env::var_os("VIZE_REF_FACTORY_PROVIDER") {
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            Path::new(&provider).join("node_modules"),
            root.join("node_modules"),
        )
        .unwrap();
        #[cfg(not(unix))]
        panic!("the explicitly hosted provider observer requires Unix");
    }
    let mut inputs = Vec::new();
    for (source, target) in [
        ("Child.vue.txt", "src/Child.vue"),
        ("Probe.vue.txt", "src/Probe.vue"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ] {
        let bytes = fs::read(fixture.join(source)).unwrap();
        inputs.push(json!({"file":target,"wholeUtf8":std::str::from_utf8(&bytes).unwrap()}));
        fs::write(root.join(target), bytes).unwrap();
    }
    let files = [root.join("src/Child.vue"), root.join("src/Probe.vue")];
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&files).unwrap();
    let packets = files.iter().map(|original| {
        let file = project.find_by_original(original).unwrap();
        let source = fs::read_to_string(original).unwrap();
        let sfc = file.source_map.sfc_map.as_ref().unwrap();
        let projection = sfc.projection();
        let link = |link: &vize_canon::virtual_ts::VizeSemanticLink| json!({"source":link.source_range,"target":link.target_range,"kind":cstr!("{:?}",link.kind)});
        let forward = (0..=file.content.len()).map(|offset| file.source_map.import_map.get_original_offset(offset as u32)).collect::<Vec<_>>();
        let pre_len = *forward.last().unwrap();
        json!({
            "file":original.strip_prefix(&root).unwrap().to_str().unwrap(),
            "source":source,"code":file.content,
            "authoredBase":projection.authored_base(),
            "rows":projection.rows().map(|row| json!({"generated":row.span.gen_range,"authored":row.span.src_range,"subSpans":row.span.sub_spans.iter().map(|span| json!({"generated":span.gen_range,"authored":span.src_range})).collect::<Vec<_>>(),"features":row.meta.features.bits(),"kind":cstr!("{:?}",row.meta.kind)})).collect::<Vec<_>>(),
            "semanticLinks":sfc.semantic_links().iter().map(link).collect::<Vec<_>>(),
            "propDefaultKeyLinks":sfc.prop_default_key_links().iter().map(link).collect::<Vec<_>>(),
            "authoredBlocks":(0..=source.len()).map(|offset| cstr!("{:?}",sfc.block_type_at(offset as u32))).collect::<Vec<_>>(),
            "importMapDebug":cstr!("{:?}",file.source_map.import_map),
            "importForward":forward,
            "importReverse":(0..=pre_len).map(|offset| file.source_map.import_map.get_virtual_offset(offset)).collect::<Vec<_>>(),
            "compositeForward":(0..=file.content.len()).map(|offset| file.source_map.get_original_position(offset as u32).map(|(authored,column,block)| json!([authored,column,cstr!("{:?}",block)]))).collect::<Vec<_>>(),
        })
    }).collect::<Vec<Value>>();
    let packet = json!({"schema":"vize.ref-factory-projection-v1","inputs":inputs,"sharedPreamble":SHARED_PREAMBLE_DTS,"files":packets});
    if let Some(destination) = std::env::var_os("VIZE_REF_FACTORY_CAPTURE") {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .unwrap();
        file.write_all(&serde_json::to_vec_pretty(&packet).unwrap())
            .unwrap();
    }
    assert_eq!(packet["files"].as_array().unwrap().len(), 2);
    drop(project);
    fs::remove_dir_all(&root).unwrap();
}
