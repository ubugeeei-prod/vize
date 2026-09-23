//! The auto-import code action's script-binding check, read through a
//! declared fact demand (Davinci P4-3a).
#![expect(
    clippy::disallowed_methods,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts, Demand, FactConsumer, FactGroup};

/// The auto-import code action reads script bindings.
struct AutoImportBindingCheck;

impl FactConsumer for AutoImportBindingCheck {
    const NAME: &'static str = "maestro/code-action-auto-import";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// True when the SFC's script (setup or plain) declares `name` via Croquis'
/// binding analysis. Used by the auto-import code action to skip names the
/// user already imported / declared.
pub(super) fn sfc_script_declares(
    descriptor: &vize_atelier_sfc::SfcDescriptor,
    name: &str,
) -> bool {
    let script_content = descriptor
        .script_setup
        .as_ref()
        .map(|s| s.content.to_string())
        .or_else(|| descriptor.script.as_ref().map(|s| s.content.to_string()));
    let Some(script_content) = script_content else {
        return false;
    };
    let mut analyzer = vize_croquis::Drawer::with_options(vize_croquis::DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    if descriptor.script_setup.is_some() {
        analyzer.analyze_script_setup(&script_content);
    } else {
        analyzer.analyze_script_plain(&script_content);
    }
    let croquis = analyzer.finish();
    let mut facts = CroquisFacts::new(&croquis);
    let view = facts.prepare::<AutoImportBindingCheck>();
    // The demand is declared above; if it were ever missing, treat the name
    // as declared so no spurious auto-import is offered.
    view.get::<Bindings>()
        .map_or(true, |bindings| bindings.contains_binding(name))
}
