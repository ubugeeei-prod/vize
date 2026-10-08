//! Closed current references for two original, partially fixed #7876 inputs.
use sha2::{Digest, Sha256};

fn exact(bytes: &[u8], expected: &str) {
    let actual = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<std::string::String>();
    assert_eq!(actual, expected, "whole expression-width authority changed");
}

pub fn expected(id: &str, input: &[u8], original: Vec<u8>) -> Vec<u8> {
    let input_hash = match id {
        "original-deep-prebroken" => {
            "851c54f3ec30d67d54dc5b6369b06303f3222583da08d873c1144cf3a1a8cb52"
        }
        "deep-compact" => "2ebabc753042ae70cc0390163f416047857fb0d04dad3c4d656d95db6cd7fccf",
        _ => return original,
    };
    exact(
        include_bytes!(
            "../_fixtures/differential/formatter-regressions/directive-print-width-7876/corpus.json"
        ),
        "5a6bc66c6dc53678480e09cab6cd64d54c8f0cecaada29fe8b1b2fcd240dba2a",
    );
    exact(input, input_hash);
    exact(
        &original,
        "851c54f3ec30d67d54dc5b6369b06303f3222583da08d873c1144cf3a1a8cb52",
    );
    let current = include_bytes!(
        "../_fixtures/differential/formatter-regressions/expression-print-width-7876/original-deep-lf.expected"
    );
    exact(
        current,
        "b2fb731eb8b2376c016442868a4f632d62cc617c53d50e7d4ffa40a073e28b16",
    );
    current.to_vec()
}
