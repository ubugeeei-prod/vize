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
