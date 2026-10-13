//! Explicit current authorities retain historical compiler packets unchanged.
use super::{ExpectedCase, load_expected, parse_snap_file};
use std::path::Path;

pub(super) fn load(path: &Path) -> Vec<ExpectedCase> {
    let mut cases = load_expected(path);
    if path.ends_with("tests/expected/vapor/component.snap") {
        let packet = include_str!(
            "../../_fixtures/differential/compiler/vapor-root-fallthrough-8625/keep-alive-successor.snap"
        );
        if let Some(successor) = parse_snap_file(packet).pop()
            && let Some(original) = cases.iter_mut().find(|case| {
                case.name == "KeepAlive" && case.input == "<KeepAlive><Component /></KeepAlive>"
            })
        {
            *original = successor;
        }
    }
    cases
}
