use tower_lsp::lsp_types::{NumberOrString, Position, Range, Url};

use super::router::{route_param_diagnostics, route_params_for_file};

const REPORTED: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/router-param-parsers/Reported.vue.txt"
);

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

#[test]
fn reported_page_only_warns_for_an_unknown_param_and_recovers() {
    let uri = Url::parse("file:///repo/src/pages/users/[id=int].vue").unwrap();
    let descriptor = vize_atelier_sfc::parse_sfc(REPORTED, Default::default()).unwrap();
    assert!(route_param_diagnostics(REPORTED, &uri, &descriptor).is_empty());

    let invalid = REPORTED.replace("route.params.id", "route.params.missing");
    let descriptor = vize_atelier_sfc::parse_sfc(&invalid, Default::default()).unwrap();
    let diagnostics = route_param_diagnostics(&invalid, &uri, &descriptor);
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String(
            "ecosystem/vue-router-route-param".into()
        ))
    );
    assert_eq!(diagnostic.source.as_deref(), Some("vize/ecosystem"));
    assert_eq!(
        diagnostic.range,
        Range::new(Position::new(4, 25), Position::new(4, 32))
    );
    assert_eq!(
        diagnostic.message,
        "Route param `missing` is not defined by this page file. Available params: id"
    );

    let descriptor = vize_atelier_sfc::parse_sfc(REPORTED, Default::default()).unwrap();
    assert!(route_param_diagnostics(REPORTED, &uri, &descriptor).is_empty());
}
