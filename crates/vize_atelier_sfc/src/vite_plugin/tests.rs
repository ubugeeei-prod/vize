use super::{
    classify_vite_plugin_request, create_virtual_id, from_virtual_id, normalize_fs_id_for_build,
    normalize_virtual_vue_module_id,
};

mod middleware;
mod precompile;

#[test]
fn snapshots_macro_define_page_request() {
    insta::assert_debug_snapshot!(
        classify_vite_plugin_request("/src/pages/Home.vue?definePage"),
        @r###"
        VitePluginRequest {
            path: "/src/pages/Home.vue",
            query_suffix: "?definePage",
            normalized_vue_path: "/src/pages/Home.vue",
            stripped_virtual_path: None,
            is_vize_virtual: false,
            is_vize_ssr_virtual: false,
            vize_virtual_path: None,
            normalized_fs_id: None,
            has_macro_query: false,
            has_define_page_query: true,
            is_macro_virtual_id: false,
            is_vue_sfc_path: true,
            is_vue_style_query: false,
            style_lang: None,
            style_index: None,
            style_scoped: None,
            has_style_module: false,
            style_virtual_suffix: None,
            boundary_kind: None,
        }
        "###
    );
}

#[test]
fn classifies_virtual_macro_paths() {
    let request = classify_vite_plugin_request("\0/src/pages/Home.vue.ts?macro=true");

    assert_eq!(
        request.stripped_virtual_path.as_deref(),
        Some("/src/pages/Home.vue")
    );
    assert_eq!(
        request.normalized_vue_path.as_str(),
        "\0/src/pages/Home.vue"
    );
    assert!(request.is_macro_virtual_id);
    assert!(request.is_vue_sfc_path);
    assert!(request.is_vize_virtual);
    assert_eq!(
        request.vize_virtual_path.as_deref(),
        Some("/src/pages/Home.vue")
    );
}

#[test]
fn classifies_ssr_virtual_vue_modules() {
    let request = classify_vite_plugin_request("\0vize-ssr:/src/pages/Home.vue.ts?used=true");

    assert!(request.is_vize_virtual);
    assert!(request.is_vize_ssr_virtual);
    assert_eq!(
        request.vize_virtual_path.as_deref(),
        Some("/src/pages/Home.vue")
    );
}

#[test]
fn normalizes_fs_ids_for_build() {
    let request = classify_vite_plugin_request("/@fs/src/entry.js?import");

    assert_eq!(
        request.normalized_fs_id.as_deref(),
        Some("/src/entry.js?import")
    );
    assert_eq!(
        normalize_fs_id_for_build("/@fs/src/entry.js?import").as_str(),
        "/src/entry.js?import"
    );
}

#[test]
fn snapshots_style_virtual_queries() {
    insta::assert_debug_snapshot!(
        classify_vite_plugin_request(
            "/src/App.vue?vue&type=style&index=2&lang=scss&module&scoped=data-v-test",
        ),
        @r###"
        VitePluginRequest {
            path: "/src/App.vue",
            query_suffix: "?vue&type=style&index=2&lang=scss&module&scoped=data-v-test",
            normalized_vue_path: "/src/App.vue",
            stripped_virtual_path: None,
            is_vize_virtual: false,
            is_vize_ssr_virtual: false,
            vize_virtual_path: None,
            normalized_fs_id: None,
            has_macro_query: false,
            has_define_page_query: false,
            is_macro_virtual_id: false,
            is_vue_sfc_path: true,
            is_vue_style_query: true,
            style_lang: Some(
                "scss",
            ),
            style_index: Some(
                2,
            ),
            style_scoped: Some(
                "data-v-test",
            ),
            has_style_module: true,
            style_virtual_suffix: Some(
                ".module.scss",
            ),
            boundary_kind: None,
        }
        "###
    );
}

#[test]
fn classifies_vue_boundaries() {
    assert_eq!(
        classify_vite_plugin_request("/src/Foo.client.vue")
            .boundary_kind
            .as_deref(),
        Some("client")
    );
    assert_eq!(
        classify_vite_plugin_request("/src/Foo.server.vue")
            .boundary_kind
            .as_deref(),
        Some("server")
    );
    assert!(
        classify_vite_plugin_request("/src/Foo.vue")
            .boundary_kind
            .is_none()
    );
}

#[test]
fn creates_and_normalizes_virtual_ids() {
    let client = create_virtual_id("/src/Foo.vue", false);
    let ssr = create_virtual_id("/src/Foo.vue", true);

    assert_eq!(client.as_str(), "\0/src/Foo.vue.ts");
    assert_eq!(ssr.as_str(), "\0vize-ssr:/src/Foo.vue.ts");
    assert_eq!(from_virtual_id(client.as_str()).as_str(), "/src/Foo.vue");
    assert_eq!(from_virtual_id(ssr.as_str()).as_str(), "/src/Foo.vue");
    assert_eq!(
        normalize_virtual_vue_module_id("\0/src/Foo.vue.ts?macro=true").as_str(),
        "/src/Foo.vue?macro=true"
    );
}
