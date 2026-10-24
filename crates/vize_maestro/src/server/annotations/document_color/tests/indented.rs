use super::{ColorInformation, DocumentColorService};

#[test]
fn indented_styles_keep_complete_named_colour_vectors_and_authored_coordinates() {
    let expected: Vec<ColorInformation> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/stylus-colors-10442/response.expected.json"
    )))
    .unwrap();
    for input in [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/stylus-colors-10442/html.vue.txt"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/stylus-colors-10442/pug.vue.txt"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/stylus-colors-10442/jade.vue.txt"
        )),
    ] {
        for lang in ["sass", "stylus", "styl"] {
            for ending in ["\n", "\r\n"] {
                let source = input
                    .replace("lang=\"stylus\"", &format!("lang=\"{lang}\""))
                    .replace('\n', ending);
                let descriptor =
                    vize_incremental::descriptor::parse_descriptor("/Colors.vue", &source).unwrap();
                assert_eq!(
                    DocumentColorService::colors(&source, &descriptor),
                    expected,
                    "{lang}/{ending:?}: comments, strings and selectors are excluded"
                );
            }
        }
    }
}
