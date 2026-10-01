#[test]
fn lint_pragmas_do_not_turn_single_root_components_into_fragments() {
    let plain = vize_atelier_core::compile!("<div>hello</div>");
    for pragma in [
        "<!-- eslint-disable-next-line vue/no-v-html --><div>hello</div>",
        "<!-- oxlint-disable-next-line vue/no-v-html --><div>hello</div>",
        "<!-- eslint-disable vue/no-v-html --><div>hello</div>",
    ] {
        let compiled = vize_atelier_core::compile!(pragma);
        assert_eq!(compiled.code, plain.code, "{pragma}");
    }

    let ordinary = vize_atelier_core::compile!("<!-- note --><div>hello</div>");
    assert_ne!(ordinary.code, plain.code);
}
