//! Closed current reference for one previously partial #7876 output.
use sha2::{Digest, Sha256};

fn exact(bytes: &[u8], expected: &str) {
    let mut actual = vize_l0::String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        actual.push_str(vize_l0::cstr!("{byte:02x}").as_str());
    }
    assert_eq!(
        actual.as_str(),
        expected,
        "whole reference authority changed"
    );
}

pub fn expected(id: &str, input: &[u8], original: Vec<u8>) -> Vec<u8> {
    exact(
        include_bytes!(
            "../_fixtures/differential/formatter-regressions/directive-print-width-7876/corpus.json"
        ),
        "5a6bc66c6dc53678480e09cab6cd64d54c8f0cecaada29fe8b1b2fcd240dba2a",
    );
    if id != "original-example" {
        return original;
    }
    exact(
        input,
        "cda86e54a1e3f15b9c45652171e664c8fad2aa8e57064dfa6ee6998940e633fb",
    );
    exact(
        &original,
        "75d1acc5d580c789d5e35934d841dc294ce0fd8fc721daea0b4b0268f82af7a6",
    );
    let current = include_bytes!(
        "../_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/original-example.expected"
    );
    exact(
        current,
        "e379010d0e5b534b84dad6f3d6774c599e59ed7845243799ca42ec62432190c7",
    );
    current.to_vec()
}
