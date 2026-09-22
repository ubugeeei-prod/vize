//! `vize check`'s template-binding check, reading the `UndefinedRefs` fact
//! group through a declared demand (Davinci P4-3a).

use vize_carton::cstr;
use vize_croquis::facts::{CroquisFacts, Demand, FactConsumer, FactGroup, UndefinedRefs};

use super::{SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity};

/// The template-binding check reads the template's unresolved reads.
struct TemplateBindingCheck;

impl FactConsumer for TemplateBindingCheck {
    const NAME: &'static str = "canon/template-bindings";
    const DEMAND: Demand = Demand::NONE.with(UndefinedRefs::ID);
}

pub fn check_template_bindings(
    summary: &vize_croquis::Croquis,
    template_offset: u32,
    result: &mut SfcTypeCheckResult,
    _strict: bool,
    suppress_options_api_setup_spread_refs: bool,
) {
    let mut facts = CroquisFacts::new(summary);
    let view = facts.prepare::<TemplateBindingCheck>();
    let undefined = view.get::<UndefinedRefs>().expect("declared demand");
    for (_, undef_ref) in undefined.iter() {
        if suppress_options_api_setup_spread_refs && undef_ref.context == "template expression" {
            continue;
        }
        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: cstr!(
                "Undefined reference '{}' in {}",
                undef_ref.name,
                undef_ref.context
            ),
            start: undef_ref.offset + template_offset,
            end: undef_ref.offset + template_offset + undef_ref.name.len() as u32,
            code: Some("undefined-binding".into()),
            help: Some(cstr!(
                "Make sure '{}' is defined in script setup or imported",
                undef_ref.name
            )),
            related: Vec::new(),
        });
    }
}
