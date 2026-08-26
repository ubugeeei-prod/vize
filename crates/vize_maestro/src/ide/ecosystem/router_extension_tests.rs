use tower_lsp::lsp_types::Url;
use vize_s0::cstr;

use super::router::route_params_for_file;

#[test]
fn infers_params_from_module_route_file_names() {
    let uri = Url::parse("file:///repo/src/pages/[tenant]/settings.server.mts").unwrap();
    let params = route_params_for_file(&uri);

    assert_eq!(
        cstr!("{params:?}"),
        r#"[RouteParam { name: "tenant", optional: false, repeatable: false }]"#
    );

    let uri = Url::parse("file:///repo/src/pages/docs/[...slug].cjs").unwrap();
    let params = route_params_for_file(&uri);

    assert_eq!(
        cstr!("{params:?}"),
        r#"[RouteParam { name: "slug", optional: false, repeatable: true }]"#
    );
}
