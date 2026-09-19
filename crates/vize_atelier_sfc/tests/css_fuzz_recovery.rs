#![cfg(feature = "native")]

mod support;
use support::css_fuzz_boundary as boundary;

#[test]
fn fuzz_boundary_distinguishes_recovered_engine_panics_from_escaping_panics() {
    let previous = std::panic::take_hook();
    boundary::install_hook();
    let source = include_str!("fixtures/css-engine/6190.css");
    let recovered = boundary::call(|| vize_atelier_sfc::parse_css_ast(source, &Default::default()));
    let escaped = boundary::call(|| panic!("uncaught integration defect"));
    std::panic::set_hook(previous);
    let result = recovered.expect("public API must recover from upstream percentage panic");
    assert!(result.ast.is_none());
    assert_eq!(
        result.errors,
        [
            "CSS parse error: the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)"
        ]
    );
    assert!(
        escaped.is_err(),
        "an escaping panic must remain a fuzz failure"
    );
}
