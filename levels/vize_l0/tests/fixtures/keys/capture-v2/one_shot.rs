// Link the already-built xxhash_rust artifact only after explicit build approval.
use std::io::{self, BufRead};
use xxhash_rust::xxh3::xxh3_128;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "id, exact preimage hex, captured digestHex (all24 rows)");
        let bytes: Vec<_> = (0..fields[1].len()).step_by(2)
            .map(|i| u8::from_str_radix(&fields[1][i..i + 2], 16).unwrap()).collect();
        let actual: String = xxh3_128(&bytes).to_be_bytes().iter()
            .map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(actual, fields[2], "one-shot digest mismatch {}", fields[0]);
        println!("{}\t{}\t{}", fields[0], fields[1], actual);
    }
}
