//! Optional evidence for the existing authored n8n targets, outside product code.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::expect_used,
    reason = "complete test artifact packets and producer custody"
)]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, sync::OnceLock};
use vize_atelier_core::{CodegenResult, CompilerError};
use vize_l1_to_l2::{DomEmitOptions, EmitError, ObservedPatchFactsEmit};

struct Output {
    root: PathBuf,
    producer: Value,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn output() -> Option<&'static Output> {
    static OUTPUT: OnceLock<Option<Output>> = OnceLock::new();
    OUTPUT
        .get_or_init(|| {
            let directory = std::env::var_os("VIZE_N8N_CONTROL_PACKET_DIR")?;
            let executable = std::env::current_exe().expect("actual test producer");
            let binary = fs::read(&executable).expect("authenticate actual test binary");
            let binary_sha256 = digest(&binary);
            let root = PathBuf::from(directory).join(&binary_sha256);
            fs::create_dir_all(&root).expect("test evidence directory");
            let producer = json!({"executable":executable,"binarySha256":binary_sha256,
                "manifestDirectory":env!("CARGO_MANIFEST_DIR"),
                "githubSha":std::env::var("GITHUB_SHA").ok(),
                "argv":std::env::args().collect::<Vec<_>>()});
            Some(Output { root, producer })
        })
        .as_ref()
}

pub(super) fn retain(name: &str, source: &str, lane: &str, packet: impl FnOnce() -> Value) {
    let Some(output) = output() else {
        return;
    };
    let identity = digest(format!("{name}\0{lane}\0{source}").as_bytes());
    let packet = json!({"schema":1,"name":name,"lane":lane,
        "originalSfc":source,"originalSha256":digest(source.as_bytes()),
        "producer":output.producer,"packet":packet()});
    fs::write(
        output.root.join(format!("{identity}.json")),
        serde_json::to_vec_pretty(&packet).expect("complete test packet serialization"),
    )
    .expect("retain complete authored compiler packet before assertions");
}

pub(super) fn retain_compile(
    input: (&str, &str, &str, &str),
    options: &DomEmitOptions<'_>,
    old: &CodegenResult,
    errors: &[CompilerError],
    native: &Result<ObservedPatchFactsEmit, EmitError>,
) {
    let (name, source, template, lane) = input;
    retain(name, source, lane, || {
        json!({
            "template":template,"optionsCompleteDebug":format!("{options:?}"),
            "legacy":{"preamble":old.preamble,"code":old.code,"map":old.map,
                "assembled":format!("{}\n{}",old.preamble,old.code),
                "errors":errors.iter().map(|error|json!({
                    "code":format!("{:?}",error.code),"message":error.message,"location":error.loc,
                    "completeDebug":format!("{error:?}")})).collect::<Vec<_>>()},
            "native":match native {
                Ok(observed) => json!({"tag":"Ok","completeDebug":format!("{observed:?}"),
                    "preamble":observed.emit.preamble,"code":observed.emit.code,
                    "assembled":observed.emit.assembled(),"materializedEntries":observed.materialized_entries}),
                Err(error) => json!({"tag":"Err","completeDebug":format!("{error:?}")}),
            }
        })
    });
}
