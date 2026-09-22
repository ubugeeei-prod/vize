//! The auto-import code action's script-binding check, read through a
//! declared fact demand (Davinci P4-3a).
#![allow(clippy::disallowed_methods)]

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
pub(super) fn sfc_script_declares(content: &str, name: &str) -> bool {
    let options = vize_atelier_sfc::SfcParseOptions::default();
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return false;
    };
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
    view.get::<Bindings>()
        .expect("declared demand")
        .contains_binding(name)
}
