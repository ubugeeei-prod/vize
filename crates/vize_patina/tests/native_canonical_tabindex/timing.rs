use super::support::{jsx, options, profile};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_patina::native::canonical::NativeCanonicalLint;

#[test]
#[ignore = "manual source-phase measurement in a quiet single-test process"]
fn canonical_tabindex_source_phase_measurement() {
    let a = Allocator::default();
    let mut body = vize_l0::String::new("<main>");
    for index in 0..64 {
        body.push_str(if index % 4 == 0 {
            "<p tabindex='1'/>"
        } else {
            "<p tabindex='-1'/>"
        });
    }
    body.push_str("</main>");
    let source = vize_l0::cstr!("<template>{body}</template>");
    let vue = lower_sfc_native(&a, &source, options());
    let lint = NativeCanonicalLint::from_vue(vue.admitted().unwrap());
    profile(&lint, "native-vue-static-64", 16);
    let source = vize_l0::cstr!("const view = {body};");
    let jsx = jsx(&a, &source, true);
    let lint = NativeCanonicalLint::from_jsx(&jsx).unwrap();
    profile(&lint, "native-tsx-static-64", 16);
}
