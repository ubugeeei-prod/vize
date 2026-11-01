//! The actual Vue generator retains its suffix, helper and config authority.

use super::{CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, CorsaVueVirtualDocumentOptions};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, path::PathBuf};
use vize_l0::{String, cstr};

fn save(capture: &Path, phase: &str, stage: &str, value: &Value) {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(capture.join(cstr!("{phase}-{stage}.json").as_str()))
        .unwrap();
    serde_json::to_writer_pretty(file, value).unwrap();
}

fn position(text: &str, byte: usize) -> (u32, u32) {
    let prefix = text.get(..byte).unwrap();
    (
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}

pub(crate) fn run() {
    let capture =
        PathBuf::from(std::env::var_os("VIZE_PRIVATE_OVERLAY_CAPTURE").unwrap()).join("vue-only");
    fs::create_dir(&capture).unwrap();
    let native = PathBuf::from(std::env::var_os("CORSA_PATH").unwrap())
        .canonicalize()
        .unwrap();
    let native_sha: String = Sha256::digest(fs::read(&native).unwrap())
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect();
    let work = tempfile::tempdir().unwrap();
    let root = work.path().canonicalize().unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::create_dir(root.join("node_modules")).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let vue = [
        repo.join("node_modules/vue"),
        repo.join("npm/builder/vite/node_modules/vue"),
    ]
    .into_iter()
    .find(|path| path.join("package.json").is_file())
    .unwrap()
    .canonicalize()
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&vue, root.join("node_modules/vue"))
        .or_else(|_| junction::create(&vue, root.join("node_modules/vue")))
        .unwrap();
    let host = root.join("src/App.vue");
    let support = root.join("src/support.ts");
    let source = "<script setup lang=\"ts\">\nimport Child from './Child.vue';\nimport { value } from '@/support';\nconst checked: number = value;\n</script>\n<template><Child :value=\"checked\" />{{ checked }}</template>\n";
    let child = "<script setup lang=\"ts\">defineProps<{ value: number }>()</script><template>{{ value }}</template>\n";
    let config = br#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","noEmit":true,"baseUrl":".","paths":{"@/*":["src/*"]}},"include":["src/**/*.vue"]}"#;
    fs::write(&host, source).unwrap();
    fs::write(root.join("src/Child.vue"), child).unwrap();
    fs::write(&support, "export const value = 1;\n").unwrap();
    fs::write(root.join("tsconfig.json"), config).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(native.clone()),
        working_dir: Some(root.clone()),
        timeout_ms: 10_000,
        ..Default::default()
    });
    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let mut rows = Vec::new();
        let disk_capture = capture.join("disk-control");
        fs::create_dir(&disk_capture).unwrap();
        for (phase, overlay) in [
            ("valid", Some("export const value = 1;\n")),
            ("dependency-error", Some("export const value = 'bad';\n")),
            ("dependency-repair", Some("export const value = 2;\n")),
            ("dependency-close", None),
        ] {
            let overlays = overlay
                .map(|text| vec![(support.clone(), text.into())])
                .unwrap_or_default();
            let opened = bridge
                .open_vue_virtual_document_with_overlays(
                    &host,
                    source,
                    CorsaVueVirtualDocumentOptions::default(),
                    &overlays,
                )
                .await;
            let mut packet = json!({"phase":phase,"source":source,"child":child,"supportOverlay":overlay,
                "authoredRoot":root,"authoredConfig":std::str::from_utf8(config).unwrap(),
                "native":{"path":native,"sha256":native_sha},"opened":opened.is_ok()});
            if let Err(error) = &opened {
                packet["openError"] = json!(cstr!("{error}"));
            }
            save(&capture, phase, "open", &packet);
            let document = opened.unwrap();
            packet["document"] = json!({"requestUri":document.request_uri,"virtualSuffix":document.virtual_suffix,
                "code":document.code,"preRewriteCode":document.pre_rewrite_code,
                "sessionProjectRoot":document.session_project_root,
                "dependencies":document.dependencies.iter().map(|dependency| json!({"sourcePath":dependency.source_path,"requestUri":dependency.request_uri,"code":dependency.code})).collect::<Vec<_>>()});
            let query_uri = document.request_uri.clone();
            let diagnostics =
                bridge
                    .with_client(move |client| {
                        client.request_diagnostics_full(&query_uri)
                .map(|fetch| json!({"usedCache":fetch.used_cache,"items":fetch.diagnostics}))
                .map_err(CorsaBridgeError::CommunicationError)
                    })
                    .await;
            packet["diagnosticsResult"] = match &diagnostics {
                Ok(value) => json!({"Ok":value}),
                Err(error) => json!({"Err":cstr!("{error}")}),
            };
            save(&capture, phase, "diagnostics", &packet);
            let diagnostics = diagnostics.unwrap();
            assert_eq!(diagnostics["usedCache"], false);
            let diagnostics = diagnostics["items"].clone();
            let query_byte =
                document.code.find("checked: number = value").unwrap() + "checked: number = ".len();
            let (line, character) = position(&document.code, query_byte);
            let query_uri = document.request_uri.clone();
            let hover = bridge
                .with_client(move |client| {
                    client
                        .hover_raw(&query_uri, line, character)
                        .map_err(CorsaBridgeError::CommunicationError)
                })
                .await;
            packet["hoverResult"] = match &hover {
                Ok(value) => json!({"Ok":value}),
                Err(error) => json!({"Err":cstr!("{error}")}),
            };
            save(&capture, phase, "hover", &packet);
            let hover = hover.unwrap().expect("actual owned value hover required");
            let query_uri = document.request_uri.clone();
            let definition = bridge
                .with_client(move |client| {
                    client
                        .definition_raw(&query_uri, line, character)
                        .map_err(CorsaBridgeError::CommunicationError)
                })
                .await;
            packet["definitionResult"] = match &definition {
                Ok(value) => json!({"Ok":value}),
                Err(error) => json!({"Err":cstr!("{error}")}),
            };
            save(&capture, phase, "definition", &packet);
            let definition = definition
                .unwrap()
                .expect("actual owned value definition required");
            let targets = definition.as_array().unwrap();
            assert_eq!(targets.len(), 1);
            let target = targets.first().unwrap()["uri"].as_str().unwrap();
            let source_member = document
                .materialized_sources
                .iter()
                .find(|member| {
                    member.source_path.canonicalize().ok() == Some(support.canonicalize().unwrap())
                })
                .unwrap();
            assert_eq!(
                crate::file_uri::file_uri_to_path(target)
                    .unwrap()
                    .canonicalize()
                    .unwrap(),
                source_member.materialized_path.canonicalize().unwrap()
            );
            assert_eq!(
                definition,
                json!([{"uri":target,"range":{"start":{"line":0,"character":13},"end":{"line":0,"character":18}}}])
            );
            let value_type = match phase {
                "valid" | "dependency-close" => "1",
                "dependency-error" => "\"bad\"",
                "dependency-repair" => "2",
                _ => unreachable!(),
            };
            assert_eq!(
                hover,
                json!({"contents":{"kind":"markdown","value":cstr!("```typescript\n(alias) const value: {value_type}\n```\n")},
                "range":{"start":{"line":line,"character":character},"end":{"line":line,"character":character+5}}})
            );
            let uri = document.request_uri.clone();
            let configuration = bridge
                .with_client(move |client| {
                    client
                        .logical_configuration_for_test(&uri)
                        .map_err(CorsaBridgeError::CommunicationError)
                })
                .await;
            packet["configurationResult"] = match &configuration {
                Ok(value) => json!({"Ok":value}),
                Err(error) => json!({"Err":cstr!("{error}")}),
            };
            save(&capture, phase, "configuration", &packet);
            let selected = configuration.unwrap().canonicalize().unwrap();
            let generated_root = document.session_project_root.as_ref().unwrap();
            assert_eq!(
                selected,
                generated_root.join("tsconfig.json").canonicalize().unwrap()
            );
            assert!(generated_root.join("__vize_vue_modules.d.ts").is_file());
            let native_path = crate::file_uri::file_uri_to_path(&document.request_uri).unwrap();
            let expected_name = cstr!("App.vue{}", document.virtual_suffix);
            assert_eq!(
                native_path.file_name().unwrap().to_str(),
                Some(expected_name.as_str())
            );
            let physical_code = fs::read(&native_path).unwrap();
            assert_eq!(physical_code, document.code.as_bytes());
            let assets = [
                (
                    "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
                    include_str!("../virtual_ts/helpers/declarations/core-types.d.ts.txt"),
                ),
                (
                    "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt",
                    include_str!("../virtual_ts/helpers/declarations/vue3-types.d.ts.txt"),
                ),
            ];
            let helpers: Vec<_> = assets.iter().map(|(path, text)| {
                let start = document.code.find(*text).expect("whole inline producer asset required");
                assert_eq!(document.code.matches(*text).count(), 1);
                assert_eq!(document.code.get(start..start+text.len()), Some(*text));
                json!({"mode":"inline-producer-asset","producerPath":path,"bytes":text.len(),
                    "sha256":Sha256::digest(text.as_bytes()).iter().map(|byte|cstr!("{byte:02x}")).collect::<String>(),
                    "startByte":start,"endByte":start+text.len(),"text":text})
            }).collect();
            packet["generatedAuthority"] = json!({"selectedConfigPath":selected,
                "selectedConfigText":fs::read_to_string(&selected).unwrap(),"helpers":helpers,
                "physicalGeneratedFile":{"path":native_path,"bytes":physical_code.len(),"sha256":Sha256::digest(&physical_code).iter().map(|byte|cstr!("{byte:02x}")).collect::<String>()}});
            save(&capture, phase, "generated-authority", &packet);
            let disk = crate::lsp_client::native_logical_disk_control_for_test(
                &native,
                generated_root,
                &document.request_uri,
                &disk_capture,
                phase,
                (line, character),
                &source_member.materialized_path,
            );
            packet["independentDiskResult"] = match &disk {
                Ok(value) => json!({"Ok":value}),
                Err(error) => json!({"Err":error}),
            };
            save(&capture, phase, "independent-disk", &packet);
            let disk = disk.unwrap();
            assert_eq!(fs::read(&host).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&support).unwrap(), b"export const value = 1;\n");
            assert_eq!(fs::read(root.join("tsconfig.json")).unwrap(), config);
            let checked = position(
                &document.code,
                document.code.find("checked: number").unwrap(),
            );
            let range = json!({"start":{"line":checked.0,"character":checked.1},"end":{"line":checked.0,"character":checked.1+7}});
            let errors: Vec<_> = diagnostics
                .as_array()
                .unwrap()
                .iter()
                .filter(|diagnostic| diagnostic["code"] == 2322 && diagnostic["range"] == range)
                .cloned()
                .collect();
            let expected_error = if phase == "dependency-error" {
                json!([{"range":range,"severity":1,"code":2322,"source":"ts",
                    "message":"Type 'string' is not assignable to type 'number'."}])
            } else {
                json!([])
            };
            packet["checkingWitness"] = json!({"actual":errors,"expected":expected_error});
            save(&capture, phase, "checking-witness", &packet);
            rows.push((packet, diagnostics, disk));
        }
        let shutdown = bridge.shutdown().await;
        save(
            &capture,
            "all",
            "shutdown",
            &json!({"shutdown":shutdown.as_ref().map(|_|()).map_err(|error|cstr!("{error}")),"rows":rows}),
        );
        shutdown.unwrap();
        let phases = rows
            .iter()
            .map(|(packet, _, _)| packet["phase"].clone())
            .collect::<Vec<_>>();
        let mut matched = 0;
        for (packet, actual, disk) in rows {
            assert_eq!(
                actual, disk["diagnostics"],
                "whole Vue-only diagnostics mismatch: {}",
                packet["phase"]
            );
            assert_eq!(packet["hoverResult"]["Ok"], disk["hover"]);
            assert_eq!(packet["definitionResult"]["Ok"], disk["definition"]);
            assert_eq!(
                packet["checkingWitness"]["actual"],
                packet["checkingWitness"]["expected"]
            );
            matched += 1;
        }
        println!(
            "\nnative-overlay-vue-only-control {}",
            json!({"schema":1,"phases":phases,
            "wholeDiagnosticsMatched":matched,"nativeSha256":native_sha})
        );
    });
}
