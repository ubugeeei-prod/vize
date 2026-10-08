use tower_lsp::lsp_types::Url;

use super::router::route_params_for_file;

#[test]
fn parser_suffix_preserves_param_modifiers_and_custom_names() {
    let cases = [
        ("[id=int].vue", "id", false, false),
        ("[slug=uuid].vue", "slug", false, false),
        ("[[id=int]].vue", "id", true, false),
        ("[ids=number]+.vue", "ids", false, true),
        ("[[ids=number]]+.vue", "ids", true, true),
        ("[...path].vue", "path", false, true),
        ("[id=int]@aside.vue", "id", false, false),
        ("[id].vue", "id", false, false),
    ];
    for (file, name, optional, repeatable) in cases {
        let uri = Url::parse(&vize_l0::cstr!("file:///repo/src/pages/users/{file}")).unwrap();
        let params = route_params_for_file(&uri);
        let observed = params
            .iter()
            .map(|param| (param.name.as_str(), param.optional, param.repeatable))
            .collect::<Vec<_>>();
        assert_eq!(observed, vec![(name, optional, repeatable)], "{file}");
    }
}

#[test]
fn parser_names_do_not_change_source_order_deduplication_or_empty_param_refusal() {
    let uri = Url::parse(
        "file:///repo/src/pages/[tenant=uuid]/[tenant]/product_[id=int]_[slug=uuid]/[=int].vue",
    )
    .unwrap();
    let params = route_params_for_file(&uri);
    let observed = params
        .iter()
        .map(|param| (param.name.as_str(), param.optional, param.repeatable))
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        vec![
            ("tenant", false, false),
            ("id", false, false),
            ("slug", false, false)
        ]
    );
}
