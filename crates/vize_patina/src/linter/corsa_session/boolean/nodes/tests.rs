//! Captured from the pinned TypeScript 7.0.2 worker, not fabricated handles.
use super::{NODE, locations, word};

const SOURCE: &str = include_str!("fixtures/conditions.ts");
const ENCODED: &[u8] = include_bytes!("fixtures/conditions.bin");

fn range(expression: &str) -> (u32, u32) {
    let start = SOURCE.find(expression).expect("authored expression");
    (
        SOURCE[..start].encode_utf16().count() as u32,
        SOURCE[..start + expression.len()].encode_utf16().count() as u32,
    )
}

fn set(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn selects_whole_element_access_and_call_after_astral_unicode() {
    let handles = locations(ENCODED, SOURCE, &[range("items[0]"), range("getTitle()")])
        .expect("encoded checker nodes");
    let actual: Vec<_> = handles.iter().map(|handle| handle.as_str()).collect();
    assert_eq!(
        actual,
        [
            "22.213./private/tmp/vize-checker-node-fixture/conditions.ts",
            "28.214./private/tmp/vize-checker-node-fixture/conditions.ts",
        ]
    );
}

#[test]
fn incomplete_and_unsupported_wire_data_fail_closed() {
    for end in 0..ENCODED.len() {
        assert!(
            locations(&ENCODED[..end], SOURCE, &[range("items[0]")]).is_err(),
            "prefix {end}"
        );
    }
    let mut changed = ENCODED.to_vec();
    set(&mut changed, 0, 7 << 24);
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    set(&mut changed, 0, 5 << 24);
    set(&mut changed, 24, 43);
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
}

#[test]
fn mismatched_snapshots_invalid_strings_and_metadata_fail_closed() {
    assert!(locations(ENCODED, "different snapshot", &[range("items[0]")]).is_err());
    let mut changed = ENCODED.to_vec();
    let extended = word(&changed, 32).expect("extended") as usize;
    set(&mut changed, extended + 8, u32::MAX);
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    let mut changed = ENCODED.to_vec();
    let nodes = word(&changed, 40).expect("nodes") as usize;
    let structured = word(&changed, 36).expect("structured");
    set(
        &mut changed,
        nodes + NODE + 20,
        0x8000_0000 | (structured - extended as u32),
    );
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
}

#[test]
fn invalid_ranges_and_parent_cycles_fail_closed() {
    for query in [(0, 0), (u32::MAX, u32::MAX), (0, 200)] {
        assert!(locations(ENCODED, SOURCE, &[query]).is_err());
    }
    let mut changed = ENCODED.to_vec();
    let nodes = word(&changed, 40).expect("nodes") as usize;
    set(&mut changed, nodes + 23 * NODE + 16, 23);
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
    set(&mut changed, nodes + 23 * NODE + 16, u32::MAX);
    assert!(locations(&changed, SOURCE, &[range("items[0]")]).is_err());
}
