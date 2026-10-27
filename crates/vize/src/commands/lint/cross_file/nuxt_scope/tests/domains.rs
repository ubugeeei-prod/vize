use super::fixture::Fixture;

#[test]
fn mixed_layout_and_page_locations_retain_the_whole_group() {
    let mut fixture = Fixture::new();
    fixture.source("src/pages/login.vue",
        "<script setup>definePageMeta({layout:'bare'})</script><template><section id=\"shell\">login</section></template>");
    let mut analyzer = fixture.analyzer(&[]);
    assert!(fixture.prove(&mut analyzer).0);
    let result = analyzer.analyze();
    let duplicate = result
        .diagnostics
        .iter()
        .find_map(|diagnostic| match &diagnostic.kind {
            vize_croquis_cf::CrossFileDiagnosticKind::DuplicateElementId { id, locations }
                if id == "shell" =>
            {
                Some(locations)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(duplicate.len(), 3);
}

#[test]
fn repeated_locations_in_one_alternative_are_never_filtered() {
    let mut fixture = Fixture::new();
    fixture.source("src/pages/login.vue",
        "<script setup>definePageMeta({layout:'bare'})</script><template><main><section id=\"auth-page\"></section><aside id=\"auth-page\"></aside></main></template>");
    let mut analyzer = fixture.analyzer(&[]);
    assert!(fixture.prove(&mut analyzer).0);
    let result = analyzer.analyze();
    let duplicate = result
        .diagnostics
        .iter()
        .find_map(|diagnostic| match &diagnostic.kind {
            vize_croquis_cf::CrossFileDiagnosticKind::DuplicateElementId { id, locations }
                if id == "auth-page" =>
            {
                Some(locations)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(duplicate.len(), 3);
}

#[test]
fn an_unknown_location_retains_the_whole_group_even_with_a_host_claim() {
    let mut fixture = Fixture::new();
    fixture.source(
        "src/components/Shared.vue",
        "<template><aside id=\"auth-page\"/></template>",
    );
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    // Defensive consumer law: a host claim cannot classify an ordinary component.
    analyzer.admit_nuxt_render_scope_once(true);
    assert!(
        Fixture::ids(&mut analyzer)
            .iter()
            .any(|id| id == "auth-page")
    );
}

#[test]
fn slot_multiplicity_guards_forwarding_and_active_carriers_are_unknown() {
    for source in [
        "<template><div id=\"shell\"><slot/><slot/></div></template>",
        "<template><div id=\"shell\"><slot v-for=\"item in [1,2]\"/></div></template>",
        "<template><div id=\"shell\"><template v-for=\"item in [1,2]\"><slot/></template></div></template>",
        "<template><div id=\"shell\"><Wrapper><slot/></Wrapper></div></template>",
        "<template><div id=\"shell\"><slot :name=\"selected\"/></div></template>",
        "<template><div id=\"shell\" onclick=\"document.body.appendChild(this.cloneNode(true))\"><slot/></div></template>",
        "<template><div id=\"shell\"><iframe src=\"/other-page\"/><slot/></div></template>",
    ] {
        let mut fixture = Fixture::new();
        fixture.source("src/layouts/bare.vue", source);
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{source}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2, "{source}");
    }
}

#[test]
fn page_transition_multiple_outlets_and_explicit_imports_remain_reported() {
    for (path, source) in [
        (
            "src/pages/login.vue",
            "<script setup>definePageMeta({layout:'bare',pageTransition:{mode:'in-out'}})</script><template><section id=\"auth-page\">login</section></template>",
        ),
        (
            "src/app.vue",
            "<template><NuxtLayout><NuxtPage/><NuxtPage/></NuxtLayout></template>",
        ),
        (
            "src/app.vue",
            "<template><NuxtLayout name=\"default\"/><NuxtLayout name=\"bare\"/><NuxtPage/></template>",
        ),
        (
            "src/app.vue",
            "<script setup>import A from './layouts/default.vue';import B from './layouts/bare.vue'</script><template><A/><B/><NuxtLayout><NuxtPage/></NuxtLayout></template>",
        ),
    ] {
        let mut fixture = Fixture::new();
        fixture.source(path, source);
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{source}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2, "{source}");
    }
}

#[test]
fn nested_parent_and_child_and_loop_errors_remain_reported() {
    let mut fixture = Fixture::new();
    fixture.source(
        "src/pages/orders.vue",
        "<template><main id=\"nested\"><NuxtPage/></main></template>",
    );
    fixture.source(
        "src/pages/orders/child.vue",
        "<template><section id=\"nested\">child</section></template>",
    );
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    assert!(Fixture::ids(&mut analyzer).iter().any(|id| id == "nested"));

    let mut fixture = Fixture::new();
    fixture.source(
        "src/pages/login.vue",
        "<template><section v-for=\"item in [1,2]\" id=\"auth-page\">{{item}}</section></template>",
    );
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        vize_croquis_cf::CrossFileDiagnosticKind::NonUniqueIdInLoop { .. }
    )));
}

#[test]
fn private_render_carriers_preserve_the_original_semantic_debug_projection() {
    let source = "<template><main id=\"debug\"/></template>";
    let mut captured = super::super::super::sfc::analyze_sfc_for_cross_file(
        source,
        std::path::Path::new("src/pages/debug.vue"),
    )
    .unwrap();
    let original = vize_l0::cstr!("{:?}", captured.analysis);
    for script in [None, Some(false), Some(true)] {
        captured.analysis.nuxt_static_render_script = script;
        if let Some(template) = captured.analysis.nuxt_render_template.as_mut() {
            template.admitted = false;
            template.opaque_structure = true;
            template.slot_outlets = 99;
        }
        assert_eq!(vize_l0::cstr!("{:?}", captured.analysis), original);
    }
}
