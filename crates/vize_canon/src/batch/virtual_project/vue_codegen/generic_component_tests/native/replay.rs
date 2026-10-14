//! Transparent prior-helper replay; this is not an unavailable old factory run.

use crate::batch::virtual_project::VirtualFile;
use vize_carton::{String, cstr};

pub(super) fn prior_contract(file: &VirtualFile, authored: &str) -> String {
    let prior = if authored == super::super::GENERIC {
        cstr!("{authored} = any")
    } else {
        authored.into()
    };
    let start = file
        .content
        .find("declare const __vize_component__: ")
        .unwrap();
    // Only an unmapped synthetic tail changes. All authored coordinates and
    // the real alias/public generic callable stay byte exact.
    assert!((start..file.content.len()).all(|offset| {
        file.source_map
            .get_original_position(offset as u32)
            .is_none()
    }));
    let (prefix, tail) = file.content.split_at(start);
    let mut tail = String::from(tail);
    for field in [
        "__vizeCheck: ",
        "__vizeResolveProps?: ",
        "__vizeResolveEmitProps?: ",
        "__vizeResolveEvents?: ",
    ] {
        let needle = cstr!("{field}<{authored}>");
        let count = tail.matches(needle.as_str()).count();
        assert!(count <= 1, "ambiguous synthetic field {field}: {tail}");
        if matches!(field, "__vizeCheck: " | "__vizeResolveProps?: ") {
            assert_eq!(
                count, 1,
                "missing actual synthetic contract {field}: {tail}"
            );
        }
        tail = tail
            .replace(needle.as_str(), cstr!("{field}<{prior}>").as_str())
            .into();
    }
    let replay = cstr!("{prefix}{tail}");
    assert_eq!(&replay[..start], &file.content[..start]);
    replay
}
