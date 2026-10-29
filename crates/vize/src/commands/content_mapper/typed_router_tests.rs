use std::io::{BufReader, Cursor, Write};
use std::path::Path;

use serde_json::{Value, json};
use vize_canon::{
    ContentMapperProjectConfiguration, generate_vue_content_mapper_transform_with_project_options,
};

use super::test_support::{exchange, frames, initialize_request};

fn open(id: u8, handle: &str, config: &Path, root: &Path) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"openProject","params":{
        "projectHandle":handle,"configFileName":config,"compilerOptions":{"rootDir":root}
    }})
}

fn transform(id: u8, handle: &str, file: &Path, source: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"transform","params":{
        "projectHandle":handle,"fileName":file,"content":source
    }})
}

fn expected(config: &Path, root: &Path, file: &Path, source: &str) -> Value {
    let compiler = json!({"rootDir":root});
    let (settings, typed_router_root) =
        ContentMapperProjectConfiguration::load_with_compiler_options(
            config,
            compiler.as_object().unwrap(),
        )
        .unwrap();
    serde_json::to_value(
        generate_vue_content_mapper_transform_with_project_options(
            file,
            source,
            settings.transform_options,
            typed_router_root.as_deref(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn two_open_projects_keep_complete_transform_packets_isolated() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let enabled = root.join("enabled.json");
    let disabled = root.join("disabled.json");
    std::fs::write(
        &enabled,
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    std::fs::write(&disabled, "{}").unwrap();
    let file = root.join("café'\"/products/[id=int].vue");
    for newline in ["\n", "\r\n"] {
        let source = [
            "<script setup lang=\"ts\">",
            "// café 😀 ' \"",
            "import { useRoute } from 'vue-router'",
            "import { definePage } from 'vue-router/experimental'",
            "definePage({})",
            "const route = useRoute()",
            "</script>",
            "<template>{{ route.params.id }}</template>",
            "",
        ]
        .join(newline);
        let enabled_packet = expected(&enabled, &root, &file, &source);
        let disabled_packet = expected(&disabled, &root, &file, &source);
        assert_ne!(enabled_packet, disabled_packet);
        let responses = exchange(&frames(&[
            initialize_request(),
            open(2, "enabled", &enabled, &root),
            open(3, "disabled", &disabled, &root),
            transform(4, "enabled", &file, &source),
            transform(5, "disabled", &file, &source),
            transform(6, "enabled", &file, &source),
        ]));
        assert_eq!(
            responses[3],
            json!({"jsonrpc":"2.0","id":4,"result":enabled_packet})
        );
        assert_eq!(
            responses[4],
            json!({"jsonrpc":"2.0","id":5,"result":disabled_packet})
        );
        assert_eq!(
            responses[5],
            json!({"jsonrpc":"2.0","id":6,"result":enabled_packet})
        );
    }
}

struct EditAfterResponse<'a> {
    output: Vec<u8>,
    responses: usize,
    config: &'a Path,
}

impl Write for EditAfterResponse<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.output.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.responses += 1;
        match self.responses {
            // The first transform must use admitted context despite config absence.
            2 => std::fs::remove_file(self.config)?,
            // Reopening the same handle must admit this changed opt-in state.
            3 => std::fs::write(self.config, "{}")?,
            _ => {}
        }
        Ok(())
    }
}

#[test]
fn transforms_do_not_read_config_and_reopen_replaces_admitted_context() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    let file = root.join("src/pages/[id].vue");
    let source = "<script setup lang=\"ts\">import { useRoute } from 'vue-router'; const route = useRoute()</script><template>{{ route.params.id }}</template>";
    std::fs::write(&config, "{}").unwrap();
    let disabled = expected(&config, &root, &file, source);
    std::fs::write(
        &config,
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    let enabled = expected(&config, &root, &file, source);
    let input = frames(&[
        initialize_request(),
        open(2, "page", &config, &root),
        transform(3, "page", &file, source),
        open(4, "page", &config, &root),
        transform(5, "page", &file, source),
    ]);
    let mut writer = EditAfterResponse {
        output: Vec::new(),
        responses: 0,
        config: &config,
    };
    super::serve(&mut BufReader::new(Cursor::new(input)), &mut writer).unwrap();
    let mut reader = BufReader::new(Cursor::new(writer.output));
    let mut responses = Vec::<Value>::new();
    while let Some(body) = super::read_frame(&mut reader).unwrap() {
        responses.push(serde_json::from_slice(&body).unwrap());
    }
    assert_eq!(
        responses[2],
        json!({"jsonrpc":"2.0","id":3,"result":enabled})
    );
    assert_eq!(
        responses[4],
        json!({"jsonrpc":"2.0","id":5,"result":disabled})
    );
    assert_ne!(
        responses[1]["result"]["configIdentity"],
        responses[3]["result"]["configIdentity"]
    );
}
