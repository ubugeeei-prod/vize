use super::format_file_source;
use std::path::Path;

#[test]
#[expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
fn format_file_source_formats_standalone_tsx() {
    let options = super::FormatOptions::default();
    let allocator = super::Allocator::default();
    let result = format_file_source(
        Path::new("Component.tsx"),
        "const Component=({label}:{label:string})=><button>{label}</button>",
        &options,
        &allocator,
        (super::VueVersion::V3, None, false),
    )
    .unwrap();

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_snapshot!(result.code.as_str());
    });
}

#[test]
fn format_file_source_formats_tsrx_host_code_around_verbatim_markup() {
    let options = super::FormatOptions::default();
    let allocator = super::Allocator::default();
    let source = "export function Badge({text}:{text:string}) @{\n\t<span class='badge'>{ text }</span>\n}\n";
    let result = format_file_source(
        Path::new("Badge.tsrx"),
        source,
        &options,
        &allocator,
        (super::VueVersion::V3, None, false),
    )
    .unwrap();
    assert!(result.changed);
    assert_eq!(
        result.code.as_str(),
        "export function Badge({ text }: { text: string }) @{\n\t<span class='badge'>{ text }</span>\n}\n"
    );
    assert!(
        format_file_source(
            Path::new("Broken.tsrx"),
            "export function A() @{\n\t<div>\n}\n",
            &options,
            &allocator,
            (super::VueVersion::V3, None, false),
        )
        .is_err()
    );
}
