use super::super::declaration_outputs::strip_internal_vue_declaration_fields;

#[test]
fn strips_internal_fallthrough_fields_from_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeRawProps?: Props;
    readonly __vizeHasFallthroughProps: true;
    readonly __vizeFallthroughProps?: Partial<__VizeNativeElement<\"button\">>;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}

#[test]
fn strips_multiline_internal_fallthrough_fields_from_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeHasFallthroughProps: true;
    readonly __vizeFallthroughProps?:
        Partial<__VizeNativeElement<\"button\">> &
        Partial<__VizeNativeElement<\"a\">>;
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}

#[test]
fn strips_internal_marker_from_raw_props_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}

#[test]
fn retains_authored_marker_text_outside_generated_metadata() {
    let source = r#"export interface PublicProps {
    label: "__vizeComponentMarker";
    __vizeComponentMarkerSuffix: string;
    prefix__vizeHasFallthroughProps: number;
    nested: { readonly __vizeFallthroughProps?: string };
    /** __vizeFallthroughProps is ordinary documentation; */
    multiline:
        "__vizeFallthroughProps" |
        "ordinary";
}
"#;
    assert_eq!(strip_internal_vue_declaration_fields(source), None);
}

#[test]
fn removes_only_direct_generated_properties_preserving_inline_neighbors() {
    let source = r#"declare const __vize_component__: {
    readonly __vizeComponentMarker: true; readonly __vizeRawProps?: Props;
    readonly __vizeHasFallthroughProps: true;
    readonly __vizeFallthroughProps?: { label: ";" } &
        Partial<NativeElement>;
    /** __vizeComponentMarker belongs to this public literal. */
    label: "__vizeComponentMarker";
    nested: { readonly __vizeFallthroughProps?: string };
} & Constructor;
"#;
    assert_eq!(
        strip_internal_vue_declaration_fields(source).unwrap(),
        r#"declare const __vize_component__: {
     readonly __vizeRawProps?: Props;
    /** __vizeComponentMarker belongs to this public literal. */
    label: "__vizeComponentMarker";
    nested: { readonly __vizeFallthroughProps?: string };
} & Constructor;
"#
    );
}

#[test]
fn strips_only_generated_true_branch_of_options_api_component() {
    let source = r#"declare const __vize_component__: Authored extends object ? ({
    readonly __vizeComponentMarker: true;
    readonly __vizeRawProps?: Props;
} & Constructor) : { readonly __vizeComponentMarker: "authored" };
"#;
    assert_eq!(
        strip_internal_vue_declaration_fields(source).unwrap(),
        r#"declare const __vize_component__: Authored extends object ? ({
    readonly __vizeRawProps?: Props;
} & Constructor) : { readonly __vizeComponentMarker: "authored" };
"#
    );
}

#[test]
fn declaration_map_composes_import_rewrite_owned_deletions_and_helper_prefix() {
    use oxc_sourcemap::{SourceMap, SourceMapBuilder};
    use vize_carton::line_index::LineIndex;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Button.vue.d.ts");
    let before = "import type { Remote } from './Remote.vue.ts'; /* 💥 */ export type Alias = Remote;\r\ndeclare const __vize_component__: {\r\n    readonly __vizeComponentMarker: true;\r\n    readonly __vizeFallthroughProps?: Partial<Remote>;\r\n    readonly __vizeRawProps?: PublicProps;\r\n} & Constructor;\r\nexport interface PublicProps {\r\n    label: \"__vizeComponentMarker\";\r\n}\r\n";
    let index = LineIndex::new(before);
    let mut builder = SourceMapBuilder::default();
    let source_id = builder.add_source_and_content("Button.vue", "authored 💥 source");
    for (position, needle) in ["Alias", "__vizeComponentMarker: true", "label:"]
        .iter()
        .enumerate()
    {
        let (line, col) = index.line_col(before.find(needle).unwrap());
        builder.add_token(line, col, position as u32 + 11, 13, Some(source_id), None);
    }
    std::fs::write(&path, before).unwrap();
    let map_path = dir.path().join("Button.vue.d.ts.map");
    std::fs::write(&map_path, builder.into_sourcemap().to_json_string()).unwrap();
    super::super::declaration_outputs::rewrite_declaration_outputs(dir.path()).unwrap();
    let after = std::fs::read_to_string(&path).unwrap();
    let raw_map = std::fs::read_to_string(map_path).unwrap();
    let map = SourceMap::from_json_string(&raw_map).unwrap();
    let after_index = LineIndex::new(&after);
    let actual: Vec<_> = map
        .get_tokens()
        .map(|token| {
            (
                token.get_dst_line(),
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
                token.get_source_id(),
            )
        })
        .collect();
    let expected: Vec<_> = [("Alias", 11), ("label:", 13)]
        .iter()
        .map(|(needle, source_line)| {
            let (line, col) = after_index.line_col(after.find(needle).unwrap());
            (line, col, *source_line, 13, Some(source_id))
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(
        after,
        concat!(
            "import type {  } from \"./__vize_helpers.js\";\n",
            "import type { Remote } from './Remote.vue'; /* 💥 */ export type Alias = Remote;\r\n",
            "declare const __vize_component__: {\r\n",
            "    readonly __vizeRawProps?: PublicProps;\r\n",
            "} & Constructor;\r\n",
            "export interface PublicProps {\r\n",
            "    label: \"__vizeComponentMarker\";\r\n",
            "}\r\n",
        )
    );
    assert_eq!(map.get_source(0), Some("Button.vue"));
    assert_eq!(map.get_source_content(0), Some("authored 💥 source"));
}

#[test]
fn retains_marker_properties_on_other_or_authored_declarations() {
    for source in [
        "declare const Other: { readonly __vizeComponentMarker: true };\n",
        "export declare const __vize_component__: { readonly __vizeComponentMarker: true };\n",
        "declare let __vize_component__: { readonly __vizeComponentMarker: true };\n",
        "const __vize_component__: { readonly __vizeComponentMarker: true } = value;\n",
        "declare namespace Public { const __vize_component__: { readonly __vizeComponentMarker: true }; }\n",
    ] {
        assert_eq!(
            strip_internal_vue_declaration_fields(source),
            None,
            "{source}"
        );
    }
}
