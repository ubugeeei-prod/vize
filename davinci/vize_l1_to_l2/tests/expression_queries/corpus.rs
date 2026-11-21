use super::{Counts, compare};
use std::{fs, path::Path};
#[test]
#[ignore = "reads a NUL-separated manifest of original Vue files"]
fn original_vue_files() {
    let manifest =
        std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").expect("VIZE_EXPRESSION_QUERY_MANIFEST");
    let mut counts = Counts::default();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in fs::read_to_string(manifest).unwrap().split_terminator('\0') {
        let bytes = fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            counts.files += 1;
            counts.invalid_utf8 += 1;
            continue;
        };
        compare(source, name, &mut counts, false);
    }
    assert_eq!(
        counts.files,
        counts.admitted + counts.refused + counts.invalid_utf8
    );
    assert!(
        counts.uses > 0 && counts.bindings > 0 && counts.templates > 0,
        "{counts:?}"
    );
    eprintln!("expression reference corpus {counts:?}");
}
