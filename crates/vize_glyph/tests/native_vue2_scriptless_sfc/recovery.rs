//! Actual static-only recovery cannot bypass body guards.

use super::refusals::refused;
use super::*;
use vize_glyph::native_doc::NativeVue2SfcRefusal as Refusal;

#[test]
fn static_only_recovery_and_original_missing_closes_do_not_escape_global_or_cursor_guards() {
    let source = "<template><div id=></div></template>";
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    let component = owner.descriptor().component().unwrap();
    assert!(component.bindings().is_empty());
    assert_eq!(
        component.errors()[0].offset,
        source.find("></div>").unwrap() as u32
    );
    refused(
        &owner,
        Refusal::ComponentRecovery {
            offset: source.find("></div>").unwrap() as u32,
        },
    );
    let source = "<template><a><span>plain</span></template>";
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    assert!(owner.descriptor().component().unwrap().errors().is_empty());
    refused(
        &owner,
        Refusal::Template(vize_glyph::native_doc::TemplateRefusal::Recovered {
            offset: "<a><span>plain</span>".len(),
        }),
    );
}
