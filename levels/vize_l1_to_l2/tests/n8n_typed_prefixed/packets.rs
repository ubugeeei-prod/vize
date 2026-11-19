//! Retain complete actual results before any admission/oracle assertion.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

pub(super) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn retain(name: &str, source: &str, lane: &str, packet: Value) {
    let Some(directory) = std::env::var_os("VIZE_N8N_CONTROL_PACKET_DIR") else {
        return;
    };
    let executable = std::env::current_exe().expect("actual test producer");
    let binary = fs::read(&executable).expect("actual producer bytes");
    let binary_sha256 = digest(&binary);
    let root = PathBuf::from(directory).join(&binary_sha256);
    fs::create_dir_all(&root).expect("authored packet directory");
    let producer = json!({"executable":executable,"binarySha256":binary_sha256,
        "manifestDirectory":env!("CARGO_MANIFEST_DIR"),
        "githubSha":std::env::var("GITHUB_SHA").ok(),
        "argv":std::env::args().collect::<Vec<_>>()});
    let identity = digest(format!("{name}\0{lane}\0{source}").as_bytes());
    let packet = json!({"schema":1,"name":name,"lane":lane,
        "originalSfc":source,"originalSha256":digest(source.as_bytes()),
        "producer":producer,"packet":packet});
    let bytes = serde_json::to_vec_pretty(&packet).expect("complete compiler packet serialization");
    let packet_sha256 = digest(&bytes);
    fs::write(root.join(format!("{identity}.json")), bytes)
        .expect("complete current output retained before assertions");
    fs::write(
        root.join(format!("{identity}.sha256")),
        format!("{packet_sha256}  {identity}.json\n"),
    )
    .expect("whole output checksum retained before assertions");
}
