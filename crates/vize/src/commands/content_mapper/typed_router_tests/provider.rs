//! Admitted same-version providers and failed reloads in the real mapper server.

use std::io::{BufReader, Cursor, Write};
use std::path::Path;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_canon::{
    ContentMapperProjectConfiguration, generate_vue_content_mapper_transform_with_project_context,
};

use super::super::test_support::{exchange, frames, initialize_request};

const CONFIG: &str = r#"{"compilerOptions":{"moduleResolution":"Bundler"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#;
const BAD_CONFIG: &str = "{\"vueCompilerOptions\":";
const PAGE: &str = "<script setup lang=\"ts\">\nimport { definePage } from 'vue-router/experimental'\nconst props = defineProps<{ extra?: string }>()\ndefinePage({ props: true })\nvoid props\n</script>\n<template>{{ id }}</template>";

fn install(root: &Path, kind: &str) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/experimental-router-project-capability");
    let authority: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("artifacts.json")).unwrap()).unwrap();
    assert_eq!(
        authority["schema"],
        "vize.router-project-capability-artifacts"
    );
    assert_eq!(authority["version"], 1);
    assert_eq!(authority["issue"], 10579);
    let provider = authority["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|provider| provider["kind"] == kind)
        .unwrap();
    assert_eq!(provider["name"], "vue-router");
    assert_eq!(provider["version"], "5.4.0");
    let package = root.join("node_modules/vue-router");
    if package.exists() {
        std::fs::remove_dir_all(&package).unwrap();
    }
    for entry in provider["files"].as_array().unwrap() {
        let path = entry["path"].as_str().unwrap();
        let bytes = std::fs::read(fixture.join(kind).join(vize_l0::cstr!("{path}.txt"))).unwrap();
        assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
        assert_eq!(
            vize_l0::cstr!("{:x}", Sha256::digest(&bytes)),
            entry["sha256"].as_str().unwrap()
        );
        let target = package.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
    }
}

fn open(id: u8, handle: &str, root: &Path) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"openProject","params":{
        "projectHandle":handle,"configFileName":root.join("tsconfig.json"),
        "compilerOptions":{"rootDir":root,"moduleResolution":100}
    }})
}

fn transform(id: u8, handle: &str, root: &Path) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"transform","params":{
        "projectHandle":handle,"fileName":root.join("src/pages/[id].vue"),"content":PAGE
    }})
}

fn expected(root: &Path) -> Value {
    let (settings, _) = ContentMapperProjectConfiguration::load_with_compiler_options(
        &root.join("tsconfig.json"),
        json!({"rootDir":root,"moduleResolution":100})
            .as_object()
            .unwrap(),
    )
    .unwrap();
    serde_json::to_value(
        generate_vue_content_mapper_transform_with_project_context(
            &root.join("src/pages/[id].vue"),
            PAGE,
            settings.transform_options,
            &settings.project_context,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn same_process_same_version_router_providers_keep_complete_packets_isolated() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_l0::path::canonicalize_non_verbatim(directory.path());
    let preview = root.join("preview");
    let published = root.join("published");
    for (project, kind) in [(&preview, "preview"), (&published, "published")] {
        std::fs::create_dir_all(project).unwrap();
        std::fs::write(project.join("tsconfig.json"), CONFIG).unwrap();
        install(project, kind);
    }
    let enabled = expected(&preview);
    let disabled = expected(&published);
    assert_ne!(enabled, disabled);
    let responses = exchange(&frames(&[
        initialize_request(),
        open(2, "preview", &preview),
        open(3, "published", &published),
        transform(4, "preview", &preview),
        transform(5, "published", &published),
        transform(6, "preview", &preview),
    ]));
    assert_eq!(
        responses[3],
        json!({"jsonrpc":"2.0","id":4,"result":enabled})
    );
    assert_eq!(
        responses[4],
        json!({"jsonrpc":"2.0","id":5,"result":disabled})
    );
    assert_eq!(
        responses[5],
        json!({"jsonrpc":"2.0","id":6,"result":enabled})
    );
}

struct ProviderEdits<'a> {
    output: Vec<u8>,
    responses: u8,
    config: &'a Path,
    entry: &'a Path,
    original: &'a [u8],
}

impl Write for ProviderEdits<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.output.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.responses += 1;
        match self.responses {
            2 => std::fs::remove_file(self.entry)?,
            4 => std::fs::write(self.config, BAD_CONFIG)?,
            7 => {
                std::fs::write(self.config, CONFIG)?;
                std::fs::write(self.entry, self.original)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[test]
fn provider_removal_reopen_and_failed_reload_never_reuse_stale_admission() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_l0::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(&config, CONFIG).unwrap();
    install(&root, "preview");
    let entry = root.join("node_modules/vue-router/dist/experimental/index.d.ts");
    let original = std::fs::read(&entry).unwrap();
    let enabled = expected(&root);
    std::fs::remove_file(&entry).unwrap();
    let disabled = expected(&root);
    assert_ne!(enabled, disabled);
    std::fs::write(&entry, &original).unwrap();
    std::fs::write(&config, BAD_CONFIG).unwrap();
    let invalid = ContentMapperProjectConfiguration::load_with_compiler_options(
        &config,
        json!({"rootDir":root,"moduleResolution":100})
            .as_object()
            .unwrap(),
    )
    .unwrap_err();
    let rejected_reload = json!({"jsonrpc":"2.0","id":6,"error":{
        "code":-32603,
        "message":vize_l0::cstr!("Failed to load content mapper configuration: {invalid}")
    }});
    std::fs::write(&config, CONFIG).unwrap();
    let input = frames(&[
        initialize_request(),
        open(2, "page", &root),
        transform(3, "page", &root),
        open(4, "page", &root),
        transform(5, "page", &root),
        open(6, "page", &root),
        transform(7, "page", &root),
        open(8, "page", &root),
        transform(9, "page", &root),
    ]);
    let mut writer = ProviderEdits {
        output: Vec::new(),
        responses: 0,
        config: &config,
        entry: &entry,
        original: &original,
    };
    super::super::serve(&mut BufReader::new(Cursor::new(input)), &mut writer).unwrap();
    let mut reader = BufReader::new(Cursor::new(writer.output));
    let mut responses = Vec::<Value>::new();
    while let Some(body) = super::super::read_frame(&mut reader).unwrap() {
        responses.push(serde_json::from_slice(&body).unwrap());
    }
    assert_eq!(responses.len(), 9);
    assert_eq!(
        responses[2],
        json!({"jsonrpc":"2.0","id":3,"result":enabled})
    );
    assert_eq!(
        responses[4],
        json!({"jsonrpc":"2.0","id":5,"result":disabled})
    );
    assert_eq!(responses[5], rejected_reload);
    assert_eq!(
        responses[6],
        json!({"jsonrpc":"2.0","id":7,"error":{"code":-32602,"message":"Unknown content mapper project: page"}})
    );
    assert_eq!(
        responses[8],
        json!({"jsonrpc":"2.0","id":9,"result":enabled})
    );
    assert_ne!(
        responses[1]["result"]["configIdentity"],
        responses[3]["result"]["configIdentity"]
    );
    assert_eq!(
        responses[1]["result"]["configIdentity"],
        responses[7]["result"]["configIdentity"]
    );
}
